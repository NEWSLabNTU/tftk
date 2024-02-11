use crate::Rotation;
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
            t: t.clone(),
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
