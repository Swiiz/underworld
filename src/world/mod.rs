use cgmath::Vector2;
use graphics::ctx::Frame;
use network::{
    connection::PacketQueue, ctx::Network, protocol::Packet, Client, NetworkSide, Server,
    ServerOnly,
};
use serde::{Deserialize, Serialize};

use crate::protocol::protocol;

use self::{
    generator::{Generate, WorldGenerator},
    player::Player,
    terrain::{Chunk, Terrain, TileId},
};

mod generator;
mod player;
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerWorldSetTilePacket {
    chunk_coords: Vector2<i32>,
    tile_coords: Vector2<i32>,
    new_tile_id: TileId,
}
impl Packet for ServerWorldSetTilePacket {
    type Side = Server;
}

pub struct World<S: NetworkSide> {
    terrain: Terrain<S>,
    players: Vec<Player>,

    server_generator: ServerOnly<S, WorldGenerator>,
    changes_queue: PacketQueue<S>,
}

impl<S: NetworkSide> World<S> {
    pub fn new() -> Self {
        let server_generator = S::server_only(WorldGenerator::new(None));

        let changes_queue = PacketQueue::new(protocol());

        let terrain = Terrain::new();
        let players = Vec::new();

        Self {
            terrain,
            players,
            server_generator,
            changes_queue,
        }
    }
}

impl World<Server> {
    pub fn server_generate(&mut self) {
        self.terrain.generate(&mut self.server_generator);
    }

    pub fn server_update(&mut self, network: &mut Network<Server>) {
        network.on::<ClientLoadWorldPacket>(|network, _, conn| {
            self.terrain.send(network, conn);
        });
        self.changes_queue.submit(network);
    }
}

impl World<Client> {
    pub fn client_update(&mut self, network: &mut Network<Client>) {
        self.terrain.client_update(network);
        self.changes_queue.submit(network);
    }

    pub fn render(&self, frame: &mut Frame) {
        self.terrain.render(frame);
    }
}
