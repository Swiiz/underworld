use std::{cell::OnceCell, sync::Arc, time::Instant};

use assets::ClientAssets;
use camera::Camera;
use cgmath::{Array, Vector2, Zero};
use common::{
    core::EntityKind,
    logger::{info, warn},
    network::proto::{
        extra::{CommonPing, ServerboundDisconnect},
        login::{ClientboundLoginSuccess, ServerboundLoginStart},
        play::{ClientboundRemoveEntity, ClientboundSetEntityPosition, ClientboundSpawnEntity},
        SyncComponentSelection,
    },
    utils::timer::Timer,
};
use ecs::{Entities, Entity, EntityHandle, Query};
use graphics::{
    color::Color3,
    sprite::{Sprite, SpriteDrawParams},
    text::{HorizontalAlign, Layout, Section, Text, VerticalAlign},
    Graphics,
};
use network::NetworkClient;
use platform::{AppLayer, PlatformHandle, PlatformInput};
use player::PlayerController;
use rendering::RenderData;
use state::ClientState;
use tilemap::ClientTileMap;
use winit::window::{Window, WindowAttributes, WindowId};

pub mod assets;
pub mod camera;
pub mod network;
pub mod platform;
pub mod player;
pub mod rendering;
pub mod state;
pub mod tilemap;

pub struct GameClient {
    config: GameClientConfig,
    window: Arc<Window>,
    graphics: Graphics,
    assets: ClientAssets,
    timer: Timer,
    network: NetworkClient,

    state: ClientState,
}

pub struct GameClientConfig {
    pub username: String,
}

impl GameClientConfig {
    pub fn default() -> Self {
        Self {
            username: "Noobie".to_string(),
        }
    }
}

impl AppLayer for GameClient {
    type Config = GameClientConfig;
    fn new(platform: PlatformHandle, config: Self::Config) -> Self {
        let timer = Timer::new();
        let window =
            platform.create_window(WindowAttributes::default().with_title("Underworld Client"));
        let assets = ClientAssets::load();
        let graphics = Graphics::new(window.inner_size(), window.clone(), assets.textures.iter());
        let mut network = NetworkClient::connect_to("127.0.0.1:8888");
        network.send(&ServerboundLoginStart {
            username: config.username.clone(),
        });
        let state = ClientState::Connecting;

        Self {
            config,
            window,
            graphics,
            assets,
            network,
            timer,
            state,
        }
    }

    fn render(&mut self, _: WindowId) {
        let dt = self.timer.render_dt();

        self.state.update_camera_pos();

        self.graphics.render(|mut frame| {
            self.state.render(&mut frame, &self.assets);

            frame.renderer.text.draw_section(
                Section::default()
                    .add_text(
                        Text::new(&format!("FPS: {}", 1. / dt))
                            .with_color(Color3::WHITE)
                            .with_scale(24.),
                    )
                    .with_layout(
                        Layout::default()
                            .h_align(HorizontalAlign::Left)
                            .v_align(VerticalAlign::Top),
                    )
                    .with_screen_position(Vector2::new(10.0, 10.0))
                    .to_owned(),
            )
        });
    }

    fn update(&mut self) {
        let dt = self.timer.update_dt();

        self.network.send(&CommonPing {
            time: Instant::now(),
        });

        self.network.handle_packets(|network, packet| {
            if let Some(CommonPing { time }) = packet.try_decode() {
                self.window
                    .set_title(&format!("Underworld - {:?}ms", time.elapsed().as_millis()));
            }

            match &mut self.state {
                ClientState::Connecting => {
                    if let Some(ClientboundLoginSuccess { terrain, ecs_state }) =
                        packet.try_decode()
                    {
                        info!("Successfully logged in!");

                        let mut entities = Entities::load(ecs_state);
                        load_entities_textures(&mut entities, &self.assets);

                        self.state = ClientState::Connected {
                            player_entity: OnceCell::new(),
                            camera: Camera::new(),
                            controller: PlayerController::default(),
                            terrain: ClientTileMap::new(terrain),
                            entities,
                        };
                    }
                }
                ClientState::Connected {
                    player_entity,
                    entities,
                    ..
                } => {
                    if let Some(ClientboundSpawnEntity { entity, state }) = packet.try_decode() {
                        let mut entity =
                            entities.load_entity::<SyncComponentSelection>(entity, state);

                        load_entity_textures(&mut entity, &self.assets);

                        // First spawned entity is player
                        player_entity.get_or_init(|| entity.id());
                    } else if let Some(ClientboundSetEntityPosition { entity, pos }) =
                        packet.try_decode()
                    {
                        let eid = entity.validate(&entities);
                        self.state.set_entity_position(eid, pos);
                    } else if let Some(ClientboundRemoveEntity { entity }) = packet.try_decode() {
                        if let Some(mut entity) = entities.edit(entity.validate(&entities)) {
                            entity.despawn();
                        } else {
                            warn!("Received entity despawn packet but entity was not found");
                        }
                    }
                }
            }
        });

        self.state.update(dt, &mut self.network);

        self.network.flush();
    }

    fn input(&mut self, _: WindowId, event: PlatformInput) {
        //TODO: move state input into here
        self.state.input(&event, self.window.inner_size());
    }

    fn exit(&mut self) {
        self.network.send(&ServerboundDisconnect::GameClosed);
        self.network.flush();
    }

    fn window_resized(&mut self) {
        self.graphics.resize(self.window.inner_size());
    }

    fn windows(&self) -> Vec<&Window> {
        vec![&self.window]
    }
}

fn load_entities_textures(entities: &mut Entities, assets: &ClientAssets) {
    for id in entities
        .with::<EntityKind>()
        .iter()
        .map(|e| e.id())
        .collect::<Vec<_>>()
    {
        load_entity_textures(&mut entities.edit(id).unwrap(), assets);
    }
}

fn load_entity_textures(entity: &mut EntityHandle, assets: &ClientAssets) {
    let kind = entity.get::<EntityKind>().unwrap().clone();
    match kind {
        EntityKind::Player => {
            entity.set(RenderData::new().with(
                Sprite {
                    pos: Vector2::zero(),
                    sheet: assets.textures.get_id("characters"),
                    size: Vector2::from_value(1),
                },
                SpriteDrawParams {
                    ..Default::default()
                },
            ));
        }
    }
}
