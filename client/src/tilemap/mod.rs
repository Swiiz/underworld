use std::collections::HashMap;

use graphics::{
    ctx::Frame,
    maths::{Array, Matrix3, Vector2},
    sprite::SpriteDrawParams,
};
use tile::{TileId, TileRegistry};

use crate::assets::Assets;

pub mod tile;

pub struct TileMapStorage {
    pub tile_registry: TileRegistry,
    tile_maps: Vec<TileMap>,
}

impl TileMapStorage {
    pub fn new() -> Self {
        Self {
            tile_registry: TileRegistry::new(),
            tile_maps: vec![],
        }
    }

    pub fn add_tile_map(&mut self, tile_map: TileMap) {
        self.tile_maps.push(tile_map);
    }

    pub fn selected_tilemap(&self) -> Option<&TileMap> {
        self.tile_maps.first() //TODO
    }

    pub fn render_selected(&self, frame: &mut Frame, assets: &Assets) {
        if let Some(tile_map) = self.selected_tilemap() {
            tile_map.render(frame, assets, &self.tile_registry);
        }
    }
}

pub struct TileMap {
    pub chunks: HashMap<ChunkCoord, TileChunk>,
}

impl TileMap {
    pub fn new(size: Vector2<i32>, background: TileId) -> Self {
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

    pub fn render(&self, frame: &mut Frame, assets: &Assets, tile_registry: &TileRegistry) {
        for (chunk_coords, chunk) in self.chunks.iter() {
            chunk.render(frame, assets, &chunk_coords, tile_registry);
        }
    }
}

const CHUNK_SIZE: Vector2<i32> = Vector2::new(16, 16);
pub struct TileChunk {
    pub tiles: [[TileId; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize],
}

impl TileChunk {
    pub fn new_filled(tile: TileId) -> Self {
        Self {
            tiles: [[tile; CHUNK_SIZE.x as usize]; CHUNK_SIZE.y as usize],
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        assets: &Assets,
        chunk_coords: &ChunkCoord,
        tile_registry: &TileRegistry,
    ) {
        let world_transform = Matrix3::from_scale(1.);
        let chunk_transform = world_transform
            * Matrix3::from_translation(chunk_coords.to_tile_coords().map(|i| i as f32));

        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let tile = tile_registry.get(*tile);

                frame.renderer.sprites.draw(
                    tile.sprite,
                    SpriteDrawParams {
                        transform: Matrix3::from_scale(0.1)
                            * chunk_transform
                            * Matrix3::from_translation(Vector2::new(x as f32, y as f32))
                            * Matrix3::from_translation(Vector2::from_value(-0.5)), // center
                        ..Default::default()
                    },
                );
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoord(pub Vector2<i32>);

impl ChunkCoord {
    pub fn to_tile_coords(&self) -> Vector2<i32> {
        self.0.zip(CHUNK_SIZE, |i, j| i * j) - Vector2::new(7, 8)
    }
}
