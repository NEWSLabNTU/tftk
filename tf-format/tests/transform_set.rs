use anyhow::Result;
use approx::assert_abs_diff_eq;
use nalgebra as na;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::Path};
use tf_format::TransformSet;

const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/example_config");

macro_rules! make_iso3 {
    ($rr:expr, $rp:expr, $ry:expr; $tx:expr, $ty:expr, $tz:expr) => {
        na::Isometry3::from_parts(
            na::Translation3::new($tx, $ty, $tz),
            na::UnitQuaternion::from_euler_angles(
                $rr.to_radians(),
                $rp.to_radians(),
                $ry.to_radians(),
            ),
        )
    };
}

#[test]
fn transfrom_set_serde() -> Result<()> {
    let config_dir = Path::new(CONFIG_DIR);
    let set: TransformSet = load_json(config_dir.join("tfset1.json"))?;

    let map_to_car = make_iso3!(0f64, 10f64, 20f64; 100.0, -70.0, 255.0);
    let car_to_lidar1 = make_iso3!(0f64, 0f64, 30f64; 10.0, 0.0, 3.0);
    let car_to_lidar2 = make_iso3!(0f64, 0f64, -30f64; -10.0, 0.0, 3.0);

    let lidar1_to_car = car_to_lidar1.inverse();
    let lidar1_to_lidar2 = lidar1_to_car * car_to_lidar2;

    assert!(set.get("map", "xxx").is_none());
    assert!(set.get("xxx", "map").is_none());
    assert!(set.get("xxx", "yyy").is_none());

    assert_abs_diff_eq!(
        set.get("car", "lidar1").unwrap(),
        car_to_lidar1,
        epsilon = 1e-6
    );
    assert_abs_diff_eq!(
        set.get("car", "lidar2").unwrap(),
        car_to_lidar2,
        epsilon = 1e-6
    );

    assert_abs_diff_eq!(
        set.get("lidar1", "lidar2").unwrap(),
        set.get("lidar2", "lidar1").unwrap().inverse(),
        epsilon = 1e-6
    );

    assert_abs_diff_eq!(
        set.get("lidar1", "lidar2").unwrap(),
        lidar1_to_lidar2,
        epsilon = 1e-6
    );

    assert_abs_diff_eq!(
        set.get("map", "car").unwrap(),
        set.get("car", "map").unwrap().inverse(),
        epsilon = 1e-6
    );
    assert_abs_diff_eq!(set.get("map", "car").unwrap(), map_to_car, epsilon = 1e-6);

    assert_abs_diff_eq!(
        set.get("lidar1", "car").unwrap() * set.get("car", "lidar2").unwrap(),
        set.get("lidar1", "lidar2").unwrap(),
        epsilon = 1e-6
    );

    assert_abs_diff_eq!(
        set.get("lidar1", "car").unwrap() * set.get("car", "lidar1").unwrap(),
        set.get("lidar1", "lidar1").unwrap(),
        epsilon = 1e-6
    );

    assert_abs_diff_eq!(
        set.get("map", "map").unwrap(),
        na::Isometry3::identity(),
        epsilon = 1e-6
    );

    Ok(())
}

#[test]
fn transform_set_consistency_test() -> Result<()> {
    let mut set = TransformSet::default();

    let map_to_car = make_iso3!(0f64, 10f64, 20f64; 100.0, -70.0, 255.0);
    let car_to_lidar1 = make_iso3!(0f64, 0f64, 30f64; 10.0, 0.0, 3.0);
    let car_to_lidar2 = make_iso3!(0f64, 0f64, -30f64; -10.0, 0.0, 3.0);

    let car_to_map = map_to_car.inverse();
    let lidar1_to_car = car_to_lidar1.inverse();
    let lidar2_to_car = car_to_lidar2.inverse();
    let lidar1_to_lidar2 = lidar1_to_car * car_to_lidar2;
    let lidar2_to_lidar1 = lidar1_to_lidar2.inverse();
    let map_to_lidar1 = map_to_car * car_to_lidar1;
    let map_to_lidar2 = map_to_car * car_to_lidar2;
    let lidar1_to_map = map_to_lidar1.inverse();
    let lidar2_to_map = map_to_lidar2.inverse();

    set.insert("car", "lidar1", car_to_lidar1)?;
    set.insert("car", "map", car_to_map)?;
    set.insert("car", "lidar2", car_to_lidar2)?;
    set.insert("lidar1", "car", lidar1_to_car)?;
    set.insert("map", "car", map_to_car)?;
    set.insert("lidar2", "car", lidar2_to_car)?;

    macro_rules! check {
        ($src:expr, $dst:expr, $expect:expr) => {
            assert_abs_diff_eq!(set.get($src, $dst).unwrap(), $expect, epsilon = 1e-6);
        };
    }

    check!("car", "lidar1", car_to_lidar1);
    check!("car", "lidar2", car_to_lidar2);
    check!("car", "map", car_to_map);
    check!("map", "car", map_to_car);
    check!("map", "lidar1", map_to_lidar1);
    check!("map", "lidar2", map_to_lidar2);
    check!("lidar1", "car", lidar1_to_car);
    check!("lidar1", "lidar2", lidar1_to_lidar2);
    check!("lidar1", "map", lidar1_to_map);
    check!("lidar2", "car", lidar2_to_car);
    check!("lidar2", "lidar1", lidar2_to_lidar1);
    check!("lidar2", "map", lidar2_to_map);

    Ok(())
}

fn load_json<T, P>(path: P) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}
