use crate::{AxisAngle, Euler, Quaternion, Rodrigues, Rotation, RotationMatrix};
use nalgebra as na;
use noisy_float::types::R64;
use num::NumCast;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub r: Rotation,
    pub t: Translation,
}

impl Transform {
    pub fn normalize_rotation(&self) -> Self {
        let Self { r, t } = self;
        Self {
            r: r.normalize(),
            t: *t,
        }
    }

    pub fn inverse(&self) -> Self {
        let iso: na::Isometry3<f64> = self.clone().into();
        let na::Isometry3 {
            rotation: rot,
            translation: trans,
        } = iso.inverse();

        let rot: Rotation = match self.r {
            Rotation::Euler(_) => Euler::from(rot).into(),
            Rotation::Quaternion(_) => Quaternion::from(rot).into(),
            Rotation::AxisAngle(_) => AxisAngle::from(rot).into(),
            Rotation::RotationMatrix(_) => RotationMatrix::from(rot).into(),
            Rotation::Rodrigues(_) => Rodrigues::from(rot).into(),
        };

        Self {
            r: rot,
            t: trans.into(),
        }
    }

    pub fn into_degrees(self) -> Self {
        let Self { t, r } = self;
        Self {
            t,
            r: r.into_degrees(),
        }
    }

    pub fn into_radians(self) -> Self {
        let Self { t, r } = self;
        Self {
            t,
            r: r.into_radians(),
        }
    }

    pub fn into_euler_format(self) -> Self {
        let Self { t, r } = self;
        Self {
            t,
            r: r.into_euler_format(),
        }
    }

    pub fn into_axis_angle_format(self) -> Self {
        let Self { t, r } = self;
        Self {
            t,
            r: r.into_axis_angle_format(),
        }
    }

    pub fn into_quaternion_format(self) -> Self {
        let Self { t, r } = self;
        Self {
            t,
            r: r.into_quaternion_format(),
        }
    }

    pub fn into_rodrigues_format(self) -> Self {
        let Self { t, r } = self;
        Self {
            t,
            r: r.into_rodrigues_format(),
        }
    }

