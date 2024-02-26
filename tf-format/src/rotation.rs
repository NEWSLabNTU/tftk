use crate::unit::{Angle, Length};
use anyhow::{bail, Result};
use nalgebra as na;
use noisy_float::types::{r64, R64};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{f64::consts::PI, str::FromStr};

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
    pub fn normalize(&self) -> Self {
        match self {
            Rotation::Euler(rot) => rot.normalize().into(),
            Rotation::Quaternion(rot) => rot.clone().into(),
            Rotation::AxisAngle(rot) => rot.normalize().into(),
            Rotation::RotationMatrix(rot) => rot.clone().into(),
            Rotation::Rodrigues(rot) => rot.normalize().into(),
        }
    }

    pub fn inverse(&self) -> Self {
        match self {
            Rotation::Euler(rot) => rot.inverse().into(),
            Rotation::Quaternion(rot) => rot.inverse().into(),
            Rotation::AxisAngle(rot) => rot.inverse().into(),
            Rotation::RotationMatrix(rot) => rot.inverse().into(),
            Rotation::Rodrigues(rot) => rot.inverse().into(),
        }
    }

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

    pub fn into_euler_format(self) -> Self {
        Euler::from(self).into()
    }

    pub fn into_axis_angle_format(self) -> Self {
        AxisAngle::from(self).into()
    }

    pub fn into_quaternion_format(self) -> Self {
        Quaternion::from(self).into()
    }

    pub fn into_rodrigues_format(self) -> Self {
        Rodrigues::from(self).into()
    }

    pub fn into_rotation_matrix_format(self) -> Self {
        RotationMatrix::from(self).into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Euler {
    pub order: EulerAxisOrder,
    pub angles: Vec<Angle>,
}

impl Euler {
    pub fn normalize(&self) -> Self {
        let quat: na::UnitQuaternion<f64> = self.clone().into();
        let Euler { order, angles } = quat.into();
        let angles: Vec<_> = angles.into_iter().map(|ang| ang.normalize()).collect();
        Self { order, angles }
    }

    pub fn inverse(&self) -> Self {
        let quat: na::UnitQuaternion<f64> = self.clone().into();
        quat.inverse().into()
    }

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

impl Quaternion {
    pub fn inverse(&self) -> Self {
        let quat: na::UnitQuaternion<f64> = self.clone().into();
        quat.inverse().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisAngle {
    pub axis: [Length; 3],
    pub angle: Angle,
}

impl AxisAngle {
    pub fn normalize(&self) -> Self {
        let Self { axis, angle } = *self;
        Self {
            axis,
            angle: angle.normalize(),
        }
    }

    pub fn inverse(&self) -> Self {
        let quat: na::UnitQuaternion<f64> = self.clone().into();
        quat.inverse().into()
    }

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

impl RotationMatrix {
    pub fn inverse(&self) -> Self {
        let quat: na::UnitQuaternion<f64> = self.clone().into();
        quat.inverse().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rodrigues {
    pub params: [R64; 3],
}

impl Rodrigues {
    pub fn normalize(&self) -> Self {
        let [r1, r2, r3] = self.params;
        let vec = na::Vector3::new(r1.raw(), r2.raw(), r3.raw());

        let orig_angle = vec.norm();
        let new_angle = orig_angle.rem_euclid(PI * 2.0);

        let vec = vec / orig_angle * new_angle;
        let [r1, r2, r3] = vec.into();
        Self {
            params: [r64(r1), r64(r2), r64(r3)],
        }
    }

    pub fn inverse(&self) -> Self {
        let quat: na::UnitQuaternion<f64> = self.clone().into();
        quat.inverse().into()
    }
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

#[cfg(test)]
mod tests {
    use super::{Angle, Euler, EulerAxis, EulerAxisOrder, Rotation};
    use crate::unit::AngleUnit;
    use approx::assert_abs_diff_eq;
    use noisy_float::types::r64;

    #[test]
    fn rotation_convert() {
        let rot: Rotation = Euler {
            order: EulerAxisOrder(vec![EulerAxis::Roll]),
            angles: vec![Angle {
                unit: AngleUnit::Degree,
                value: r64(10.0),
            }],
        }
        .into();

        let rot = rot.into_radians().into_degrees();
        let rot = rot.into_axis_angle_format();

        let rot = rot.into_radians().into_degrees();
        let rot = rot.into_quaternion_format();

        let rot = rot.into_radians().into_degrees();
        let rot = rot.into_rotation_matrix_format();

        let rot = rot.into_radians().into_degrees();
        let rot = rot.into_rodrigues_format();

        let rot = rot.into_radians().into_degrees();
        let rot = rot.into_euler_format();

        let rot = rot.into_radians().into_degrees();

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
