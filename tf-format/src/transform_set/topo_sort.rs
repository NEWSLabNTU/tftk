use indexmap::IndexSet;
use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};

pub struct TopologicalSort<T> {
    nodes: IndexSet<T>,
    edges: HashMap<T, IndexSet<T>>,
}

impl<T> TopologicalSort<T>
where
    T: Clone + Hash + Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_edge(&mut self, a: T, b: T) {
        if a == b {
            self.nodes.insert(a);
        } else {
            self.nodes.insert(a.clone());
            self.nodes.insert(b.clone());
            self.edges.entry(a.clone()).or_default().insert(b.clone());
            self.edges.entry(b).or_default().insert(a);
        }
    }

    pub fn sort(self) -> Vec<Component<T>> {
        let Self { nodes, mut edges } = self;
        let mut visited: IndexSet<T> = IndexSet::new();

        nodes
            .into_iter()
            .filter_map(|start| {
                let mut fronts: VecDeque<(T, T)> = {
                    if !visited.insert(start.clone()) {
                        return None;
                    }

                    edges
                        .remove(&start)
                        .into_iter()
                        .flatten()
                        .map(|next| (start.clone(), next))
                        .collect()
                };

                let mut seq = vec![];

                while let Some((prev, curr)) = fronts.pop_front() {
                    if !visited.insert(curr.clone()) {
                        continue;
                    }
                    fronts.extend(
                        edges
                            .remove(&curr)
                            .into_iter()
                            .flatten()
                            .map(|next| (curr.clone(), next)),
                    );
                    seq.push((prev, curr));
                }

                Some(Component { start, seq })
            })
            .collect()
    }
}

impl<T> Default for TopologicalSort<T> {
    fn default() -> Self {
        Self {
            nodes: IndexSet::new(),
            edges: HashMap::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Component<T> {
    pub start: T,
    pub seq: Vec<(T, T)>,
}

#[cfg(test)]
mod tests {
    use super::{Component, TopologicalSort};

    #[test]
    fn topo_sort_test() {
        {
            let tree: TopologicalSort<u32> = TopologicalSort::new();
            assert!(tree.sort().is_empty());
        }

        {
            let mut tree = TopologicalSort::new();
            tree.insert_edge(1, 1);
            assert_eq!(
                tree.sort(),
                vec![Component {
                    start: 1,
                    seq: vec![]
                }]
            );
        }

        {
            let mut tree = TopologicalSort::new();
            tree.insert_edge(1, 2);
            tree.insert_edge(1, 3);
            tree.insert_edge(2, 3);
            assert_eq!(
                tree.sort(),
                vec![Component {
                    start: 1,
                    seq: vec![(1, 2), (1, 3)]
                }]
            );
        }

        {
            let mut tree = TopologicalSort::new();
            tree.insert_edge(1, 2);
            tree.insert_edge(1, 3);
            tree.insert_edge(2, 3);

            tree.insert_edge(4, 5);
            tree.insert_edge(6, 5);

            tree.insert_edge(7, 7);

            assert_eq!(
                tree.sort(),
                vec![
                    Component {
                        start: 1,
                        seq: vec![(1, 2), (1, 3)]
                    },
                    Component {
                        start: 4,
                        seq: vec![(4, 5), (5, 6)]
                    },
                    Component {
                        start: 7,
                        seq: vec![]
                    },
                ]
            );
        }
    }
}
