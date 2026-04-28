use pyo3::pymodule;

#[pymodule]
mod gps_data_codec {
    use pyo3::PyResult;
    use pyo3::exceptions::PyValueError;
    use pyo3::pyfunction;

    #[inline(always)]
    fn decode_unsigned_value(data: &[u8], idx: &mut usize) -> i64 {
        let mut result: i64 = 0;
        let mut shift = 0;
        loop {
            let byte = data[*idx] - 63;
            *idx += 1;
            if (byte & 0x20) == 0 {
                result |= (byte as i64) << shift;
                return result;
            } else {
                result |= ((byte & 0x1f) as i64) << shift;
            }
            shift += 5;
        }
    }

    #[inline(always)]
    fn decode_signed_value(data: &[u8], idx: &mut usize) -> i64 {
        let val = decode_unsigned_value(data, idx);
        if val & 1 == 1 { !(val >> 1) } else { val >> 1 }
    }

    #[inline(always)]
    fn encode_unsigned_number(out: &mut Vec<u8>, mut num: u64) {
        loop {
            if num < 0x20 {
                out.push(num as u8 + 63);
                break;
            } else {
                out.push(((num as u8 & 0x1f) | 0x20) + 63);
                num >>= 5;
            }
        }
    }

    #[inline(always)]
    fn encode_signed_number(out: &mut Vec<u8>, num: i64) {
        let sgn_num = if num < 0 { !(num << 1) } else { num << 1 };
        encode_unsigned_number(out, sgn_num as u64);
    }

    const YEAR2010: i64 = 1262304000;

    #[pyfunction]
    fn decode(input: &str) -> PyResult<Vec<(i64, f64, f64)>> {
        let data = input.as_bytes();
        let len = data.len();
        let mut idx: usize = 0;
        let mut timestamp: i64 = YEAR2010;
        let mut latitude: i64 = 0;
        let mut longitude: i64 = 0;
        // Estimate ~3 bytes per point minimum (3 fields, 1 bytes each)
        let mut output: Vec<(i64, f64, f64)> = Vec::with_capacity(len / 3 + 1);

        while idx < len {
            if idx == 0 {
                timestamp += decode_signed_value(data, &mut idx);
            } else {
                timestamp += decode_unsigned_value(data, &mut idx);
            }

            latitude += decode_signed_value(data, &mut idx);
            longitude += decode_signed_value(data, &mut idx);

            output.push((timestamp, (latitude as f64) / 1e5, (longitude as f64) / 1e5));
        }
        Ok(output)
    }

    #[pyfunction]
    fn decode_first_location(input: &str) -> PyResult<(i64, f64, f64)> {
        let data = input.as_bytes();
        let mut idx: usize = 0;

        let timestamp = YEAR2010 + decode_signed_value(data, &mut idx);
        let latitude = decode_signed_value(data, &mut idx);
        let longitude = decode_signed_value(data, &mut idx);

        Ok((timestamp, (latitude as f64) / 1e5, (longitude as f64) / 1e5))
    }
    
    #[pyfunction]
    fn encoded_diff(prev_input: &str, input: &str) -> PyResult<String> {
        let encoded_p = prev_input.as_bytes();
        let encoded_p_length = encoded_p.len();
        let mut idx_p: usize = 0;
        let mut timestamp_p: i64 = YEAR2010;
        
        let encoded = input.as_bytes();
        let encoded_length = encoded.len();
        let mut idx: usize = 0;

        let mut timestamp: i64 = YEAR2010;
        let mut latitude: i64 = 0;
        let mut longitude: i64 = 0;

        let mut prev_timestamp: i64 = YEAR2010;
        let mut prev_latitude: i64 = 0;
        let mut prev_longitude: i64 = 0;

        let mut output: Vec<u8> = Vec::with_capacity(encoded_length);
        let mut is_first: bool = true;

        // At first decode until prev data is exhausted
        while idx_p < encoded_p_length && idx < encoded_length {
            // We decode one point of both data
            if idx == 0 {
                timestamp += decode_signed_value(encoded, &mut idx);
            } else {
                timestamp += decode_unsigned_value(encoded, &mut idx);
            }

            if idx_p == 0 {
                timestamp_p += decode_signed_value(encoded_p, &mut idx_p);
            } else {
                timestamp_p += decode_unsigned_value(encoded_p, &mut idx_p);
            }

            latitude += decode_signed_value(encoded, &mut idx);
            longitude += decode_signed_value(encoded, &mut idx);
            
            decode_signed_value(encoded_p, &mut idx_p);
            decode_signed_value(encoded_p, &mut idx_p);
            
            // if the older data is exhausted stop and next loop will write data left in newest stream
            if idx_p >= encoded_p_length {
                break;
            }
            // as long the timestamp differ write the newest data
            while timestamp != timestamp_p {
                // write the point that is discovered
                let timestamp_diff = timestamp - prev_timestamp;
                let latitude_diff: i64 = latitude - prev_latitude;
                let longitude_diff: i64 = longitude - prev_longitude;

                if is_first {
                    encode_signed_number(&mut output, timestamp_diff);
                    is_first = false;
                } else {
                    encode_unsigned_number(&mut output, timestamp_diff as u64);
                }
                encode_signed_number(&mut output, latitude_diff);
                encode_signed_number(&mut output, longitude_diff);
                                
                prev_timestamp = timestamp;
                prev_latitude = latitude;
                prev_longitude = longitude;

                // if newest stream is exhausted stop reading
                if idx >= encoded_length {
                    break;
                }

                // read next point
                timestamp += decode_unsigned_value(encoded, &mut idx);
                latitude += decode_signed_value(encoded, &mut idx);
                longitude += decode_signed_value(encoded, &mut idx);
            }
        }

        // if there is still data in latest stream 
        if idx_p >= encoded_p_length && idx < encoded_length {
            // if the last point was same in both stream read next point
            if timestamp == timestamp_p {
                if idx == 0 {
                    timestamp += decode_signed_value(encoded, &mut idx);
                } else {
                    timestamp += decode_unsigned_value(encoded, &mut idx);
                }
                latitude += decode_signed_value(encoded, &mut idx);
                longitude += decode_signed_value(encoded, &mut idx);
            }
        
            // write the latest point that differ
            let timestamp_diff = timestamp - prev_timestamp;
            let latitude_diff: i64 = latitude - prev_latitude;
            let longitude_diff: i64 = longitude - prev_longitude;

            if is_first {
                encode_signed_number(&mut output, timestamp_diff);
            } else {
                encode_unsigned_number(&mut output, timestamp_diff as u64);
            }
            encode_signed_number(&mut output, latitude_diff);
            encode_signed_number(&mut output, longitude_diff);
            
            // the following data stay the same
            output.extend_from_slice(&encoded[idx..]);
        }
        Ok(unsafe { String::from_utf8_unchecked(output) })
    }

