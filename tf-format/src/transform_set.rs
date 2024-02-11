use crate::Transform;
use anyhow::{bail, ensure, Result};
use approx::abs_diff_eq;
use indexmap::IndexMap;
use itertools::Itertools;
use nalgebra as na;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordTransform {
    pub src: String,
    pub dst: String,
    pub tf: Transform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "SerializedTransformSet", into = "SerializedTransformSet")]
pub struct TransformSet {
    lookup: IndexMap<String, Vec<na::Isometry3<f64>>>,
}

impl TransformSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, src: &str, dst: &str) -> Option<na::Isometry3<f64>> {
        let Self { lookup, .. } = self;
        let src_idx = lookup.get_index_of(src)?;
        let dst_idx = lookup.get_index_of(dst)?;
        let tf = self.get_by_range(src_idx, dst_idx)?;
        Some(tf)
    }

    pub fn insert(
        &mut self,
        src: &str,
        dst: &str,
        tf: na::Isometry3<f64>,
    ) -> Result<(), Option<na::Isometry3<f64>>> {
        let Self { lookup, .. } = self;

        let src_idx = lookup.get_index_of(src);
        let dst_idx = lookup.get_index_of(dst);

        let (to_base, from_idx, tf) = match (src_idx, dst_idx) {
            (None, None) => {
                return if lookup.is_empty() {
                    lookup.insert(src.to_string(), vec![tf]);
                    lookup.insert(dst.to_string(), vec![]);
                    Ok(())
                } else {
                    Err(None)
                };
            }
            (None, Some(dst_idx)) => (src, dst_idx, tf.inverse()),
            (Some(src_idx), None) => (dst, src_idx, tf),
            (Some(src_idx), Some(dst_idx)) => {
                #[cfg(test)]
                self.assert_consistency();

                let expect = self.get_by_range(src_idx, dst_idx).unwrap();
                return if abs_diff_eq!(tf, expect, epsilon = 1e-6) {
                    Ok(())
                } else {
                    Err(Some(expect))
                };
            }
        };

        self.lookup.insert(to_base.to_string(), vec![]);
        let last_idx = self.lookup.len() - 1;
        let former_tf = self.get_by_range(from_idx, last_idx - 1).unwrap();
        let mut latter_tf = former_tf.inverse() * tf;

        for nth in 0..usize::BITS {
            let offset = 1 << nth;

            let idx = last_idx - offset;
            let (_, tf_list) = self.lookup.get_index_mut(idx).unwrap();
            tf_list.push(latter_tf);

            let Some(idx2) = idx.checked_sub(offset) else {
                break;
            };

            let (_, tf_list2) = self.lookup.get_index(idx2).unwrap();
            latter_tf = tf_list2[nth as usize] * latter_tf;
        }

        #[cfg(test)]
        self.assert_consistency();

        Ok(())
    }

    pub fn try_from_iter<T>(iter: T) -> Result<Self>
    where
        T: IntoIterator<Item = CoordTransform>,
    {
        let edges: Vec<_> = iter
            .into_iter()
            .map(|trans| {
                let CoordTransform { src, dst, tf } = trans;
                let tf: na::Isometry3<f64> = tf.into();

                ensure!(
                    src != dst || abs_diff_eq!(tf, na::Isometry3::identity()),
                    "The transform from '{src}' to itself must be an identity transform"
                );

                Ok((src, dst, tf))
            })
            .try_collect()?;

        let adj: HashMap<String, HashMap<String, _>> = edges
            .into_iter()
            .filter(|(src, dst, _)| src != dst)
            .flat_map(|(src, dst, tf)| {
                [(src.clone(), (dst.clone(), tf)), (dst, (src, tf.inverse()))]
            })
            .into_grouping_map()
            .collect();

        // BFS
        let mut set = TransformSet::new();
        let mut visited = HashSet::with_capacity(adj.len());
        let mut frontiers = VecDeque::with_capacity(adj.len());

        let Some(first) = adj.keys().next() else {
            return Ok(set);
        };
        frontiers.push_back(first);

        while let Some(src) = frontiers.pop_front() {
            if !visited.insert(src) {
                continue;
            }

            for (dst, tf) in &adj[src] {
                eprintln!("{src} {dst}");

                match set.insert(src, dst, *tf) {
                    Ok(()) => {}
                    Err(Some(expect)) => {
                        let expect = Transform::from(expect)
                            .into_axis_angle_format()
                            .into_degrees()
                            .normalize_rotation();
                        let actual = Transform::from(*tf)
                            .into_axis_angle_format()
                            .into_degrees()
                            .normalize_rotation();

                        bail!(
                            r#"Inconsistent transform from "{src}" to "{dst}".
Expect
"""
{expect:#?}
"""
but found
"""
{actual:#?}
"""
"#
                        );
                    }
                    Err(None) => unreachable!(),
                }

                frontiers.push_back(dst);
            }
        }

        ensure!(visited.len() == adj.len(), "disjoint transforms found");

        Ok(set)
    }

    fn get_by_range(&self, start: usize, end: usize) -> Option<na::Isometry3<f64>> {
        let Self { lookup, .. } = self;

        let (min, max, inverse) = if start <= end {
            (start, end, false)
        } else {
            (end, start, true)
        };

        if max >= lookup.len() {
            return None;
        }

        let mut diff = max - min;
        let mut curr = min;
        let mut tf = na::Isometry3::identity();

        while diff != 0 {
            let count = diff.trailing_zeros();
            let pow = 1 << count;

            let (_, tf_list) = lookup.get_index(curr).unwrap();

            tf = tf_list[count as usize] * tf;
            diff ^= pow;
            curr += pow;
        }

        let tf = if inverse { tf.inverse() } else { tf };
        Some(tf)
    }

    #[cfg(test)]
    pub fn assert_consistency(&self) {
        use itertools::izip;

        let Self { lookup, .. } = self;

        let bases: Vec<_> = lookup
            .iter()
            .enumerate()
            .map(|(idx, (base, _))| (idx, base))
            .collect();
        eprintln!("{bases:?}");

        for (idx, (base1, list1)) in izip!(0.., lookup) {
            let max_pow = (0..).find(|pow| {
                let offset = 1 << pow;
                idx + offset >= lookup.len()
            });

            let Some(max_pow) = max_pow else {
                continue;
            };

            for pow in 1..max_pow {
                let offset = 1 << (pow - 1);
                let (base2, list2) = lookup.get_index(idx + offset).unwrap();
                let (base3, _) = lookup.get_index(idx + offset * 2).unwrap();

                assert!(
                    abs_diff_eq!(list1[pow], list1[pow - 1] * list2[pow - 1], epsilon = 1e-6),
                    "The conversion {base1}->{base3} is not consistent with {base1}->{base2} * {base2}->{base3}"
                );
            }
        }
    }
}

