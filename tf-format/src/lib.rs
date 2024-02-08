mod conv_impl;
mod rotation;
mod unit;

pub use crate::rotation::{
    AxisAngle, Euler, EulerAxis, EulerAxisOrder, Quaternion, Rotation, RotationMatrix,
};
pub use unit::{Angle, Length};

use noisy_float::types::R64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub r: Rotation,
    pub t: Translation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Translation(pub [R64; 3]);
