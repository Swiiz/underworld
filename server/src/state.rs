use std::net::SocketAddr;

use cgmath::Vector2;
use common::{
    core::spatial::Position, network::proto::play::ClientboundSetEntityPosition, tilemap::TileMap,
};
use ecs::Entities;

use crate::{assets::ServerAssets, network::NetworkServer};

pub struct ServerState {
    pub terrain: TileMap,
    pub entities: Entities,
}

impl ServerState {
    pub fn new(assets: &ServerAssets) -> Self {
        Self {
            terrain: TileMap::generate(Vector2::new(16, 16), assets.common.tiles.get_id("grass")),
            entities: Entities::new(),
        }
    }

    pub fn set_player_position(
        &mut self,
        addr: &SocketAddr,
        pos: Position,
        network: &mut NetworkServer,
    ) {
        let entity = network.get_remote(&addr).unwrap().entity;
        self.entities.edit(entity).unwrap().set(pos);
        network.broadcast_except(
            &addr,
            &ClientboundSetEntityPosition {
                entity: entity.into(),
                pos,
            },
        );
    }
}
