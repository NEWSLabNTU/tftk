use crate::Rotation;
use noisy_float::types::R64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub r: Rotation,
    pub t: Translation,
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
