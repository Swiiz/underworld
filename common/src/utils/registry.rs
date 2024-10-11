use std::{collections::HashMap, marker::PhantomData, path::Path};

use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

    pub fn load_json_part_from_disk_mapped<Item: DeserializeOwned>(
        path: impl AsRef<Path>,
        part: &str,
        map_fn: impl Fn(Item) -> T,
    ) -> Self {
        let raw = serde_json::from_str::<HashMap<String, HashMap<String, serde_json::Value>>>(
            std::fs::read_to_string(path).unwrap().as_str(),
        )
        .expect("Failed to load registry manifest");
        let mut entries = raw
            .into_iter()
            .map(|(name, parts)| {
                let v = parts
                    .into_iter()
                    .find_map(|(k, v)| (k == part).then(|| v))
                    .expect("Failed to load registry manifest selected part");
                (
                    name,
                    map_fn(serde_json::from_value::<Item>(v).expect("Failed to parse common tile")),
                )
            })
            .collect::<Vec<_>>();

        entries.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            _marker: PhantomData,
            entries: IndexMap::<String, T, _>::from_iter(entries),
        }
    }

    pub fn load_json_part_from_disk(path: impl AsRef<Path>, part: &str) -> Self
    where
        T: DeserializeOwned,
    {
        Self::load_json_part_from_disk_mapped(path, part, |v| v)
    }

    pub fn load_whole_json_from_disk_mapped<Item: DeserializeOwned>(
        path: impl AsRef<Path>,
        map_fn: impl Fn(Item) -> T,
    ) -> Self {
        let mut entries = serde_json::from_str::<HashMap<String, Item>>(
            std::fs::read_to_string(path).unwrap().as_str(),
        )
        .expect("Failed to load registry manifest")
        .into_iter()
        .map(|(k, v)| (k, map_fn(v)))
        .collect::<Vec<_>>();

        entries.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            _marker: PhantomData,
            entries: IndexMap::<String, T, _>::from_iter(entries),
        }
    }

    pub fn load_whole_json_from_disk(path: impl AsRef<Path>) -> Self
    where
        T: DeserializeOwned,
    {
        Self::load_whole_json_from_disk_mapped(path, |v| v)
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
