use crate::{
    rotation::{Quaternion, Rodrigues, Rotation},
    Angle, AxisAngle, Euler, EulerAxis, EulerAxisOrder, RotationMatrix, Transform, Translation,
};
use nalgebra as na;
use noisy_float::types::{r64, R64};
use num::{NumCast, ToPrimitive, Zero};

impl From<RotationMatrix> for Rotation {
    fn from(v: RotationMatrix) -> Self {
        Self::RotationMatrix(v)
    }
}

impl From<AxisAngle> for Rotation {
    fn from(v: AxisAngle) -> Self {
        Self::AxisAngle(v)
    }
}

impl From<Quaternion> for Rotation {
    fn from(v: Quaternion) -> Self {
        Self::Quaternion(v)
    }
}

impl From<Euler> for Rotation {
    fn from(v: Euler) -> Self {
        Self::Euler(v)
    }
}

impl From<Rodrigues> for Rotation {
    fn from(v: Rodrigues) -> Self {
        Self::Rodrigues(v)
    }
}

impl From<Rotation> for Euler {
    fn from(rot: Rotation) -> Self {
        let quat: na::UnitQuaternion<f64> = rot.into();
        quat.into()
    }
}

impl From<Rotation> for AxisAngle {
    fn from(rot: Rotation) -> Self {
        let quat: na::UnitQuaternion<f64> = rot.into();
        quat.into()
    }
}

impl From<Rotation> for RotationMatrix {
    fn from(rot: Rotation) -> Self {
        let quat: na::UnitQuaternion<f64> = rot.into();
        quat.into()
    }
}

impl From<Rotation> for Quaternion {
    fn from(rot: Rotation) -> Self {
        let quat: na::UnitQuaternion<f64> = rot.into();
        quat.into()
    }
}

impl From<Rotation> for Rodrigues {
    fn from(rot: Rotation) -> Self {
        let quat: na::UnitQuaternion<f64> = rot.into();
        quat.into()
    }
}

impl<T> From<Transform> for na::Isometry3<T>
where
    T: na::RealField + NumCast,
{
    fn from(tf: Transform) -> Self {
        macro_rules! cast {
            ($val:expr) => {
                <T as NumCast>::from($val).unwrap()
            };
        }

        let Transform {
            r: rot,
            t: Translation([x, y, z]),
        } = tf;

        let trans = na::Translation3::new(cast!(x), cast!(y), cast!(z));
        let rot: na::UnitQuaternion<T> = rot.into();
        Self::from_parts(trans, rot)
    }
}

impl<T> From<na::Isometry3<T>> for Transform
where
    T: na::RealField + NumCast,
{
    fn from(iso: na::Isometry3<T>) -> Self {
        use nalgebra::base::coordinates::XYZ;

        macro_rules! cast {
            ($val:expr) => {
                <R64 as NumCast>::from($val.clone()).unwrap()
            };
        }

        let na::Isometry3 {
            translation,
            rotation,
        } = iso;
        let XYZ { x, y, z } = &*translation.vector;

        Self {
            r: rotation.into(),
            t: Translation([cast!(x), cast!(y), cast!(z)]),
        }
    }
}

impl<T> From<Rotation> for na::UnitQuaternion<T>
where
    T: na::RealField + NumCast,
{
    fn from(from: Rotation) -> Self {
        match from {
            Rotation::Euler(rot) => rot.into(),
            Rotation::Quaternion(rot) => rot.into(),
            Rotation::AxisAngle(rot) => rot.into(),
            Rotation::RotationMatrix(rot) => rot.into(),
            Rotation::Rodrigues(rot) => rot.into(),
        }
    }
}

impl<T> From<na::UnitQuaternion<T>> for Rotation
where
    T: na::RealField + NumCast,
{
    fn from(quat: na::UnitQuaternion<T>) -> Self {
        Self::Quaternion(quat.into())
    }
}

impl<T> From<Euler> for na::UnitQuaternion<T>
where
    T: na::RealField + NumCast,
{
    fn from(euler: Euler) -> Self {
        let Euler {
            order: EulerAxisOrder(order),
            angles,
        } = euler;
        assert_eq!(order.len(), angles.len());

        order
            .into_iter()
            .zip(angles)
            .map(|(axis, angle)| {
                let angle = T::from(angle.as_radians_value()).unwrap();

                match axis {
                    EulerAxis::Roll => {
                        na::UnitQuaternion::from_euler_angles(angle, T::zero(), T::zero())
                    }
                    EulerAxis::Pitch => {
                        na::UnitQuaternion::from_euler_angles(T::zero(), angle, T::zero())
                    }
                    EulerAxis::Yaw => {
                        na::UnitQuaternion::from_euler_angles(T::zero(), T::zero(), angle)
                    }
                }
            })
            .reduce(|lhs, rhs| rhs * lhs)
            .unwrap_or_else(|| Self::identity())
    }
}

impl<T> From<na::UnitQuaternion<T>> for Euler
where
    T: na::RealField + ToPrimitive,
{
    fn from(quat: na::UnitQuaternion<T>) -> Self {
        macro_rules! cast {
            ($val:expr) => {
                Angle::from_radians(<R64 as NumCast>::from($val).unwrap())
            };
        }

        let (r, p, y) = quat.euler_angles();

        Self {
            order: EulerAxisOrder(vec![EulerAxis::Roll, EulerAxis::Pitch, EulerAxis::Yaw]),
            angles: vec![cast!(r), cast!(p), cast!(y)],
        }
    }
}

