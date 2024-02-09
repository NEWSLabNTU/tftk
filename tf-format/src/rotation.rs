use crate::unit::{Angle, Length};
use anyhow::{bail, Result};
use noisy_float::types::R64;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "kebab-case")]
pub enum Rotation {
    Euler(Euler),
    Quaternion(Quaternion),
    AxisAngle(AxisAngle),
    RotationMatrix(RotationMatrix),
    Rodrigues(Rodrigues),
}

impl Rotation {
    pub fn into_degrees(self) -> Self {
        match self {
            Rotation::Euler(rot) => rot.into_degrees().into(),
            Rotation::Quaternion(rot) => rot.into(),
            Rotation::AxisAngle(rot) => rot.into_degrees().into(),
            Rotation::RotationMatrix(rot) => rot.into(),
            Rotation::Rodrigues(rot) => rot.into(),
        }
    }

    pub fn into_radians(self) -> Self {
        match self {
            Rotation::Euler(rot) => rot.into_radians().into(),
            Rotation::Quaternion(rot) => rot.into(),
            Rotation::AxisAngle(rot) => rot.into_radians().into(),
            Rotation::RotationMatrix(rot) => rot.into(),
            Rotation::Rodrigues(rot) => rot.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Euler {
    pub order: EulerAxisOrder,
    pub angles: Vec<Angle>,
}

impl Euler {
    pub fn into_degrees(self) -> Self {
        let Self { order, angles } = self;
        Self {
            order,
            angles: angles.into_iter().map(|ang| ang.to_degrees()).collect(),
        }
    }

    pub fn into_radians(self) -> Self {
        let Self { order, angles } = self;
        Self {
            order,
            angles: angles.into_iter().map(|ang| ang.to_radians()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quaternion {
    pub ijkw: [R64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisAngle {
    pub axis: [Length; 3],
    pub angle: Angle,
}

impl AxisAngle {
    pub fn into_degrees(self) -> Self {
        let Self { axis, angle } = self;
        Self {
            axis,
            angle: angle.to_degrees(),
        }
    }

    pub fn into_radians(self) -> Self {
        let Self { axis, angle } = self;
        Self {
            axis,
            angle: angle.to_radians(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationMatrix {
    pub matrix: [[R64; 3]; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rodrigues {
    pub params: [R64; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EulerAxisOrder(pub Vec<EulerAxis>);

impl ToString for EulerAxisOrder {
    fn to_string(&self) -> String {
        self.0.iter().map(|axis| axis.to_char()).collect()
    }
}

impl FromStr for EulerAxisOrder {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let result: Result<Vec<_>, _> = text.chars().map(EulerAxis::from_char).collect();
        Ok(Self(result?))
    }
}

impl Serialize for EulerAxisOrder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EulerAxisOrder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        text.parse()
            .map_err(|err| D::Error::custom(format!("{err}")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EulerAxis {
    Roll,
    Pitch,
    Yaw,
}

impl EulerAxis {
    pub fn from_char(code: char) -> Result<Self> {
        let axis = match code {
            'r' => Self::Roll,
            'p' => Self::Pitch,
            'y' => Self::Yaw,
            _ => bail!("unexpected axis code '{code}'"),
        };
        Ok(axis)
    }

    pub fn to_char(&self) -> char {
        match self {
            EulerAxis::Roll => 'r',
            EulerAxis::Pitch => 'p',
            EulerAxis::Yaw => 'y',
        }
    }
}
