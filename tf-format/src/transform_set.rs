mod error;
pub(crate) mod mutual_set;
mod serialized;
mod topo_sort;
mod transform_set;

use crate::Transform;
use serde::{Deserialize, Serialize};

pub use transform_set::TransformSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordTransform {
    pub src: String,
    pub dst: String,
    pub tf: Transform,
}
