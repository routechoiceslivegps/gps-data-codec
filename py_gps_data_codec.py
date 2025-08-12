"""
A Python implementation of Google's Encoded Polyline Algorithm Format.
"""
import io

YEAR2010 = 1262304000

def _write_unsigned(output, num):
    while num >= 0x20:
        output.write(chr((0x20 | (num & 0x1f)) + 63))
        num >>= 5
    output.write(chr(num + 63))


def _write_signed(output, num):
    sgn_num = num << 1
    if num < 0:
        sgn_num = ~sgn_num
    return _write_unsigned(output, sgn_num)


def _trans_unsigned(value, index):
    byte, result, shift = None, 0, 0
    while byte is None or byte >= 0x20:
        byte = ord(value[index]) - 63
        index += 1
        result |= (byte & 0x1f) << shift
        shift += 5
    return result, index


def _trans_signed(value, index):
    result, index = _trans_unsigned(value, index)
    comp = result & 1
    return ~(result >> 1) if comp else (result >> 1), index


def decode(expression):
    """
    Decode a polyline string into a set of coordinates.

    :param expression: Polyline string, e.g. 'u{~vFvyys@fS]'.
    :param precision: Precision of the encoded coordinates. Google Maps uses 5, OpenStreetMap uses 6.
        The default value is 5.
    :param geojson: Set output of tuples to (lon, lat), as per https://tools.ietf.org/html/rfc7946#section-3.1.1
    :return: List of coordinate tuples in (lat, lon) order, unless geojson is set to True.
    """
    tim, lat, lng = (YEAR2010, 0, 0)
    coordinates = []
    index = 0
    length = len(expression)
    while index < length:
        if index == 0:
            tim_change, index = _trans_signed(expression, index)
        else:
            tim_change, index = _trans_unsigned(expression, index)
        tim += tim_change
        lat_change, index = _trans_signed(expression, index)
        lat += lat_change
        lng_change, index = _trans_signed(expression, index)
        lng += lng_change
        coordinates.append((tim, lat / 1e5, lng / 1e5))
    return coordinates


def encode(coordinates):
    """
    Encode a set of coordinates in a polyline string.

    :param coordinates: List of coordinate tuples, e.g. [(0, 0), (1, 0)]. Unless geojson is set to True, the order
        is expected to be (lat, lon).
    :param precision: Precision of the coordinates to encode. Google Maps uses 5, OpenStreetMap uses 6.
        The default value is 5.
    :param geojson: Set to True in order to encode (lon, lat) tuples.
    :return: The encoded polyline string.
    """
    output = io.StringIO()
    p_t = YEAR2010
    p_la = 0
    p_lo = 0
    for i, curr in enumerate(coordinates):
        t_d = round(curr[0] - p_t)
        if i == 0:
            _write_signed(output, t_d)
        else:
            _write_unsigned(output, t_d)
        p_t += t_d

        la_d = round((curr[1] - p_la) * 1e5)
        _write_signed(output, la_d)
        p_la += la_d / 1e5
        
        lo_d = round((curr[2] - p_lo) * 1e5)
        _write_signed(output, lo_d)
        p_lo += lo_d / 1e5

    return output.getvalue()