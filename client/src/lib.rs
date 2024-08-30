use std::sync::Arc;

use assets::ClientAssets;
use camera::Camera;
use common::{
    network::proto::login::{ClientboundLoginSuccess, ServerboundLoginStart},
    state::CommonState,
    utils::timer::Timer,
};
use ecs::Entities;
use graphics::Graphics;
use network::NetworkClient;
use platform::{AppLayer, PlatformHandle, PlatformInput};
use player::PlayerController;
use state::ClientState;
use uflow::SendMode;
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
        network.send(
            &ServerboundLoginStart {
                username: config.username.clone(),
            },
            SendMode::Reliable,
        );
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
        let _dt = self.timer.render_dt();

        self.state.update_camera_pos();

        self.graphics.render(|mut frame| {
            self.state.render(&mut frame, &self.assets);
        });
    }

    fn update(&mut self) {
        let dt = self.timer.update_dt();

        self.network.handle_packets(|packet| match self.state {
            ClientState::Connecting => {
                if let Some(ClientboundLoginSuccess { ecs_state }) = packet.try_decode() {
                    println!("Successfully logged in!");

                    self.state = ClientState::Connected {
                        camera: Camera::new(),
                        controller: PlayerController::default(),
                        common: CommonState {
                            //terrain: ,
                            entities: Entities::load(ecs_state),
                        },
                    };
                }
            }
            _ => unimplemented!(),
        });

        // Send data, update client application state
        // ...

        self.state.update_player(dt);
        self.state.update_world(dt);

        self.network.flush();
    }

    fn input(&mut self, _: WindowId, event: PlatformInput) {
        //TODO: move state input into here
        self.state.input(&event, self.window.inner_size());
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
