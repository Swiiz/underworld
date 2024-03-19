use std::collections::HashMap;

use cgmath::{Array, ElementWise, Matrix3, Transform, Vector2};
use graphics::{
    color::Color3,
    ctx::Frame,
    sprite::{Sprite, SpriteParams},
};
use network::{
    connection::PacketQueue,
    ctx::{ConnectionHandle, Network},
    Client, ClientOnly, NetworkSide, Server,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::assets::{SpriteSheets, DEBUG_SPRITE, VOID_SPRITE};

use super::{
    generator::{Generate, WorldGenerator},
    ServerWorldSendChunkPacket, ServerWorldSetTilePacket,
};

pub struct Terrain<S: NetworkSide> {
    tile_registry: TileRegistry<S>,
    chunks: HashMap<Vector2<i32>, Chunk>,
}

impl<S: NetworkSide> Terrain<S> {
    pub fn new() -> Self {
        Self {
            tile_registry: TileRegistry::new(),
            chunks: HashMap::new(),
        }
    }

    fn chunks_in_sight(&self) -> impl Iterator<Item = Vector2<i32>> {
        (-2..2)
            .into_iter()
            .map(|x| (-2..2).into_iter().map(move |y| (x, y)))
            .flatten()
            .map(|(x, y)| Vector2 { x, y })
    }
}

impl Generate for Terrain<Server> {
    fn generate(
        &mut self,
        generator: &mut WorldGenerator,
        changes_queue: &mut PacketQueue<Server>,
    ) {
        for cc in self.chunks_in_sight() {
            let mut chunk = Chunk::new(cc);
            chunk.generate(generator, changes_queue);
            self.chunks.insert(cc, chunk);
        }
    }
}

impl Terrain<Server> {
    pub fn send(&self, network: &mut Network<Server>, conn: ConnectionHandle<Server>) {
        for chunk in self.chunks.values().cloned() {
            network.send(&ServerWorldSendChunkPacket { chunk }, conn);
        }
    }
}

impl Terrain<Client> {
    pub fn client_update(&mut self, network: &mut Network<Client>) {
        network.on::<ServerWorldSendChunkPacket>(|_, p, _| {
            self.chunks.insert(p.chunk.coords, p.chunk);
        });

        network.on::<ServerWorldSetTilePacket>(|_, p, _| {
            let (x, y) = p.tile_coords.into();
            self.chunks
                .entry(p.chunk_coords)
                .or_insert(Chunk::new(p.chunk_coords))
                .tiles[x as usize][y as usize] = p.new_tile_id;
        });
    }

    pub fn render(&self, frame: &mut Frame) {
        for cc in self.chunks_in_sight() {
            if let Some(chunk) = self.chunks.get(&cc) {
                chunk.render(frame, &self.tile_registry);
            }
        }
    }
}

pub const CHUNK_SIZE: usize = 32;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk {
    coords: Vector2<i32>,
    tiles: [[TileId; CHUNK_SIZE]; CHUNK_SIZE],
}

pub type TileId = u16;

impl Chunk {
    pub fn new(coords: Vector2<i32>) -> Self {
        Self {
            coords,
            tiles: [[VOID_TILE_ID; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }

    fn render(&self, frame: &mut Frame, tile_registry: &TileRegistry<Client>) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                frame.draw(
                    tile_registry.entries[self.tiles[x][y] as usize]
                        .client_sprite
                        .clone(),
                    SpriteParams {
                        depth: 1.,
                        tint: Color3::WHITE,
                        transform: Matrix3::from_scale(0.1)
                            * Matrix3::from_translation(
                                self.coords.map(|e| e as f32) * CHUNK_SIZE as f32
                                    + Vector2::new(x as f32, y as f32)
                                    + Vector2::from_value(0.5),
                            ),
                    },
                )
            }
        }
    }
}

impl Generate for Chunk {
    fn generate(
        &mut self,
        generator: &mut WorldGenerator,
        changes_queue: &mut PacketQueue<Server>,
    ) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                if generator.rng.gen::<bool>() {
                    self.tiles[x][y] = DEBUG_TILE_ID;
                    changes_queue.push(&ServerWorldSetTilePacket {
                        chunk_coords: self.coords,
                        tile_coords: Vector2 {
                            x: x as i32,
                            y: y as i32,
                        },
                        new_tile_id: DEBUG_TILE_ID,
                    })
                }
            }
        }
    }
}

pub struct TileRegistry<S: NetworkSide> {
    entries: Vec<Tile<S>>,
}

impl<S: NetworkSide> TileRegistry<S> {
    pub fn new() -> Self {
        Self {
            entries: vec![void_tile(), debug_tile()],
        }
    }
}

pub struct Tile<S: NetworkSide> {
    client_sprite: ClientOnly<S, Sprite<SpriteSheets>>,
}

const VOID_TILE_ID: TileId = 0;
fn void_tile<S: NetworkSide>() -> Tile<S> {
    Tile {
        client_sprite: S::client_only(VOID_SPRITE),
    }
}
const DEBUG_TILE_ID: TileId = 1;
fn debug_tile<S: NetworkSide>() -> Tile<S> {
    Tile {
        client_sprite: S::client_only(DEBUG_SPRITE),
    }
}
