# gps-data-codec

Python library, implemented in Rust, that include base functions for encoding and decoding series of gps data (timestamp, latitude, longitude) using a algorithm similar to the one seen on encoded polylines.

## Install
 
```
pip install gps-data-codec
```

# Usage
```
>> from gps_data_codec import decode, encode
>> encode([(1628667993, 4.56543, -110.536214)]) # [(unix epoch in seconds, latitude, longitude), ...]
'qtaxyT}tzZhbtaT'
>> decode('qtaxyT}tzZhbtaT')
[(1628667993, 4.56543, -110.53621)]
```

## Warning:
  - timestamps are rounded to the closest integer value.
  - latitudes and longitudes values are rounded to the 5th decimal precision when encoding.
  - The series of timestamped locations must be sorted by timestamps in increasing order before encoding.
