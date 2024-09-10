use serde::{Deserialize, Serialize};

pub mod spatial;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityKind {
    Player,
}
