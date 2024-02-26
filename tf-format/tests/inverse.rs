use anyhow::Result;
use approx::assert_abs_diff_eq;
use nalgebra as na;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::Path};
use tf_format::{
    AxisAngle, Euler, EulerAxis, EulerAxisOrder, Quaternion, Rotation, RotationMatrix, Transform,
    Translation,
};

const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/example_config");

const EPSILON: f64 = 1e-7;

macro_rules! assert_abs_diff_eq_list {
    ($tested:expr, $target:expr) => {
        $tested
            .into_iter()
            .zip($target.into_iter())
            .for_each(|(lhs, rhs)| {
                assert_abs_diff_eq!(lhs, rhs, epsilon = EPSILON);
            });
    };
}

#[test]
fn inverse_rotation() -> Result<()> {
    let config_dir = Path::new(CONFIG_DIR);

    // euler
    {
        let rot: Rotation = load_json(config_dir.join("rot_euler.json"))?;
        let rot = rot.inverse().inverse();
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
        assert_abs_diff_eq_list!(
            [r, p, y].iter().map(|v| v.as_degrees_value().raw()),
            [10.0, -5.0, 3.0]
        );
    }

    // axis-angle
    {
        let rot: Rotation = load_json(config_dir.join("rot_axis_angle.json"))?;
        let rot = rot.inverse().inverse();
        let Rotation::AxisAngle(axis_angle) = rot else {
            panic!();
        };

        let AxisAngle { axis, angle } = axis_angle;

        assert_abs_diff_eq_list!(axis.iter().map(|v| v.raw()), [0.6, -0.8, 0.0]);
        assert_abs_diff_eq!(angle.as_degrees_value().raw(), 45.0, epsilon = 1e-7);
    }

    // quaternion
    {
        let rot: Rotation = load_json(config_dir.join("rot_quaternion.json"))?;
        let rot = rot.inverse().inverse();
        let Rotation::Quaternion(quat) = rot else {
            panic!();
        };
        let Quaternion { ijkw } = quat;
        assert_abs_diff_eq_list!(ijkw.iter().map(|v| v.raw()), [0.0, 0.0, 0.0, 1.0]);
    }

    // matrix
    {
        let rot: Rotation = load_json(config_dir.join("rot_matrix.json"))?;
        let rot = rot.inverse().inverse();
        let Rotation::RotationMatrix(rot_mat) = rot else {
            panic!();
        };
        let RotationMatrix { matrix } = rot_mat;

        assert_abs_diff_eq_list!(
            matrix.into_iter().flatten().map(|v| v.raw()),
            [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0],]
                .into_iter()
                .flatten()
        );
    }

    // transform w/ euler
    {
        let tf: Transform = load_json(config_dir.join("tf_euler.json"))?;
        let orig_quat: na::UnitQuaternion<f64> = tf.r.clone().into();

        let tf = tf.inverse().inverse();
        let Transform {
            t: Translation(trans),
            r: Rotation::Euler(euler),
        } = tf
        else {
            panic!();
        };

        let new_quat: na::UnitQuaternion<f64> = euler.clone().into();
        let Euler {
            order: EulerAxisOrder(order),
            ..
        } = euler;

        assert_abs_diff_eq_list!(trans.iter().map(|v| v.raw()), [1.0, -2.0, 0.3]);
        assert_eq!(
            order,
            vec![EulerAxis::Roll, EulerAxis::Pitch, EulerAxis::Yaw]
        );
        assert_abs_diff_eq!(orig_quat, new_quat, epsilon = EPSILON);
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
