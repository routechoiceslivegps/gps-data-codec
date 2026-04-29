import gps_data_codec


def test_lib():
    gps_data = [
        (-1,0,0),
        (1628667993, 4.56543, -110.53621),
        (1628667994, 4.56553, -110.53625)
    ]
    bad_gps_data = [
        (-1,0,0),
        (1628667995, 4.56543, -110.53621),
        (1628667994, 4.56553, -110.53625)
    ]
    expected_encoded = '`o|sfjA??ya_fpo@}tzZhbtaT@SF'
    
    print("Encode/decode:")
    encoded = gps_data_codec.encode(gps_data)
    print(f"expected: {expected_encoded}")
    print(f"actual:   {encoded}")
    assert(encoded == expected_encoded)
    output = gps_data_codec.decode(encoded)
    assert(output == gps_data)
    
    exception_raised = False
    try:
        gps_data_codec.encode(bad_gps_data)
    except ValueError as e:
        exception_raised = "Input data is not sorted" == f"{e}"
    assert(exception_raised)
    print()

    print("Extract Interval (bound within):")
    expected_encoded = "qtaxyT}tzZhbtaT"
    encoded, nb_pts = gps_data_codec.extract_encoded_interval(
         expected_encoded,
         0,
         1628667993
    )
    print(f"expected: {expected_encoded}")
    print(f"actual:   {encoded}")
    assert(expected_encoded == encoded)
    assert(nb_pts == 1)
    print()

    print("Extract Interval (bound includes start):")
    expected_encoded = "`o|sfjA??ya_fpo@}tzZhbtaT"
    encoded, nb_pts = gps_data_codec.extract_encoded_interval(
         expected_encoded,
         -2,
         1628667993
    )
    print(f"expected: {expected_encoded}")
    print(f"actual:   {encoded}")
    assert(expected_encoded == encoded)
    assert(nb_pts == 2)
    print()
    
    print("Extract Interval (bound includes end):")
    expected_encoded = "qtaxyT}tzZhbtaT@SF"
    encoded, nb_pts = gps_data_codec.extract_encoded_interval(
         expected_encoded,
         0,
         2628667993
    )
    print(f"expected: {expected_encoded}")
    print(f"actual:   {encoded}")
    assert(expected_encoded == encoded)
    assert(nb_pts == 2)
    print()

    print("Encoded Diff:")
    a = gps_data_codec.encode([(1, 0, 0), (2, 0, 0), (4, 0, 0)])
    b = gps_data_codec.encode([(2, 0, 0), (3, 0, 0), (4, 0 ,0), (6, 0, 0)])
    expected_encoded = "xn|sfjA??B??"
    c = gps_data_codec.encoded_diff(a, b)
    print(f"expected: {expected_encoded}")
    print(f"actual:   {c}")
    print(f"new timestamps: {[pt[0] for pt in gps_data_codec.decode(c)]}")

    assert(c == expected_encoded)
    print()
    
    print("First point:")
    first_point = gps_data_codec.decode_first_location("`o|sfjA??ya_fpo@}tz")
    expected = (-1,0.0,0.0)
    print(f"expected: {expected}")
    print(f"actual:   {first_point}")
    assert(first_point == expected)
    print()
    
    print("Perormance testing:")
    import time
    with open('test_data.txt', "r") as fp:
        data = fp.read()
    t0 = time.perf_counter()
    x1 = gps_data_codec.decode(data)
    t1 = time.perf_counter()
    s1 = gps_data_codec.encode(x1)
    t2 = time.perf_counter()
    print("-- Rust --")
    print("Decoding: ", t1 - t0)
    print("Encoding: ", t2 - t1)
    print("Total: ", t2 - t0)
    assert(data == s1)


if __name__ == "__main__":
    test_lib()