    #[pyfunction]
    fn extract_encoded_interval(input: &str, from_ts: i64, end_ts: i64) -> PyResult<(String, usize)> {
        let encoded = input.as_bytes();
        let encoded_length = encoded.len();
        let mut idx: usize = 0;
        let mut timestamp: i64 = YEAR2010;
        let mut latitude: i64 = 0;
        let mut longitude: i64 = 0;
    
        let mut start_found = false;
        let mut start_idx: usize = 0;
        let mut end_idx: usize = 0;
    
        let mut output: Vec<u8> = Vec::with_capacity(encoded_length);
        let mut nb_pts = 0;
    
        while idx < encoded_length {
            if idx == 0 {
                timestamp += decode_signed_value(encoded, &mut idx);
            } else {
                timestamp += decode_unsigned_value(encoded, &mut idx);
            }
            
            let lat_diff = decode_signed_value(encoded, &mut idx);
            let lng_diff = decode_signed_value(encoded, &mut idx);

            if !start_found {
                latitude += lat_diff;
                longitude += lng_diff;
                if timestamp >= from_ts && timestamp <= end_ts {
                    start_found = true;
                    start_idx = idx;
                    encode_signed_number(&mut output, timestamp - YEAR2010);
                    encode_signed_number(&mut output, latitude);
                    encode_signed_number(&mut output, longitude);
                    nb_pts += 1;
                }  
            } else if timestamp <= end_ts  {
                end_idx = idx;
                nb_pts += 1;
            } else {
                break;
            }
        }
        if nb_pts > 1 {
            output.extend_from_slice(&encoded[start_idx..end_idx]);
        }
        Ok((unsafe { String::from_utf8_unchecked(output) }, nb_pts))
    }

    #[pyfunction]
    fn encode(data: Vec<(i64, f64, f64)>) -> PyResult<String> {
        let mut prev_timestamp: i64 = YEAR2010;
        let mut prev_latitude: i64 = 0;
        let mut prev_longitude: i64 = 0;

        // ~6 bytes per point estimate
        let mut output: Vec<u8> = Vec::with_capacity(data.len() * 6);
        let mut is_first: bool = true;
        for (timestamp, latitude, longitude) in data.iter() {
            let timestamp_diff = timestamp - prev_timestamp;
            let latitude_diff: i64 = (latitude * 1e5).round() as i64 - prev_latitude;
            let longitude_diff: i64 = (longitude * 1e5).round() as i64 - prev_longitude;

            if is_first {
                encode_signed_number(&mut output, timestamp_diff);
                is_first = false;
            } else {
                if timestamp_diff < 0 {
                    return Err(PyValueError::new_err("Input data is not sorted"));
                }
                encode_unsigned_number(&mut output, timestamp_diff as u64);
            }
            encode_signed_number(&mut output, latitude_diff);
            encode_signed_number(&mut output, longitude_diff);

            prev_timestamp += timestamp_diff;
            prev_latitude += latitude_diff;
            prev_longitude += longitude_diff;
        }
        Ok(unsafe { String::from_utf8_unchecked(output) })
    }
}
