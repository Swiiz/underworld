use common::state::CommonState;
use ecs::Entities;

use crate::assets::ServerAssets;

pub struct ServerState {
    pub common: CommonState,
}

impl ServerState {
    pub fn new(assets: &ServerAssets) -> Self {
        Self {
            common: CommonState {
                //terrain: TileMap::new(assets),
                entities: Entities::new(),
            },
        }
    }
}
