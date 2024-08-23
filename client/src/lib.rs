use std::sync::Arc;

use assets::Assets;
use camera::Camera;
use cgmath::{Array, Vector2, Zero};
use common::{core::spatial::Position, utils::timer::Timer};
use common::{
    tilemap::{tile::Tile, TileMap},
    utils::registry::Registry,
};
use ecs::{Entities, Entity, EntityId};
use graphics::{
    ctx::GraphicsCtx,
    renderer::Renderer,
    sprite::{renderer::SpriteRenderer, Sprite, SpriteDrawParams},
    Graphics,
};
use network::NetworkClient;
use platform::{AppLayer, PlatformInput};
use player::PlayerController;
use rendering::{draw_entities, RenderData};
use tilemap::{ClientTile, ClientTileMap, ClientTileRegistry};
use winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub mod assets;
pub mod camera;
pub mod network;
pub mod platform;
pub mod player;
pub mod rendering;
pub mod tilemap;

pub struct GameClient {
    window: Arc<Window>,
    graphics: Graphics,
    assets: Assets,
    timer: Timer,

    network: NetworkClient,

    camera: Camera,
    player_entity: EntityId,
    controller: PlayerController,

    tiles: ClientTileRegistry,
    terrain: ClientTileMap,
    entities: Entities,
}

impl AppLayer for GameClient {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let timer = Timer::new();

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let ctx = GraphicsCtx::new(window.inner_size(), window.clone());
        let (assets, sprite_sheets_registry) = Assets::load();
        let graphics = Graphics {
            renderer: Renderer {
                sprites: SpriteRenderer::new(&ctx, window.inner_size(), &sprite_sheets_registry),
            },
            ctx,
        };

        let server_address = "127.0.0.1:8888";
        let network = NetworkClient::new(server_address);

        let camera = Camera::new();
        let controller = PlayerController::default();

        let mut tiles = Registry::new();

        let debug_tile = tiles.register(ClientTile {
            sprite: Sprite {
                pos: Vector2::zero(),
                sheet: assets.get_texture("debug"),
                size: Vector2::from_value(1),
            },
            common: Tile::default(),
        });

        let terrain = ClientTileMap::new(TileMap::new(Vector2::new(16, 16), debug_tile));

        let mut entities = Entities::new();

        let player_entity = entities
            .spawn()
            .set(Position(Vector2::zero()))
            .set(RenderData::new().with(
                Sprite {
                    pos: Vector2::zero(),
                    sheet: assets.get_texture("characters"),
                    size: Vector2::from_value(1),
                },
                SpriteDrawParams {
                    ..Default::default()
                },
            ))
            .id();

        Self {
            window,
            graphics,
            assets,
            network,
            timer,
            camera,
            controller,
            player_entity,
            tiles,
            terrain,
            entities,
        }
    }

    fn render(&mut self, _: WindowId) {
        let _dt = self.timer.render_dt();

        if let Some(player) = self.entities.get(&self.player_entity) {
            self.camera.pos = player.get::<Position>().unwrap().0;
        }

        self.graphics.render(|mut frame| {
            self.terrain
                .render(&mut frame, &self.assets, &self.tiles, &self.camera);

            draw_entities(&self.entities, &mut frame, &self.camera);
        });
    }

    fn update(&mut self) {
        let dt = self.timer.update_dt();

        self.network.handle_packets(|packet| {
            // handle packets
        });

        // Send data, update client application state
        // ...

        if let Some(player) = self.entities.get(&self.player_entity) {
            self.controller.update_entity(&player, dt);
        }

        self.network.flush();
    }

    fn input(&mut self, _: WindowId, event: PlatformInput) {
        self.terrain.input(&event, self.window.inner_size());
        self.controller.handle_platform_input(&event);
        self.camera.handle_scroll(&event);
    }

    fn exit(&mut self) {
        self.network.exit();
    }

    fn window_resized(&mut self) {
        self.graphics.resize(self.window.inner_size());
    }

    fn windows(&self) -> Vec<&Window> {
        vec![&self.window]
    }
}
