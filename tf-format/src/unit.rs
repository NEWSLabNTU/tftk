use anyhow::bail;
use approx::AbsDiffEq;
use noisy_float::types::{r64, R64};
use num::{Float, Zero};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{cmp::Ordering, f64::consts::PI, str::FromStr};

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
    pub fn normalize(&self) -> Self {
        let Self { unit, value } = *self;

        let value = match unit {
            AngleUnit::Radian => value.raw().rem_euclid(PI * 2.0),
            AngleUnit::Degree => value.raw().rem_euclid(360.0),
        };

        Self {
            unit,
            value: r64(value),
        }
    }

    pub fn zero() -> Self {
        Self {
            unit: AngleUnit::Radian,
            value: R64::zero(),
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

impl AbsDiffEq for Angle {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon {
        f64::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.as_radians_value()
            .raw()
            .abs_diff_eq(&other.as_radians_value().raw(), epsilon)
    }
}

fn angle_eq(lhs: &Angle, rhs: &Angle) -> bool {
    lhs.as_radians_value().eq(&rhs.as_radians_value())
}

fn angle_ord(lhs: &Angle, rhs: &Angle) -> Ordering {
    lhs.as_radians_value().cmp(&rhs.as_radians_value())
}

#[cfg(test)]
mod tests {
    use super::{Angle, AngleUnit};
    use noisy_float::types::r64;
    use std::f64::consts::FRAC_PI_2;

    #[test]
    fn parse_angle() {
        {
            let angle: Angle = "74.3d".parse().unwrap();
            assert_eq!(
                angle,
                Angle {
                    unit: AngleUnit::Degree,
                    value: r64(74.3)
                }
            );
        }

        {
            let angle: Angle = "-47.2deg".parse().unwrap();
            assert_eq!(
                angle,
                Angle {
                    unit: AngleUnit::Degree,
                    value: r64(-47.2)
                }
            );
        }

        {
            let angle: Angle = "97.0r".parse().unwrap();
            assert_eq!(
                angle,
                Angle {
                    unit: AngleUnit::Radian,
                    value: r64(97.0)
                }
            );
        }

        {
            let angle: Angle = "-61.4rad".parse().unwrap();
            assert_eq!(
                angle,
                Angle {
                    unit: AngleUnit::Radian,
                    value: r64(-61.4)
                }
            );
        }
    }

    #[test]
    fn angle_unit() {
        let angle = Angle {
            unit: AngleUnit::Degree,
            value: r64(90.0),
        };

        angle.to_degrees();
        assert_eq!(
            angle,
            Angle {
                unit: AngleUnit::Degree,
                value: r64(90.0)
            }
        );

        angle.to_radians();
        assert_eq!(
            angle,
            Angle {
                unit: AngleUnit::Radian,
                value: r64(FRAC_PI_2)
            }
        );

        angle.to_radians();
        assert_eq!(
            angle,
            Angle {
                unit: AngleUnit::Radian,
                value: r64(FRAC_PI_2)
            }
        );

        angle.to_degrees();
        assert_eq!(
            angle,
            Angle {
                unit: AngleUnit::Degree,
                value: r64(90.0)
            }
        );
    }
}
