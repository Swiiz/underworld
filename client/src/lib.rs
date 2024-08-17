use std::sync::Arc;

use assets::{load_sprite_sheets, Assets};
use cgmath::{Array, Matrix3, Vector2, Zero};
use core::{
    rendering::{draw_entities, RenderData},
    spatial::Position,
};
use ecs::{Entities, Query};
use graphics::{
    ctx::GraphicsCtx,
    renderer::Renderer,
    sprite::{renderer::SpriteRenderer, Sprite, SpriteDrawParams},
    Graphics,
};
use platform::AppLayer;
use player::{PlayerController, PlayerTag};
use tilemap::{tile::Tile, TileMap, TileMapStorage};
use timer::Timer;
use winit::{
    event::DeviceEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub mod assets;
pub mod core;
pub mod platform;
pub mod player;
pub mod tilemap;
pub mod timer;

pub struct Game {
    window: Arc<Window>,
    graphics: Graphics,
    assets: Assets,
    timer: Timer,

    controller: PlayerController,

    tile_maps: TileMapStorage,
    entities: Entities,
}

impl AppLayer for Game {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let timer = Timer::new();

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

        let controller = PlayerController::default();

        let mut tile_maps = TileMapStorage::new();

        let debug_tile = tile_maps.tile_registry.add_tile(Tile {
            sprite: Sprite {
                position: Vector2::zero(),
                sheet: sprite_sheets.debug,
                size: Vector2::from_value(1),
            },
        });

        tile_maps.add_tile_map(TileMap::new(Vector2::new(16, 16), debug_tile));

        let mut entities = Entities::new();

        entities
            .spawn()
            .set(PlayerTag)
            .set(Position(Vector2::zero()))
            .set(RenderData::new().with(
                Sprite {
                    position: Vector2::zero(),
                    sheet: sprite_sheets.characters,
                    size: Vector2::from_value(1),
                },
                SpriteDrawParams {
                    transform: Matrix3::from_scale(0.1)
                        * Matrix3::from_translation(Vector2::from_value(-0.5)),
                    ..Default::default()
                },
            ));

        Self {
            window,
            graphics,
            assets: Assets { sprite_sheets },
            timer,
            controller,
            tile_maps,
            entities,
        }
    }

    fn render(&mut self, _: WindowId) {
        let _dt = self.timer.render_dt();
        self.graphics.render(|mut frame| {
            self.tile_maps.render_selected(&mut frame, &self.assets);

            draw_entities(&self.entities, &mut frame);
        });
    }

    fn update(&mut self) {
        let dt = self.timer.update_dt();
        for e in self.entities.with::<PlayerTag>().iter() {
            self.controller.update_player_entity(e, dt)
        }
    }

    fn input(&mut self, event: DeviceEvent) {
        self.controller.handle_platform_input(&event);
    }

    fn window_resized(&mut self) {
        self.graphics.resize(self.window.inner_size());
    }

    fn windows(&self) -> Vec<&Window> {
        vec![&self.window]
    }
}
