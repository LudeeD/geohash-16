extern crate geo_types;
extern crate geohash;

use geohash::{decode, encode, neighbors, Coordinate};

#[test]
fn test_encode() {
    let c0 = Coordinate {
        x: 112.5584f64,
        y: 37.8324f64,
    };
    assert_eq!(encode(c0, 12usize).unwrap(), "e71150dc9947".to_string());

    let c1 = Coordinate {
        x: 117f64,
        y: 32f64,
    };
    assert_eq!(encode(c1, 3usize).unwrap(), "e65".to_string());

    let c2 = Coordinate {
        x: 190f64,
        y: -100f64,
    };
    assert!(encode(c2, 3usize).is_err());
}

fn compare_within(a: f64, b: f64, diff: f64) {
    assert!(
        (a - b).abs() < diff,
        format!("{:?} and {:?} should be within {:?}", a, b, diff)
    );
}

fn compare_decode(gh: &str, exp_lon: f64, exp_lat: f64, exp_lon_err: f64, exp_lat_err: f64) {
    let (coord, lon_err, lat_err) = decode(gh).unwrap();
    let diff = 1e-5f64;
    compare_within(lon_err, exp_lon_err, diff);
    compare_within(lat_err, exp_lat_err, diff);
    compare_within(coord.x, exp_lon, diff);
    compare_within(coord.y, exp_lat, diff);
}

#[test]
fn test_decode() {
    compare_decode("e71150", 112.543945, 37.814941, 0.0439453125, 0.02197265625);
    compare_decode("e65b4a", 117.02636 , 32.01416, 0.0439453125, 0.02197265625);

    assert!(decode("wwgj").is_err());
}

#[test]
fn test_neighbor() {
    let ns = neighbors( "e71150dc99").unwrap();
    assert_eq!(ns.sw,   "e71150dc92");
    assert_eq!(ns.s,    "e71150dc98");
    assert_eq!(ns.se,   "e71150dc9a");
    assert_eq!(ns.w,    "e71150dc93");
    assert_eq!(ns.e,    "e71150dc9b");
    assert_eq!(ns.nw,   "e71150dc96");
    assert_eq!(ns.n,    "e71150dc9c");
    assert_eq!(ns.ne,   "e71150dc9e");
}

#[test]
fn test_neighbor_wide() {
    let ns = neighbors("e7115").unwrap();
    assert_eq!(ns.sw, "e5bbe");
    assert_eq!(ns.s, "e7114");
    assert_eq!(ns.se, "e7116");
    assert_eq!(ns.w, "e5bbf");
    assert_eq!(ns.e, "e7117");
    assert_eq!(ns.nw, "e5bea");
    assert_eq!(ns.n, "e7140");
    assert_eq!(ns.ne, "e7142");
}
