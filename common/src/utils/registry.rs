use std::marker::PhantomData;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub struct Registry<T, I: From<usize> = RecordId> {
    _marker: PhantomData<I>,
    pub entries: IndexMap<String, T>,
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RecordId(pub usize);
impl From<usize> for RecordId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}
impl Into<usize> for RecordId {
    fn into(self) -> usize {
        self.0
    }
}

impl<T, I: From<usize> + Into<usize> + Copy> Registry<T, I> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            entries: IndexMap::new(),
        }
    }

    pub fn register(&mut self, label: String, tile: T) -> RecordId {
        self.entries.insert(label, tile);
        RecordId(self.entries.len() - 1)
    }

    pub fn get(&self, label: &str) -> &T {
        self.entries
            .get(label)
            .unwrap_or_else(|| panic!("Invalid entry id: {}", label))
    }

    pub fn get_id(&self, label: &str) -> I {
        self.entries
            .get_index_of(label)
            .map(I::from)
            .unwrap_or_else(|| panic!("Invalid entry id: {}", label))
    }

    pub fn get_label(&self, id: I) -> &str {
        self.entries
            .get_index(id.into())
            .unwrap_or_else(|| panic!("Invalid entry id: {}", id.into()))
            .0
    }

    pub fn lookup(&self, id: RecordId) -> (&String, &T) {
        self.entries.iter().nth(id.0).unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.values()
    }
}
