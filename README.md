# gps-data-codec

Python library, implemented in rust, including base functions for encoding and decoding of gps data (timestamp, latitude, and longitude) using a algorithm similar to the one seen on encoded polylines.
It is used to store competitors data on https://www.routechoices.com

## install
 
```
pip install gps-data-codec
```


```
>> import gps_data_codec
>> gps_data_codec.encode([(1628667993, 4.56543, -110.536214), ]) # [(time, lat, lon), ...]
'qtaxyT}tzZhbtaT'
>> gps_data_codec.decode('qtaxyT}tzZhbtaT')
[(1628...
```

# Warning:  
  - The list of GPS points must be sorted by increasing timestamps.
