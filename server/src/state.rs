use std::net::SocketAddr;

use common::{
    core::spatial::Position, network::proto::play::ClientboundSetEntityPosition, state::CommonState,
};
use ecs::Entities;

use crate::{assets::ServerAssets, network::NetworkServer};

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

    pub fn set_player_position(
        &mut self,
        addr: &SocketAddr,
        pos: Position,
        network: &mut NetworkServer,
    ) {
        let entity = network.get_remote(&addr).unwrap().entity;
        self.common.entities.edit(entity).unwrap().set(pos);
        network.broadcast_except(
            &addr,
            &ClientboundSetEntityPosition {
                entity: entity.into(),
                pos,
            },
        );
    }
}
