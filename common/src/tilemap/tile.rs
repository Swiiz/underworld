use crate::utils::registry::RecordId;

#[derive(Default)]
pub struct Tile {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TileId(pub RecordId);
