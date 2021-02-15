//! Contains the various decision tree learning algorithms.

use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    iter::IntoIterator,
    marker::PhantomData,
    slice::Iter as SliceIter,
    vec::IntoIter as VecIter,
};

pub mod c45;
pub mod id3;
pub mod id4;

/// A collection of attributes.
pub trait Attributes<L, V> {
    fn get_attribute(&self, key: &L) -> &V;
}

impl<L, V> Attributes<L, V> for BTreeMap<L, V>
where
    L: Ord,
{
    fn get_attribute(&self, key: &L) -> &V {
        self.get(key).unwrap()
    }
}

impl<L, V> Attributes<L, V> for HashMap<L, V>
where
    L: Hash + Eq,
{
    fn get_attribute(&self, key: &L) -> &V {
        self.get(key).unwrap()
    }
}

/// A set of data. Each entry contains two things: An [`Attributes`] instance and a result.
pub struct DataSet<A, R, L, V>
where
    A: Attributes<L, V>,
{
    data: Vec<(A, R)>,
    labels: Vec<L>,
    possible_results: Vec<R>,
    _marker: PhantomData<V>,
}
impl<A: Attributes<L, V>, R, L, V> DataSet<A, R, L, V> {
    pub fn new(labels: Vec<L>, possible_results: Vec<R>) -> Self {
        Self {
            data: Vec::new(),
            labels,
            possible_results,
            _marker: PhantomData,
        }
    }

    pub fn possible_results(&self) -> &[R] {
        &self.possible_results
    }

    pub fn labels(&self) -> &[L] {
        &self.labels
    }

    pub fn add_entry(&mut self, attributes: A, result: R) {
        self.data.push((attributes, result));
    }

    pub fn iter(&self) -> SliceIter<(A, R)> {
        self.data.iter()
    }
}

impl<A: Attributes<L, V>, R, L, V> IntoIterator for DataSet<A, R, L, V> {
    type Item = (A, R);
    type IntoIter = VecIter<(A, R)>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}