impl Default for TransformSet {
    fn default() -> Self {
        Self {
            lookup: IndexMap::new(),
        }
    }
}

impl TryFrom<SerializedTransformSet> for TransformSet {
    type Error = anyhow::Error;

    fn try_from(set: SerializedTransformSet) -> Result<Self, Self::Error> {
        Self::try_from_iter(set.0)
    }
}

impl From<TransformSet> for SerializedTransformSet {
    fn from(set: TransformSet) -> Self {
        let TransformSet { lookup, .. } = set;
        let vec: Vec<_> = lookup
            .iter()
            .tuple_windows()
            .map(|((src, tf_list), (dst, _))| CoordTransform {
                src: src.to_string(),
                dst: dst.to_string(),
                tf: tf_list[0].into(),
            })
            .collect();

        Self(vec)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct SerializedTransformSet(pub Vec<CoordTransform>);

#[cfg(test)]
mod tests {
    use super::TransformSet;
    use anyhow::Result;
    use approx::assert_abs_diff_eq;
    use serde::Deserialize;
    use nalgebra as na;
    use std::{fs::File, io::BufReader, path::Path};

    const CONFIG_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/example_config");

    #[test]
    fn transfrom_set_serde() -> Result<()> {
        loop {
            let config_dir = Path::new(CONFIG_DIR);
            let set: TransformSet = load_json(config_dir.join("tfset1.json"))?;
            eprintln!();

            assert!(set.get("map", "xxx").is_none());
            assert!(set.get("xxx", "map").is_none());
            assert!(set
                .get(
                    "xxx", "yyy
"
                )
                .is_none());

            assert_abs_diff_eq!(
                set.get("lidar1", "lidar2").unwrap(),
                set.get("lidar2", "lidar1").unwrap().inverse(),
                epsilon = 1e-6
            );

            assert_abs_diff_eq!(
                set.get("map", "car").unwrap(),
                set.get("car", "map").unwrap().inverse(),
                epsilon = 1e-6
            );

            assert_abs_diff_eq!(
                set.get("lidar1", "car").unwrap() * set.get("car", "lidar2").unwrap(),
                set.get("lidar1", "lidar2").unwrap(),
                epsilon = 1e-6
            );

            assert_abs_diff_eq!(
                set.get("lidar1", "car").unwrap() * set.get("car", "lidar1").unwrap(),
                set.get("lidar1", "lidar1").unwrap(),
                epsilon = 1e-6
            );

            assert_abs_diff_eq!(
                set.get("map", "map").unwrap(),
                na::Isometry3::identity(),
                epsilon = 1e-6
            );
        }
        Ok(())
    }

    fn load_json<T, P>(path: P) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader)?;
        Ok(data)
    }
}