impl<T> From<Quaternion> for na::UnitQuaternion<T>
where
    T: na::RealField + NumCast,
{
    fn from(quat: Quaternion) -> Self {
        let Quaternion { ijkw: [i, j, k, w] } = quat;

        macro_rules! cast {
            ($ang:expr) => {
                T::from($ang).unwrap()
            };
        }

        na::Unit::new_normalize(na::Quaternion::new(cast!(w), cast!(i), cast!(j), cast!(k)))
    }
}

impl<T> From<na::UnitQuaternion<T>> for Quaternion
where
    T: na::RealField + ToPrimitive + Clone,
{
    fn from(quat: na::UnitQuaternion<T>) -> Self {
        use nalgebra::base::coordinates::IJKW;

        macro_rules! cast {
            ($val:expr) => {
                <R64 as NumCast>::from($val.clone()).unwrap()
            };
        }

        let IJKW { i, j, k, w } = &**quat;

        Self {
            ijkw: [cast!(i), cast!(j), cast!(k), cast!(w)],
        }
    }
}

impl<T> From<AxisAngle> for na::UnitQuaternion<T>
where
    T: na::RealField + NumCast,
{
    fn from(axis_angle: AxisAngle) -> Self {
        macro_rules! cast {
            ($val:expr) => {
                T::from($val).unwrap()
            };
        }

        let AxisAngle {
            axis: [x, y, z],
            angle,
        } = axis_angle;

        let axis = na::Unit::new_normalize(na::Vector3::new(cast!(x), cast!(y), cast!(z)));
        let radians = T::from(angle.as_radians_value()).unwrap();

        Self::from_axis_angle(&axis, radians)
    }
}

impl<T> From<na::UnitQuaternion<T>> for AxisAngle
where
    T: na::RealField + NumCast + Clone,
{
    fn from(quat: na::UnitQuaternion<T>) -> Self {
        use nalgebra::base::coordinates::XYZ;

        macro_rules! cast {
            ($val:expr) => {
                <R64 as NumCast>::from($val.clone()).unwrap()
            };
        }

        let Some((axis, angle)) = quat.axis_angle() else {
            return Self {
                axis: [r64(1.0), r64(0.0), r64(0.0)],
                angle: Angle::zero(),
            };
        };

        let XYZ { x, y, z } = &**axis;

        Self {
            axis: [cast!(x), cast!(y), cast!(z)],
            angle: Angle::from_radians(cast!(angle)),
        }
    }
}

impl<T> From<RotationMatrix> for na::UnitQuaternion<T>
where
    T: na::RealField + NumCast,
{
    fn from(mat: RotationMatrix) -> Self {
        let elems = mat
            .matrix
            .into_iter()
            .flatten()
            .map(|elem| T::from(elem).unwrap());
        let mat = na::Matrix3::from_row_iterator(elems);
        Self::from_matrix(&mat)
    }
}

impl<T> From<na::UnitQuaternion<T>> for RotationMatrix
where
    T: na::RealField + NumCast + Clone,
{
    fn from(quat: na::UnitQuaternion<T>) -> Self {
        use nalgebra::base::coordinates::M3x3;

        macro_rules! cast {
            ($val:expr) => {
                <R64 as NumCast>::from($val.clone()).unwrap()
            };
        }

        let rot_mat = quat.to_rotation_matrix();
        let mat = rot_mat.matrix();
        let M3x3 {
            m11,
            m21,
            m31,
            m12,
            m22,
            m32,
            m13,
            m23,
            m33,
        } = &**mat;

        Self {
            matrix: [
                [cast!(m11), cast!(m12), cast!(m13)],
                [cast!(m21), cast!(m22), cast!(m23)],
                [cast!(m31), cast!(m32), cast!(m33)],
            ],
        }
    }
}

impl<T> From<Rodrigues> for na::UnitQuaternion<T>
where
    T: na::RealField + NumCast,
{
    fn from(rod: Rodrigues) -> Self {
        let [r1, r2, r3] = rod.params;
        let axis = na::Vector3::new(r1.raw(), r2.raw(), r3.raw());
        let Some((axis, angle)) = na::Unit::try_new_and_get(axis, 1e-8) else {
            return na::UnitQuaternion::identity();
        };
        na::UnitQuaternion::from_axis_angle(&axis, angle).cast()
    }
}

impl<T> From<na::UnitQuaternion<T>> for Rodrigues
where
    T: na::RealField + NumCast,
{
    fn from(quat: na::UnitQuaternion<T>) -> Self {
        macro_rules! cast {
            ($val:expr) => {
                <R64 as NumCast>::from($val).unwrap()
            };
        }

        let Some((axis, angle)) = quat.axis_angle() else {
            let z = R64::zero();
            return Rodrigues { params: [z, z, z] };
        };
        let params = axis.into_inner() * angle;
        let [r1, r2, r3] = params.into();
        Rodrigues {
            params: [cast!(r1), cast!(r2), cast!(r3)],
        }
    }
}
