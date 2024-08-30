use std::collections::HashMap;

use cgmath::Vector2;
use serde::{Deserialize, Serialize};

use crate::utils::registry::RecordId;

pub mod tile;

#[derive(Clone, Serialize, Deserialize)]
pub struct TileMap {
    pub chunks: HashMap<ChunkCoord, TileChunk>,
}

impl TileMap {
    pub fn new(size: Vector2<i32>, background: RecordId) -> Self {
        let chunk_size = size.zip(CHUNK_SIZE, |i, j| i / j);
        let mut chunks = HashMap::with_capacity((chunk_size.x * chunk_size.y) as usize);
        let (xstart, ystart) = (-chunk_size.x / 2, -chunk_size.y / 2);
        let xrange = xstart..(xstart + chunk_size.x);
        let yrange = ystart..(ystart + chunk_size.y);

        for x in xrange {
            for y in yrange.clone() {
                chunks.insert(
                    ChunkCoord(Vector2::new(x, y)),
                    TileChunk::new_filled(background),
                );
            }
        }

        TileMap { chunks }
    }
}

const CHUNK_SIZE: Vector2<i32> = Vector2::new(16, 16);

#[derive(Clone, Deserialize, Serialize)]
pub struct TileChunk {
    pub tiles: [[RecordId; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize],
}

impl TileChunk {
    pub fn new_filled(tile: RecordId) -> Self {
        Self {
            tiles: [[tile; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize],
        }
    }
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoord(pub Vector2<i32>);

impl ChunkCoord {
    pub fn to_tile_coords(&self) -> Vector2<i32> {
        self.0.zip(CHUNK_SIZE, |i, j| i * j)
    }
}
