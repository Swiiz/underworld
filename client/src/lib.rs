use std::sync::Arc;

use assets::{load_sprite_sheets, Assets, SpriteSheets};
use graphics::{
    ctx::GraphicsCtx,
    maths::{Array, Matrix3, Vector2, Zero},
    renderer::Renderer,
    sprite::{renderer::SpriteRenderer, Sprite, SpriteDrawParams},
    Graphics,
};
use platform::AppLayer;
use tilemap::{tile::Tile, TileMap, TileMapStorage};
use winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub mod assets;
pub mod platform;
pub mod tilemap;

pub struct Game {
    window: Arc<Window>,
    graphics: Graphics,
    assets: Assets,

    tile_maps: TileMapStorage,
}

impl AppLayer for Game {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let ctx = GraphicsCtx::new(window.inner_size(), window.clone());
        let (sprite_sheets, sprite_sheets_registry) = load_sprite_sheets();
        let graphics = Graphics {
            renderer: Renderer {
                sprites: SpriteRenderer::new(&ctx, window.inner_size(), &sprite_sheets_registry),
            },
            ctx,
        };

        let mut tile_maps = TileMapStorage::new();

        let debug_tile = tile_maps.tile_registry.add_tile(Tile {
            sprite: Sprite {
                position: Vector2::zero(),
                sheet: sprite_sheets.debug,
                size: Vector2::from_value(1),
            },
        });

        tile_maps.add_tile_map(TileMap::new(Vector2::new(16, 16), debug_tile));

        Self {
            window,
            graphics,
            assets: Assets { sprite_sheets },

            tile_maps,
        }
    }

    fn render(&mut self, _: WindowId) {
        self.graphics.render(|mut frame| {
            self.tile_maps.render_selected(&mut frame, &self.assets);

            frame.renderer.sprites.draw(
                Sprite {
                    position: Vector2::zero(),
                    sheet: self.assets.sprite_sheets.characters,
                    size: Vector2::from_value(1),
                },
                SpriteDrawParams {
                    transform: Matrix3::from_scale(0.1)
                        * Matrix3::from_translation(Vector2::from_value(-0.5)),
                    ..Default::default()
                },
            );
        });
    }

    fn window_resized(&mut self) {
        self.graphics.resize(self.window.inner_size());
    }

    fn windows(&self) -> Vec<&Window> {
        vec![&self.window]
    }
}
