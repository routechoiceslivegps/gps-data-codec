import gps_data_codec

import py_gps_data_codec

def test_lib():
    input = [(-1,0,0),(1628667993, 4.56543, -110.53621), (1628667994, 4.56553, -110.53625)]
    expected_encoded = '`o|sfjA??ya_fpo@}tzZhbtaT@SF'
    encoded = gps_data_codec.encode(input)
    assert(encoded == expected_encoded)
    output = gps_data_codec.decode(encoded)
    assert(output == input)
    import time
    with open('test_data.txt', "r") as fp:
        data = fp.read()
    t0 = time.time()
    x1 = gps_data_codec.decode(data)
    t1 = time.time()
    s1 = gps_data_codec.encode(x1)
    t2 = time.time()
    print("-- Rust --")
    print("Decoding: ", t1 - t0)
    print("Encoding: ", t2 - t1)
    print("Total: ", t2 - t0)
    t0 = time.time()
    x2 = py_gps_data_codec.decode(data)
    t1 = time.time()
    s2 = py_gps_data_codec.encode(x1)
    t2 = time.time()
    assert(x1 == x2)
    assert(s1 == s2)
    print("-- Python --")
    print("Decoding: ", t1 - t0)
    print("Encoding: ", t2 - t1)
    print("Total: ", t2 - t0)


if __name__ == "__main__":
    test_lib()
