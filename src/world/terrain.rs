use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use cgmath::{Array, Matrix3, Vector2};
use graphics::{
    color::Color3,
    ctx::Frame,
    sprite::{Sprite, SpriteParams},
};
use network::{
    ctx::{ConnectionHandle, Network},
    BaseNetworkSide, Client, ClientOnly, ClientOnlySerde, NetworkSide, Server,
};
use platform::{debug, info};
use rand::Rng;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{assets::SpriteSheets, mods::api::ModsApi};

use super::{
    generator::{Generate, WorldGenerator},
    ServerWorldSendChunkPacket, ServerWorldSetTilePacket,
};

pub struct Terrain<S: NetworkSide> {
    pub tile_registry: TileRegistry<S>,
    chunks: HashMap<Vector2<i32>, Chunk>,
}

impl<S: NetworkSide> Terrain<S> {
    pub fn new(api: &ModsApi<S>) -> Self {
        Self {
            tile_registry: TileRegistry::new(api),
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
    fn generate(&mut self, generator: &mut WorldGenerator) {
        for cc in self.chunks_in_sight() {
            let mut chunk = Chunk::new(cc);
            chunk.generate(generator);
            self.chunks.insert(cc, chunk);
        }
    }
}

impl Terrain<Server> {
    pub fn send(&self, network: &mut Network<Server>, conn: ConnectionHandle<Server>) {
        for chunk in self.chunks.values().cloned() {
            network.send(&[ServerWorldSendChunkPacket { chunk }], &[conn]);
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
            tiles: [[DIRT_TILE_ID; CHUNK_SIZE]; CHUNK_SIZE],
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
    fn generate(&mut self, generator: &mut WorldGenerator) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                if generator.rng.gen::<f32>() > 0.90 {
                    self.tiles[x][y] = GOLD_ORE_TILE_ID;
                } else if generator.rng.gen::<f32>() > 0.95 {
                    self.tiles[x][y] = DIAMOND_ORE_TILE_ID;
                }

                if x % CHUNK_SIZE == 0 && y % CHUNK_SIZE == 0 {
                    self.tiles[x][y] = DEBUG_TILE_ID;
                }
            }
        }
    }
}

pub struct TileRegistry<S: NetworkSide> {
    entries: Vec<Tile<S>>,
}

impl<S: NetworkSide> TileRegistry<S> {
    pub fn new(api: &ModsApi<S>) -> Self {
        Self {
            entries: api.tile_registry().collect(),
        }
    }

    pub fn add(&mut self, tile: Tile<S>) {
        self.entries.push(tile)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tile<S: BaseNetworkSide> {
    id: String,
    client_sprite: ClientOnlySerde<S, Sprite<SpriteSheets>>,
}

const DEBUG_TILE_ID: TileId = 0;
const DIRT_TILE_ID: TileId = 1;
const DIAMOND_ORE_TILE_ID: TileId = 2;
const GOLD_ORE_TILE_ID: TileId = 3;
