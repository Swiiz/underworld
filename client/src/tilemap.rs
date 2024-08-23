use cgmath::{Array, Matrix3, Vector2, Zero};
use common::{
    tilemap::{tile::Tile, ChunkCoord, TileChunk, TileMap},
    utils::registry::Registry,
};
use graphics::{
    ctx::Frame,
    sprite::{Sprite, SpriteDrawParams},
};

use crate::{assets::Assets, camera::Camera, platform::PlatformInput};

pub type ClientTileRegistry = Registry<ClientTile>;

pub struct ClientTile {
    pub common: Tile,
    pub sprite: Sprite,
}

pub struct ClientTileMap {
    common: TileMap,
    relative_selected_tile: Vector2<f32>,
}

impl ClientTileMap {
    pub fn new(common: TileMap) -> Self {
        Self {
            common,
            relative_selected_tile: Vector2::new(0., 0.),
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        assets: &Assets,
        tile_registry: &ClientTileRegistry,
        camera: &Camera,
    ) {
        for (chunk_coords, chunk) in self.common.chunks.iter() {
            render_chunk(&chunk, frame, assets, &chunk_coords, tile_registry, camera)
        }

        frame.renderer.sprites.draw(
            Sprite {
                sheet: assets.get_texture("tilemap_overlay").clone(),
                pos: Vector2::zero(),
                size: Vector2::new(1, 1),
            },
            SpriteDrawParams {
                transform: camera.view_transform()
                    * Matrix3::from_translation(self.selected_tile(camera).map(|i| i as f32))
                    * Matrix3::from_translation(Vector2::from_value(-0.5)),
                ..Default::default()
            },
        );
    }

    pub fn input(&mut self, event: &PlatformInput, window_size: impl Into<(u32, u32)>) {
        let PlatformInput::CursorMoved { x, y } = *event else {
            return;
        };

        let (w, h) = window_size.into();
        let aspect_ratio = w as f32 / h as f32;

        let mut pos = Vector2::new((x as f32 * aspect_ratio) / w as f32, y as f32 / h as f32) * 2.
            - Vector2::new(1. * aspect_ratio, 1.);
        pos.y *= -1.;

        self.relative_selected_tile = pos;
    }

    pub fn selected_tile(&self, camera: &Camera) -> Vector2<i32> {
        let mut pos = self.relative_selected_tile / camera.zoom + camera.pos;
        pos.x += if pos.x < 0. { -1. } else { 1. } * 0.5;
        pos.y += if pos.y < 0. { -1. } else { 1. } * 0.5;
        pos.map(|i| i as i32)
    }
}

fn render_chunk(
    chunk: &TileChunk,
    frame: &mut Frame,
    _assets: &Assets,
    chunk_coords: &ChunkCoord,
    tile_registry: &ClientTileRegistry,
    camera: &Camera,
) {
    let chunk_coords = chunk_coords.to_tile_coords();

    for (y, row) in chunk.tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let tile = tile_registry.get(*tile);
            let coords = chunk_coords + Vector2::new(x as i32, y as i32);
            frame.renderer.sprites.draw(
                tile.sprite,
                SpriteDrawParams {
                    transform: camera.view_transform() // Wtf?
                        * Matrix3::from_translation(coords.map(|i| i as f32))
                        * Matrix3::from_translation(Vector2::from_value(-0.5)), // center
                    ..Default::default()
                },
            );
        }
    }
}
