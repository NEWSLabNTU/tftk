mod error;
pub(crate) mod mutual_set;
mod serialized;
mod topo_sort;
mod tset;

use crate::Transform;
use serde::{Deserialize, Serialize};

pub use self::tset::TransformSet;

/// Represent coordinate transformation in 3D Euclidean space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordTransform {
    pub src: String,
    pub dst: String,
    pub tf: Transform,
}
