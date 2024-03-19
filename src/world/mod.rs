use cgmath::{Array, Vector2};
use graphics::{ctx::Frame, Graphics};
use network::{
    connection::PacketQueue, ctx::Network, protocol::Packet, Client, NetworkSide, Server,
    ServerOnly,
};
use serde::{Deserialize, Serialize};

use crate::protocol::protocol;

use self::{
    generator::{Generate, WorldGenerator},
    terrain::{Chunk, Terrain, TileId},
};

mod generator;
mod terrain;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ClientLoadWorldPacket;
impl Packet for ClientLoadWorldPacket {
    type Side = Client;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerWorldSendChunkPacket {
    chunk: Chunk,
}
impl Packet for ServerWorldSendChunkPacket {
    type Side = Server;
}
impl Default for ServerWorldSendChunkPacket {
    fn default() -> Self {
        Self {
            chunk: Chunk::new(Vector2::from_value(i32::MAX)),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerWorldSetTilePacket {
    chunk_coords: Vector2<i32>,
    tile_coords: Vector2<i32>,
    new_tile_id: TileId,
}
impl Packet for ServerWorldSetTilePacket {
    type Side = Server;
}
impl Default for ServerWorldSetTilePacket {
    fn default() -> Self {
        Self {
            chunk_coords: Vector2::new(0, 0),
            tile_coords: Vector2::new(0, 0),
            new_tile_id: 0,
        }
    }
}

pub struct World<S: NetworkSide> {
    terrain: Terrain<S>,

    server_generator: ServerOnly<S, WorldGenerator>,
    server_changes_queue: ServerOnly<S, PacketQueue<Server>>,
}

impl<S: NetworkSide> World<S> {
    pub fn new() -> Self {
        let server_generator = S::server_only(WorldGenerator::new(None));
        let server_changes_queue = S::server_only(PacketQueue::new(protocol()));

        let terrain = Terrain::new();

        Self {
            terrain,
            server_generator,
            server_changes_queue,
        }
    }
}

impl World<Server> {
    pub fn server_generate(&mut self) {
        self.terrain
            .generate(&mut self.server_generator, &mut self.server_changes_queue);
    }

    pub fn server_update(&mut self, network: &mut Network<Server>) {
        network.on::<ClientLoadWorldPacket>(|network, _, conn| {
            self.terrain.send(network, conn);
        });
        network.broadcast(&mut self.server_changes_queue);
    }
}

impl World<Client> {
    pub fn client_update(&mut self, network: &mut Network<Client>) {
        self.terrain.client_update(network)
    }

    pub fn render(&self, frame: &mut Frame) {
        self.terrain.render(frame);
    }
}
