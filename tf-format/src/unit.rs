use anyhow::bail;
use noisy_float::types::{r64, R64};
use num::Float;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{cmp::Ordering, str::FromStr};

pub type Length = R64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AngleUnit {
    Radian,
    Degree,
}

#[derive(Debug, Clone, Copy)]
pub struct Angle {
    pub unit: AngleUnit,
    pub value: R64,
}

impl Angle {
    pub fn zero() -> Self {
        Self {
            unit: AngleUnit::Radian,
            value: r64(0.0),
        }
    }

    pub fn to_radians(&self) -> Self {
        let Self { unit, value } = *self;

        let new_value = match unit {
            AngleUnit::Radian => value,
            AngleUnit::Degree => value.to_radians(),
        };

        Self {
            unit: AngleUnit::Radian,
            value: new_value,
        }
    }

    pub fn to_degrees(&self) -> Self {
        let Self { unit, value } = *self;

        let new_value = match unit {
            AngleUnit::Radian => value.to_degrees(),
            AngleUnit::Degree => value,
        };

        Self {
            unit: AngleUnit::Degree,
            value: new_value,
        }
    }

    pub fn as_radians_value(&self) -> R64 {
        let Self { unit, value } = *self;

        match unit {
            AngleUnit::Radian => value,
            AngleUnit::Degree => value.to_radians(),
        }
    }

    pub fn as_degrees_value(&self) -> R64 {
        let Self { unit, value } = *self;

        match unit {
            AngleUnit::Radian => value.to_degrees(),
            AngleUnit::Degree => value,
        }
    }

    pub fn from_radians(value: R64) -> Self {
        Self {
            unit: AngleUnit::Radian,
            value,
        }
    }

    pub fn from_degrees(value: R64) -> Self {
        Self {
            unit: AngleUnit::Degree,
            value,
        }
    }
}

impl FromStr for Angle {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (unit, value) = if let Some(prefix) = text.strip_suffix('°') {
            let value: f64 = prefix.parse()?;
            (AngleUnit::Degree, value)
        } else if let Some(prefix) = text.strip_suffix("rad") {
            let value: f64 = prefix.parse()?;
            (AngleUnit::Radian, value)
        } else if let Some(prefix) = text.strip_suffix("deg") {
            let value: f64 = prefix.parse()?;
            (AngleUnit::Degree, value)
        } else if let Some(prefix) = text.strip_suffix('d') {
            let value: f64 = prefix.parse()?;
            (AngleUnit::Degree, value)
        } else if let Some(prefix) = text.strip_suffix('r') {
            let value: f64 = prefix.parse()?;
            (AngleUnit::Radian, value)
        } else {
            bail!("unable to parse angle value '{text}'");
        };

        let Ok(value) = R64::try_from(value) else {
            bail!("invalid angle value '{value}'");
        };

        Ok(Self { unit, value })
    }
}

impl ToString for Angle {
    fn to_string(&self) -> String {
        let Self { unit, value } = *self;

        match unit {
            AngleUnit::Radian => format!("{value}r"),
            AngleUnit::Degree => format!("{value}d"),
        }
    }
}

impl Serialize for Angle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Angle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        text.parse()
            .map_err(|err| D::Error::custom(format!("{err}")))
    }
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        angle_eq(self, other)
    }
}

impl Eq for Angle {}

impl PartialOrd for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Angle {
    fn cmp(&self, other: &Self) -> Ordering {
        angle_ord(self, other)
    }
}

fn angle_eq(lhs: &Angle, rhs: &Angle) -> bool {
    lhs.as_radians_value().eq(&rhs.as_radians_value())
}

fn angle_ord(lhs: &Angle, rhs: &Angle) -> Ordering {
    lhs.as_radians_value().cmp(&rhs.as_radians_value())
}
