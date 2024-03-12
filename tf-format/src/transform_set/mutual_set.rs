use std::collections::HashSet;

use super::error::InsertionError;
use anyhow::Result;
use approx::abs_diff_eq;
use indexmap::IndexMap;
use nalgebra as na;

#[derive(Debug, Clone)]
pub(crate) struct MutualSet {
    pub(crate) lookup: IndexMap<String, Vec<na::Isometry3<f64>>>,
}

impl MutualSet {
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
    ) -> Result<(), InsertionError> {
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
                    Err(InsertionError::DisjointCoordinates {
                        src: src.to_string(),
                        dst: dst.to_string(),
                    })
                };
            }
            (None, Some(dst_idx)) => (src, dst_idx, tf.inverse()),
            (Some(src_idx), None) => (dst, src_idx, tf),
            (Some(src_idx), Some(dst_idx)) => {
                #[cfg(test)]
                self.assert_consistency();

                let expect = self.get_by_range(src_idx, dst_idx).unwrap();

                if abs_diff_eq!(tf, expect, epsilon = 1e-6) {
                    return Ok(());
                } else {
                    return Err(InsertionError::inconsistent_transform_error(expect, tf));
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

    pub fn coord_iter(&self) -> impl Iterator<Item = &str> {
        self.lookup.keys().map(|coord| coord.as_str())
    }

    pub fn merge(mut self, other: Self) -> Result<Self, (Self, Self)> {
        let common_coord = {
            let lcoords: HashSet<_> = self.lookup.keys().collect();
            let rcoords: HashSet<_> = other.lookup.keys().collect();

            let Some(common_coord) = lcoords.intersection(&rcoords).next() else {
                return Err((self, other));
            };
            common_coord.to_string()
        };

        for coord in other.lookup.keys() {
            let tf = other.get(&common_coord, coord).unwrap();
            self.insert(&common_coord, coord, tf).unwrap();
        }

        Ok(self)
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
        let mut prod_tf = na::Isometry3::identity();

        while diff != 0 {
            let pow = diff.trailing_zeros();
            let step = 1 << pow;

            let tf = self.get_by_pow_index(curr, pow as usize).unwrap();
            prod_tf *= tf;

            diff ^= step;
            curr += step;
        }

        debug_assert_eq!(curr, max);

        let tf = if inverse { prod_tf.inverse() } else { prod_tf };
        Some(tf)
    }

    fn get_by_pow_index(&self, at: usize, pow: usize) -> Option<&na::Isometry3<f64>> {
        let Self { lookup, .. } = self;
        let (_, tf_list) = lookup.get_index(at)?;
        tf_list.get(pow)
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

impl Default for MutualSet {
    fn default() -> Self {
        Self {
            lookup: IndexMap::new(),
        }
    }
}