    pub fn into_rotation_matrix_format(self) -> Self {
        let Self { t, r } = self;
        Self {
            t,
            r: r.into_rotation_matrix_format(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "SerializedTransform", into = "SerializedTransform")]
pub struct MaybeTransform {
    pub r: Rotation,
    pub t: Option<Translation>,
}

impl From<Transform> for MaybeTransform {
    fn from(tf: Transform) -> Self {
        let Transform { t, r } = tf;
        Self { r, t: Some(t) }
    }
}

impl From<Rotation> for MaybeTransform {
    fn from(rot: Rotation) -> Self {
        Self { r: rot, t: None }
    }
}

impl TryFrom<MaybeTransform> for Transform {
    type Error = MaybeTransform;

    fn try_from(tf: MaybeTransform) -> Result<Self, Self::Error> {
        let MaybeTransform { r, t } = tf;
        let Some(t) = t else {
            return Err(MaybeTransform { t, r });
        };
        Ok(Self { t, r })
    }
}

impl TryFrom<MaybeTransform> for Rotation {
    type Error = MaybeTransform;

    fn try_from(tf: MaybeTransform) -> Result<Self, Self::Error> {
        let MaybeTransform { r, t } = tf;
        if t.is_some() {
            return Err(MaybeTransform { t, r });
        }
        Ok(r)
    }
}

impl MaybeTransform {
    pub fn to_na_isometry3<T>(&self) -> na::Isometry3<T>
    where
        T: na::RealField + NumCast,
    {
        let Self { r, t } = self;
        let rot: na::UnitQuaternion<T> = r.clone().into();
        let trans: na::Translation3<T> = match t {
            Some(t) => (*t).into(),
            None => na::Translation3::identity(),
        };
        na::Isometry3::from_parts(trans, rot)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum SerializedTransform {
    Transform(Transform),
    Rotation(Rotation),
}

impl From<SerializedTransform> for MaybeTransform {
    fn from(from: SerializedTransform) -> Self {
        let (r, t) = match from {
            SerializedTransform::Transform(Transform { t, r }) => (r, Some(t)),
            SerializedTransform::Rotation(r) => (r, None),
        };

        Self { t, r }
    }
}

impl From<MaybeTransform> for SerializedTransform {
    fn from(from: MaybeTransform) -> Self {
        let MaybeTransform { r, t } = from;

        match t {
            Some(t) => Self::Transform(Transform { r, t }),
            None => Self::Rotation(r),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Translation(pub [R64; 3]);

impl<T> From<Translation> for na::Translation3<T>
where
    T: na::RealField + NumCast,
{
    fn from(trans: Translation) -> Self {
        macro_rules! cast {
            ($val:expr) => {
                <T as NumCast>::from($val).unwrap()
            };
        }

        let Translation([x, y, z]) = trans;
        na::Translation3::new(cast!(x), cast!(y), cast!(z)).cast()
    }
}

impl<T> From<na::Translation3<T>> for Translation
where
    T: na::RealField + NumCast,
{
    fn from(trans: na::Translation3<T>) -> Self {
        macro_rules! cast {
            ($val:expr) => {
                <R64 as NumCast>::from($val).unwrap()
            };
        }

        let [x, y, z] = trans.vector.into();
        Self([cast!(x), cast!(y), cast!(z)])
    }
}

#[cfg(test)]
mod tests {
    use super::{Transform, Translation};
    use crate::{unit::AngleUnit, Angle, Euler, EulerAxis, EulerAxisOrder, Rotation};
    use approx::assert_abs_diff_eq;
    use noisy_float::types::r64;

    #[test]
    fn transform_convert() {
        let trans = Transform {
            r: Euler {
                order: EulerAxisOrder(vec![EulerAxis::Roll]),
                angles: vec![Angle {
                    unit: AngleUnit::Degree,
                    value: r64(10.0),
                }],
            }
            .into(),
            t: Translation([r64(-10.0), r64(20.0), r64(30.0)]),
        };

        let trans = trans.into_radians().into_degrees();
        let trans = trans.into_axis_angle_format();

        let trans = trans.into_radians().into_degrees();
        let trans = trans.into_quaternion_format();

        let trans = trans.into_radians().into_degrees();
        let trans = trans.into_rotation_matrix_format();

        let trans = trans.into_radians().into_degrees();
        let trans = trans.into_rodrigues_format();

        let trans = trans.into_radians().into_degrees();
        let trans = trans.into_euler_format();

        let trans = trans.into_radians().into_degrees();

        let Transform {
            r: rot,
            t: Translation([x, y, z]),
        } = trans;

        assert_abs_diff_eq!(x.raw(), -10.0, epsilon = 1e-6);
        assert_abs_diff_eq!(y.raw(), 20.0, epsilon = 1e-6);
        assert_abs_diff_eq!(z.raw(), 30.0, epsilon = 1e-6);

        let Rotation::Euler(Euler {
            order: EulerAxisOrder(order),
            angles,
        }) = rot
        else {
            panic!("expect Euler variant");
        };

        assert_eq!(
            order,
            vec![EulerAxis::Roll, EulerAxis::Pitch, EulerAxis::Yaw,]
        );

        let [roll, pitch, yaw] = *angles else {
            panic!("expect three angles");
        };

        assert_abs_diff_eq!(
            roll,
            Angle {
                unit: AngleUnit::Degree,
                value: r64(10.0)
            },
            epsilon = 1e-5
        );
        assert_abs_diff_eq!(pitch, Angle::zero(), epsilon = 1e-5);
        assert_abs_diff_eq!(yaw, Angle::zero(), epsilon = 1e-5);
    }
}
