use super::{error::InsertionError, TransformSet};
use crate::CoordTransform;
use anyhow::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct SerializedTransformSet(pub Vec<CoordTransform>);

impl TryFrom<SerializedTransformSet> for TransformSet {
    type Error = InsertionError;

    fn try_from(set: SerializedTransformSet) -> Result<Self, Self::Error> {
        Self::try_from_iter(set.0)
    }
}

impl From<TransformSet> for SerializedTransformSet {
    fn from(tset: TransformSet) -> Self {
        let vec: Vec<_> = tset
            .mid_to_set
            .values()
            .flat_map(|mset| {
                mset.lookup
                    .iter()
                    .tuple_windows()
                    .map(|((src, tf_list), (dst, _))| CoordTransform {
                        src: src.to_string(),
                        dst: dst.to_string(),
                        tf: tf_list[0].into(),
                    })
            })
            .collect();

        Self(vec)
    }
}
