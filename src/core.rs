use crate::neighbors::Direction;
use crate::{Coordinate, GeohashError, Neighbors, Rect};

use failure::Error;

static BASE32_CODES: &'static [char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a','b', 'c', 'd', 'e', 'f',
];

/// Encode a coordinate to a geohash with length `len`.
///
/// ### Examples
///
/// Encoding a coordinate to a length five geohash:
///
/// ```rust
/// let coord = geohash::Coordinate { x: -120.6623, y: 35.3003 };
///
/// let geohash_string = geohash::encode(coord, 5).expect("Invalid coordinate");
///
/// assert_eq!(geohash_string, "4d8c0");
/// ```
///
/// Encoding a coordinate to a length ten geohash:
///
/// ```rust
/// let coord = geohash::Coordinate { x: -120.6623, y: 35.3003 };
///
/// let geohash_string = geohash::encode(coord, 10).expect("Invalid coordinate");
///
/// assert_eq!(geohash_string, "4d8c0f1817");
/// ```
pub fn encode(c: Coordinate<f64>, len: usize) -> Result<String, Error> {
    let mut out = String::with_capacity(len);

    let mut bits_total: i8 = 0;
    let mut hash_value: usize = 0;
    let mut max_lat = 90f64;
    let mut min_lat = -90f64;
    let mut max_lon = 180f64;
    let mut min_lon = -180f64;

    if c.x < min_lon || c.x > max_lon || c.y < min_lat || c.y > max_lat {
        bail!(GeohashError::InvalidCoordinateRange { c });
    }

    while out.len() < len {
        for _ in 0..4 {
            if bits_total % 2 == 0 {
                let mid = (max_lon + min_lon) / 2f64;
                if c.x > mid {
                    hash_value = (hash_value << 1) + 1usize;
                    min_lon = mid;
                } else {
                    hash_value <<= 1;
                    max_lon = mid;
                }
            } else {
                let mid = (max_lat + min_lat) / 2f64;
                if c.y > mid {
                    hash_value = (hash_value << 1) + 1usize;
                    min_lat = mid;
                } else {
                    hash_value <<= 1;
                    max_lat = mid;
                }
            }
            bits_total += 1;
        }

        let code: char = BASE32_CODES[hash_value];
        out.push(code);
        hash_value = 0;
    }
    Ok(out)
}

/// Decode geohash string into latitude, longitude
///
/// Parameters:
/// Geohash encoded `&str`
///
/// Returns:
/// A four-element tuple describs a bound box:
/// * min_lat
/// * max_lat
/// * min_lon
/// * max_lon
pub fn decode_bbox(hash_str: &str) -> Result<Rect<f64>, Error> {
    let mut is_lon = true;
    let mut max_lat = 90f64;
    let mut min_lat = -90f64;
    let mut max_lon = 180f64;
    let mut min_lon = -180f64;
    let mut mid: f64;
    let mut hash_value: usize;

    for c in hash_str.chars() {
        hash_value = hash_value_of_char(c)?;

        for bs in 0..4 {
            let bit = (hash_value >> (3 - bs)) & 1usize;
            if is_lon {
                mid = (max_lon + min_lon) / 2f64;

                if bit == 1 {
                    min_lon = mid;
                } else {
                    max_lon = mid;
                }
            } else {
                mid = (max_lat + min_lat) / 2f64;

                if bit == 1 {
                    min_lat = mid;
                } else {
                    max_lat = mid;
                }
            }
            is_lon = !is_lon;
        }
    }

    Ok(Rect {
        min: Coordinate {
            x: min_lon,
            y: min_lat,
        },
        max: Coordinate {
            x: max_lon,
            y: max_lat,
        },
    })
}

fn hash_value_of_char(c: char) -> Result<usize, Error> {
    let ord = c as usize;
    if 48 <= ord && ord <= 57 {
        return Ok(ord - 48);
    } else if 97 <= ord && ord <= 102{
        return Ok(ord - 87);
    }
    Err(GeohashError::InvalidHashCharacter { character: c })?
}

/// Decode a geohash into a coordinate with some longitude/latitude error. The
/// return value is `(<coordinate>, <longitude error>, <latitude error>)`.
///
/// ### Examples
///
/// Decoding a length five geohash:
///
/// ```rust
/// let geohash_str = "4d8c0";
///
/// let decoded = geohash::decode(geohash_str).expect("Invalid hash string");
///
/// assert_eq!(
///     decoded,
///     (
///         geohash::Coordinate {
///             x: -120.76171875,
///             y: 35.244140625,
///         },
///         0.17578125,
///         0.087890625,
///     ),
/// );
/// ```
///
/// Decoding a length eight geohash:
///
/// ```rust
/// let geohash_str = "4d8c0f1817";
///
/// let decoded = geohash::decode(geohash_str).expect("Invalid hash string");
///
/// assert_eq!(
///     decoded,
///     (
///         geohash::Coordinate {
///             x: -120.66232681274414,
///             y: 35.30035972595215,
///         },
///         0.000171661376953125,
///         0.0000858306884765625,
///     ),
/// );
/// ```
pub fn decode(hash_str: &str) -> Result<(Coordinate<f64>, f64, f64), Error> {
    let rect = decode_bbox(hash_str)?;
    let c0 = rect.min;
    let c1 = rect.max;
    Ok((
            Coordinate {
                x: (c0.x + c1.x) / 2f64,
                y: (c0.y + c1.y) / 2f64,
            },
            (c1.x - c0.x) / 2f64,
            (c1.y - c0.y) / 2f64,
            ))
}

/// Find neighboring geohashes for the given geohash and direction.
pub fn neighbor(hash_str: &str, direction: Direction) -> Result<String, Error> {
    let (coord, lon_err, lat_err) = decode(hash_str)?;
    let neighbor_coord = match direction.to_tuple() {
        (dlat, dlng) => Coordinate {
            x: coord.x + 2f64 * lon_err.abs() * dlng,
            y: coord.y + 2f64 * lat_err.abs() * dlat,
        },
    };
    encode(neighbor_coord, hash_str.len())
}

/// Find all neighboring geohashes for the given geohash.
///
/// ### Examples
///
/// ```
/// let geohash_str = "4d8c0f1817";
///
/// let neighbors = geohash::neighbors(geohash_str).expect("Invalid hash string");
///
/// assert_eq!(
///     neighbors,
///     geohash::Neighbors {
///         n: "4d8c0f1842".to_owned(),
///         ne: "4d8c0f1848".to_owned(),
///         e: "4d8c0f181d".to_owned(),
///         se: "4d8c0f181c".to_owned(),
///         s: "4d8c0f1816".to_owned(),
///         sw: "4d8c0f1814".to_owned(),
///         w: "4d8c0f1815".to_owned(),
///         nw: "4d8c0f1840".to_owned(),
///     }
/// );
/// ```
pub fn neighbors(hash_str: &str) -> Result<Neighbors, Error> {
    Ok(Neighbors {
        sw: neighbor(hash_str, Direction::SW)?,
        s: neighbor(hash_str, Direction::S)?,
        se: neighbor(hash_str, Direction::SE)?,
        w: neighbor(hash_str, Direction::W)?,
        e: neighbor(hash_str, Direction::E)?,
        nw: neighbor(hash_str, Direction::NW)?,
        n: neighbor(hash_str, Direction::N)?,
        ne: neighbor(hash_str, Direction::NE)?,
    })
}
