use std::{net::TcpStream, sync::Arc};

use assets::SpriteSheets;
use cgmath::{Array, Matrix3, Vector2, Zero};
use commons::network::{Protocol, SERVER_PORT};
use graphics::{
    color::Color3,
    sprite::{renderer::SpriteRenderer, Sprite, SpriteParams},
    Graphics,
};
use network::client::NetworkClient;
use platform::{Event, Platform, Window};
use world::World;

pub mod assets;
pub mod world;

pub struct App {
    window: Arc<Window>,
    graphics: Graphics<'static>,

    network: NetworkClient<Protocol>,
    world: World,
}

impl App {
    pub fn new(platform: &Platform) -> Self {
        let window = platform.window.clone();
        let window_size = window.inner_size().into();

        let mut graphics = Graphics::new(window_size, window.clone());

        graphics
            .renderer
            .add_plugin(SpriteRenderer::<SpriteSheets>::new(
                &graphics.ctx,
                window_size,
            ));

        //TODO: Change remote!
        let tcp_host_addr = format!("127.0.0.1:{}", SERVER_PORT);
        let network = NetworkClient::new(
            TcpStream::connect(&tcp_host_addr).expect("Could not connect to network server!"),
        );
        println!("Connection established with remote tcp server: {tcp_host_addr}");

        let world = World::generate();

        Self {
            window,
            graphics,
            network,
            world,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Update => self.update(),
            Event::Render => self.render(),
            Event::Resize => {
                self.graphics.resize(self.window.inner_size().into());
            }
        }
    }

    fn update(&mut self) {
        self.network.update();
    }

    fn render(&mut self) {
        self.graphics.render(|frame| {
            frame.draw(
                Sprite {
                    sheet: SpriteSheets::Characters,
                    position: Vector2::zero(),
                    size: Vector2::from_value(1),
                },
                SpriteParams {
                    transform: Matrix3::from_translation(Vector2::from_value(-0.5)), // At -0.5, -0.5
                    tint: Color3::WHITE,
                    depth: 0.0,
                },
            );
        });
    }
}
