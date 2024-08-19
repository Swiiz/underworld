pub struct Registry<T> {
    pub entries: Vec<T>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RecordId(pub usize);

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn register(&mut self, tile: T) -> RecordId {
        self.entries.push(tile);
        RecordId(self.entries.len() - 1)
    }

    pub fn get(&self, id: RecordId) -> &T {
        self.entries
            .get(id.0)
            .unwrap_or_else(|| panic!("Invalid tile id: {}", id.0))
    }
}
