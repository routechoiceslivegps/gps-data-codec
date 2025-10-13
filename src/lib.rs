use pyo3::pymodule;

#[pymodule]
mod gps_data_codec {
    use pyo3::PyResult;
    use pyo3::exceptions::PyValueError;
    use pyo3::pyfunction;

    struct DecodingResult {
        value: i64,
        offset: u32,
    }

    fn decode_unsigned_value_from_string<'a>(
        slice: &mut impl Iterator<Item = &'a u8>,
    ) -> DecodingResult {
        let mut result: i64 = 0;
        let mut shift = 0;
        let mut position: u32 = 0;
        loop {
            let byte = slice.next().unwrap() - 63;
            position += 1;
            if (byte & 0x20) == 0 {
                result |= (byte as i64) << shift;
                return DecodingResult {
                    value: result,
                    offset: position,
                };
            } else {
                result |= ((byte & 0x1f) as i64) << shift;
            }
            shift += 5
        }
    }

    fn decode_signed_value_from_string<'a>(
        encoded: &mut impl Iterator<Item = &'a u8>,
    ) -> DecodingResult {
        let tmp_result: DecodingResult = decode_unsigned_value_from_string(encoded);
        if tmp_result.value & 1 == 1 {
            DecodingResult {
                value: !(tmp_result.value >> 1),
                offset: tmp_result.offset,
            }
        } else {
            DecodingResult {
                value: tmp_result.value >> 1,
                offset: tmp_result.offset,
            }
        }
    }

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

    fn encode_signed_number(out: &mut Vec<u8>, num: i64) {
        let mut sgn_num: i64 = num << 1;
        if num < 0 {
            sgn_num = !sgn_num;
        }
        let unsigned_num = sgn_num as u64;
        encode_unsigned_number(out, unsigned_num);
    }

    const YEAR2010: i64 = 1262304000;

    #[pyfunction]
    fn decode(input: String) -> PyResult<Vec<(i64, f64, f64)>> {
        let mut encoded = input.as_bytes().iter();
        let encoded_length: u32 = encoded.len() as u32;
        let mut bytes_consumed: u32 = 0;
        let mut timestamp: i64 = YEAR2010;
        let mut latitude: i64 = 0;
        let mut longitude: i64 = 0;
        let mut output: Vec<(i64, f64, f64)> = vec![];

        while bytes_consumed < encoded_length {
            if bytes_consumed == 0 {
                let decoding_result = decode_signed_value_from_string(&mut encoded);
                bytes_consumed += decoding_result.offset;
                timestamp += decoding_result.value;
            } else {
                let decoding_result = decode_unsigned_value_from_string(&mut encoded);
                bytes_consumed += decoding_result.offset;
                timestamp += decoding_result.value;
            }

            let decoding_result = decode_signed_value_from_string(&mut encoded);
            bytes_consumed += decoding_result.offset;
            latitude += decoding_result.value;

            let decoding_result = decode_signed_value_from_string(&mut encoded);
            bytes_consumed += decoding_result.offset;
            longitude += decoding_result.value;

            output.push((timestamp, (latitude as f64) / 1e5, (longitude as f64) / 1e5));
        }
        Ok(output)
    }
    
    #[pyfunction]
    fn encoded_diff(prev_input: String, input: String) -> PyResult<String> {
        let encoded_p = prev_input.as_bytes();
        let mut encoded_pi = encoded_p.iter();
        let encoded_p_length: u32 = encoded_p.len() as u32;

        let mut bytes_consumed_p: u32 = 0;
        let mut timestamp_p: i64 = YEAR2010;
        
        let encoded = input.as_bytes();
        let mut encoded_i = encoded.iter();
        let encoded_length: u32 = encoded.len() as u32;

        let mut bytes_consumed: u32 = 0;
        let mut timestamp: i64 = YEAR2010;
        let mut latitude: i64 = 0;
        let mut longitude: i64 = 0;

        let mut prev_timestamp: i64 = YEAR2010;
        let mut prev_latitude: i64 = 0;
        let mut prev_longitude: i64 = 0;

        let mut output: Vec<u8> = vec![];
        let mut is_first: bool = true;

        // At first decode until prev data is exhausted
        while bytes_consumed_p < encoded_p_length && bytes_consumed < encoded_length {
            // We decode one point of both data
            if bytes_consumed == 0 {
                let decoding_result = decode_signed_value_from_string(&mut encoded_i);
                timestamp += decoding_result.value;
                bytes_consumed += decoding_result.offset;

            } else {
                let decoding_result = decode_unsigned_value_from_string(&mut encoded_i);
                timestamp += decoding_result.value;
                bytes_consumed += decoding_result.offset;
            }

            if bytes_consumed_p == 0 {
                let decoding_result = decode_signed_value_from_string(&mut encoded_pi);
                timestamp_p += decoding_result.value;
                bytes_consumed_p += decoding_result.offset;
            } else {
                let decoding_result = decode_unsigned_value_from_string(&mut encoded_pi);
                timestamp_p += decoding_result.value;
                bytes_consumed_p += decoding_result.offset;
            }

            let lat_decoding_result = decode_signed_value_from_string(&mut encoded_i);
            bytes_consumed += lat_decoding_result.offset;
            latitude += lat_decoding_result.value;
            let lng_decoding_result = decode_signed_value_from_string(&mut encoded_i);
            bytes_consumed += lng_decoding_result.offset;
            longitude += lng_decoding_result.value;
            
            let decoding_result = decode_signed_value_from_string(&mut encoded_pi);
            bytes_consumed_p += decoding_result.offset;
            let decoding_result = decode_signed_value_from_string(&mut encoded_pi);
            bytes_consumed_p += decoding_result.offset;
            
            
            // if the older data is exhausted stop and next loop will write data left in newest stream
            if bytes_consumed_p >= encoded_p_length {
                break;
            }
            // as long the timestamp differ write the newest data
            while timestamp != timestamp_p {
                // write the point that is discovered
                let timestamp_diff = timestamp - prev_timestamp;
                let latitude_diff: i64 = latitude - prev_latitude;
                let longitude_diff: i64 = longitude - prev_longitude;

                prev_timestamp += timestamp_diff;
                prev_latitude += latitude_diff;
                prev_longitude += longitude_diff;

                if is_first {
                    encode_signed_number(&mut output, timestamp_diff);
                    is_first = false;
                } else {
                    encode_unsigned_number(&mut output, timestamp_diff as u64);
                }
                encode_signed_number(&mut output, latitude_diff);
                encode_signed_number(&mut output, longitude_diff);
                
                // if newest stream is exhausted stop reading
                if bytes_consumed >= encoded_length {
                    break;
                }
                
                // read next point
                let decoding_result = decode_unsigned_value_from_string(&mut encoded_i);
                bytes_consumed += decoding_result.offset;
                timestamp += decoding_result.value;

                let lat_decoding_result = decode_signed_value_from_string(&mut encoded_i);
                bytes_consumed += lat_decoding_result.offset;
                latitude += lat_decoding_result.value;

                let lng_decoding_result = decode_signed_value_from_string(&mut encoded_i);
                bytes_consumed += lng_decoding_result.offset;
                longitude += lng_decoding_result.value;
            }
        }

        // if there is still data in latest stream 
        if bytes_consumed_p >= encoded_p_length && bytes_consumed < encoded_length {
            // if the last point was same in both stream read next point
            if timestamp == timestamp_p {
                if bytes_consumed == 0 {
                    let decoding_result = decode_signed_value_from_string(&mut encoded_i);
                    timestamp += decoding_result.value;
                    bytes_consumed += decoding_result.offset;
                } else {
                    let decoding_result = decode_unsigned_value_from_string(&mut encoded_i);
                    bytes_consumed += decoding_result.offset;
                    timestamp += decoding_result.value;
                }
                let lat_decoding_result = decode_signed_value_from_string(&mut encoded_i);
                bytes_consumed += lat_decoding_result.offset;
                latitude += lat_decoding_result.value;

                let lng_decoding_result = decode_signed_value_from_string(&mut encoded_i);
                bytes_consumed += lng_decoding_result.offset;
                longitude += lng_decoding_result.value;
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
            output.append(&mut encoded[bytes_consumed as usize..].to_vec())
        }
        Ok(unsafe { String::from_utf8_unchecked(output) })
    }

    #[pyfunction]
    fn extract_encoded_interval(input: String, from_ts: i64, end_ts: i64) -> PyResult<(String, usize)> {
        let encoded = input.as_bytes();
        let mut encodedi = encoded.iter();
        let encoded_length: u32 = encoded.len() as u32;
        let mut bytes_consumed: u32 = 0;
        let mut timestamp: i64 = YEAR2010;
        let mut latitude: i64 = 0;
        let mut longitude: i64 = 0;
        let mut start_found = false;
        let mut prev_idx: u32 = 0;
        let mut output: Vec<u8> = vec![];
        let mut nb_pts = 0;
        while bytes_consumed < encoded_length {
            if bytes_consumed == 0 {
                let decoding_result = decode_signed_value_from_string(&mut encodedi);
                bytes_consumed += decoding_result.offset;
                timestamp += decoding_result.value;
            } else {
                let decoding_result = decode_unsigned_value_from_string(&mut encodedi);
                bytes_consumed += decoding_result.offset;
                timestamp += decoding_result.value;
            }
            
            let lat_decoding_result = decode_signed_value_from_string(&mut encodedi);
            bytes_consumed += lat_decoding_result.offset;
            let lng_decoding_result = decode_signed_value_from_string(&mut encodedi);
            bytes_consumed += lng_decoding_result.offset;

            if !start_found {
                latitude += lat_decoding_result.value;
                longitude += lng_decoding_result.value;
                if timestamp >= from_ts && timestamp <= end_ts {
                    start_found = true;
                    encode_signed_number(&mut output, timestamp - YEAR2010);
                    encode_signed_number(&mut output, latitude);
                    encode_signed_number(&mut output, longitude);
                    prev_idx = bytes_consumed;
                    nb_pts += 1;
                }  
            } else if timestamp <= end_ts  {
                output.append(&mut encoded[prev_idx as usize..bytes_consumed as usize].to_vec());
                prev_idx = bytes_consumed;
                nb_pts += 1;
            } else if timestamp > end_ts {
                break;
            }
        }
        Ok((unsafe { String::from_utf8_unchecked(output) }, nb_pts))
    }
    #[pyfunction]
    fn encode(data: Vec<(i64, f64, f64)>) -> PyResult<String> {
        let mut prev_timestamp: i64 = YEAR2010;
        let mut prev_latitude: i64 = 0;
        let mut prev_longitude: i64 = 0;

        let mut output: Vec<u8> = vec![];
        let mut is_first: bool = true;
        for (timestamp, latitude, longitude) in data.iter() {
            let timestamp_diff = timestamp - prev_timestamp;
            let latitude_diff: i64 = (latitude * 1e5).round() as i64 - prev_latitude;
            let longitude_diff: i64 = (longitude * 1e5).round() as i64 - prev_longitude;

            prev_timestamp += timestamp_diff;
            prev_latitude += latitude_diff;
            prev_longitude += longitude_diff;

            if is_first {
                encode_signed_number(&mut output, timestamp_diff);
                is_first = false;
            } else {
                if *timestamp < prev_timestamp {
                    return Err(PyValueError::new_err("Input data is not sorted"));
                }
                encode_unsigned_number(&mut output, timestamp_diff as u64);
            }
            encode_signed_number(&mut output, latitude_diff);
            encode_signed_number(&mut output, longitude_diff);
        }
        Ok(unsafe { String::from_utf8_unchecked(output) })
    }
}
