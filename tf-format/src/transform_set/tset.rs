use super::{error::InsertionError, mutual_set::MutualSet, serialized::SerializedTransformSet};
use crate::{transform_set::topo_sort::TopologicalSort, CoordTransform};
use approx::abs_diff_eq;
use itertools::chain;
use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
    rc::Rc,
};

/// Represent a set of related or disjoint coordinate transformations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "SerializedTransformSet", into = "SerializedTransformSet")]
pub struct TransformSet {
    mid: usize,
    coord_to_mid: HashMap<String, usize>,
    pub(crate) mid_to_set: HashMap<usize, MutualSet>,
}

impl TransformSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, src: &str, dst: &str) -> Option<na::Isometry3<f64>> {
        let src_mid = self.coord_to_mid.get(src)?;
        let dst_mid = self.coord_to_mid.get(dst)?;

        if src_mid != dst_mid {
            return None;
        }

        self.mid_to_set[src_mid].get(src, dst)
    }

    pub fn contains_coord(&self, coord: &str) -> bool {
        self.coord_to_mid.contains_key(coord)
    }

    pub fn insert(
        &mut self,
        src: &str,
        dst: &str,
        tf: na::Isometry3<f64>,
    ) -> Result<(), InsertionError> {
        let src_mid = self.coord_to_mid.get(src).copied();
        let dst_mid = self.coord_to_mid.get(dst).copied();

        match (src_mid, dst_mid) {
            (None, None) => {
                let new_mid = self.next_mid();

                let mut new_set = MutualSet::new();
                new_set.insert(src, dst, tf)?;
                self.mid_to_set.insert(new_mid, new_set);

                self.coord_to_mid.insert(src.to_string(), new_mid);
                self.coord_to_mid.insert(dst.to_string(), new_mid);

                Ok(())
            }
            (None, Some(dst_mid)) => {
                self.coord_to_mid.insert(src.to_string(), dst_mid);
                let mutual_set = self.mid_to_set.get_mut(&dst_mid).unwrap();
                mutual_set.insert(src, dst, tf)
            }
            (Some(src_mid), None) => {
                self.coord_to_mid.insert(dst.to_string(), src_mid);
                let mutual_set = self.mid_to_set.get_mut(&src_mid).unwrap();
                mutual_set.insert(src, dst, tf)
            }
            (Some(src_mid), Some(dst_mid)) => {
                if src_mid != dst_mid {
                    let new_mid = self.next_mid();

                    let mut src_set = self.mid_to_set.remove(&src_mid).unwrap();
                    let dst_set = self.mid_to_set.remove(&dst_mid).unwrap();

                    for coord in chain!(src_set.coord_iter(), dst_set.coord_iter()) {
                        *self.coord_to_mid.get_mut(coord).unwrap() = new_mid;
                    }

                    src_set.insert(src, dst, tf).unwrap();

                    let new_set = src_set.merge(dst_set).unwrap();
                    self.mid_to_set.insert(new_mid, new_set);

                    Ok(())
                } else {
                    let mutual_set = self.mid_to_set.get_mut(&src_mid).unwrap();
                    mutual_set.insert(src, dst, tf)
                }
            }
        }
    }

    pub fn merge(self, other: Self) -> Self {
        todo!()
    }

    pub fn try_from_iter<T>(iter: T) -> Result<Self, InsertionError>
    where
        T: IntoIterator<Item = CoordTransform>,
    {
        let mut topo_sort = TopologicalSort::new();
        let mut adj: HashMap<Rc<String>, HashMap<Rc<String>, _>> = HashMap::new();

        for trans in iter {
            let CoordTransform { src, dst, tf } = trans;
            let tf: na::Isometry3<f64> = tf.into();
            let src = Rc::new(src);
            let dst = Rc::new(dst);

            if src == dst {
                let id = na::Isometry3::identity();
                if !abs_diff_eq!(tf, id) {
                    return Err(InsertionError::inconsistent_transform_error(id, tf));
                }
            } else {
                adj.entry(dst.clone())
                    .or_default()
                    .insert(src.clone(), tf.inverse());
                adj.entry(src.clone()).or_default().insert(dst.clone(), tf);
            }

            topo_sort.insert_edge(src, dst);
        }

        let mid_to_set: HashMap<_, _> = topo_sort
            .sort()
            .into_iter()
            .enumerate()
            .map(|(mid, comp)| {
                let mut mset = MutualSet::new();

                for (src, dst) in comp.seq {
                    let tf = adj[&src][&dst];
                    mset.insert(&src, &dst, tf).unwrap();
                }

                (mid, mset)
            })
            .collect();

        let coord_to_mid: HashMap<_, _> = mid_to_set
            .iter()
            .flat_map(|(&mid, mset)| mset.coord_iter().map(move |coord| (coord.to_string(), mid)))
            .collect();

        Ok(Self {
            mid: mid_to_set.len(),
            coord_to_mid,
            mid_to_set,
        })
    }

    pub fn from_json_file<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let reader = BufReader::new(File::open(path)?);
        let set: TransformSet = serde_json::from_reader(reader)?;
        Ok(set)
    }

    pub fn from_json_dir<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut merged_set = TransformSet::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if !path.canonicalize()?.is_file() {
                continue;
            }

            let Some(ext) = path.extension() else {
                continue;
            };

            if ext != "json" {
                continue;
            }

            let curr_set = TransformSet::from_json_file(path)?;
            merged_set = merged_set.merge(curr_set);
        }

        Ok(merged_set)
    }

    fn next_mid(&mut self) -> usize {
        let out = self.mid;
        self.mid += 1;
        out
    }
}

impl Default for TransformSet {
    fn default() -> Self {
        Self {
            mid: 0,
            coord_to_mid: HashMap::new(),
            mid_to_set: HashMap::new(),
        }
    }
}
