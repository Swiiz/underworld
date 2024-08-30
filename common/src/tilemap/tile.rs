use serde::{Deserialize, Serialize};

use crate::utils::registry::RecordId;

#[derive(Default, Serialize, Deserialize)]
pub struct Tile {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TileId(pub RecordId);
