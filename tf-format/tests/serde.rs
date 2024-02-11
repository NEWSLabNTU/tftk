use anyhow::Result;
use approx::assert_abs_diff_eq;
use nalgebra as na;
use noisy_float::types::r64;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::Path};
use tf_format::{
    AxisAngle, Euler, EulerAxis, EulerAxisOrder, Quaternion, Rotation, RotationMatrix, Transform,
    TransformSet, Translation,
};

const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/example_config");

#[test]
fn json_parsing() -> Result<()> {
    let config_dir = Path::new(CONFIG_DIR);

    // euler
    {
        let rot: Rotation = load_json(config_dir.join("rot_euler.json"))?;
        let Rotation::Euler(euler) = rot else {
            panic!();
        };

        let Euler {
            order: EulerAxisOrder(order),
            angles,
        } = euler;

        let [r, p, y] = *angles else { panic!() };

        assert_eq!(
            order,
            vec![EulerAxis::Roll, EulerAxis::Pitch, EulerAxis::Yaw]
        );
        assert_eq!(
            [
                r.as_degrees_value(),
                p.as_degrees_value(),
                y.as_degrees_value(),
            ],
            [r64(10.0), r64(-5.0), r64(3.0)]
        );
    }

    // axis-angle
    {
        let rot: Rotation = load_json(config_dir.join("rot_axis_angle.json"))?;
        let Rotation::AxisAngle(axis_angle) = rot else {
            panic!();
        };

        let AxisAngle { axis, angle } = axis_angle;

        assert_eq!(axis, [r64(0.6), r64(-0.8), r64(0.0)]);
        assert_eq!(angle.as_degrees_value(), 45.0);
    }

    // quaternion
    {
        let rot: Rotation = load_json(config_dir.join("rot_quaternion.json"))?;
        let Rotation::Quaternion(quat) = rot else {
            panic!();
        };
        let Quaternion { ijkw } = quat;

        assert_eq!(ijkw, [r64(0.0), r64(0.0), r64(0.0), r64(1.0)]);
    }

    // matrix
    {
        let rot: Rotation = load_json(config_dir.join("rot_matrix.json"))?;
        let Rotation::RotationMatrix(rot_mat) = rot else {
            panic!();
        };
        let RotationMatrix { matrix } = rot_mat;

        assert_eq!(
            matrix,
            [
                [r64(0.0), r64(1.0), r64(0.0)],
                [r64(0.0), r64(0.0), r64(1.0)],
                [r64(1.0), r64(0.0), r64(0.0)],
            ]
        );
    }

    // transform w/ euler
    {
        let tf: Transform = load_json(config_dir.join("tf_euler.json"))?;
        let Transform {
            t: Translation(trans),
            r: Rotation::Euler(euler),
        } = tf
        else {
            panic!();
        };

        let Euler {
            order: EulerAxisOrder(order),
            angles,
        } = euler;

        let [y, p, r] = *angles else { panic!() };

        assert_eq!(trans, [1.0, -2.0, 0.3]);
        assert_eq!(
            order,
            vec![EulerAxis::Yaw, EulerAxis::Pitch, EulerAxis::Roll]
        );
        assert_eq!(
            [
                y.as_radians_value(),
                p.as_degrees_value(),
                r.as_radians_value(),
            ],
            [r64(-3.14), r64(27.0), r64(-30.0)]
        );
    }

    Ok(())
}

#[test]
fn nalgebra_conversion() -> Result<()> {
    let config_dir = Path::new(CONFIG_DIR);

    for entry in config_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        let Some(ext) = path.extension() else {
            continue;
        };
        if ext != "json" {
            continue;
        }

        let Some(stem) = path.file_stem() else {
            continue;
        };
        let Some(stem) = stem.to_str() else {
            continue;
        };

        if stem.starts_with("rot_") {
            let rot: Rotation = load_json(path)?;
            let quat: na::UnitQuaternion<f64> = rot.into();
            let rot2: Rotation = quat.into();
            let quat2: na::UnitQuaternion<f64> = rot2.into();
            assert_abs_diff_eq!(quat, quat2, epsilon = 1e-6);
        } else if stem.starts_with("tf_") {
            let tf: Transform = load_json(path)?;
            let iso: na::Isometry3<f64> = tf.into();
            let tf2: Transform = iso.into();
            let iso2: na::Isometry3<f64> = tf2.into();
            assert_abs_diff_eq!(iso, iso2, epsilon = 1e-6);
        }
    }

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
