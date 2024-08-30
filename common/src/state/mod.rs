use cgmath::Vector2;
use ecs::Entities;

use crate::{assets::CommonAssets, tilemap::TileMap};

pub struct CommonState {
    //pub terrain: TileMap,
    pub entities: Entities,
}
