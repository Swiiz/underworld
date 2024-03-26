use std::{
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    sync::Arc,
};

use assets::SpriteSheets;
use cgmath::{Array, Matrix3, Vector2, Zero};
use graphics::{
    color::Color3,
    sprite::{renderer::SpriteRenderer, Sprite, SpriteParams},
    Graphics,
};
use mods::ModLoader;
use network::{ctx::Network, Client, ClientOnly, NetworkSide, Server};
use platform::{
    debug, info,
    window::{Window, WindowPlatform, WindowPlatformEvent},
};
use protocol::{protocol, ClientPingPacket, SERVER_PORT};
use world::World;

use crate::{protocol::ServerPongPacket, world::ClientLoadWorldPacket};

pub mod assets;
mod mods;
pub mod protocol;
pub mod world;

pub struct App<S: NetworkSide> {
    window: ClientOnly<S, Arc<Window>>,
    graphics: ClientOnly<S, Graphics<'static>>,

    network: Network<S>,
    mods: ModLoader<S>,

    world: World<S>,
}

pub const REMOTE: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), SERVER_PORT));

impl App<Client> {
    pub fn new(platform: &WindowPlatform) -> Self {
        let window = platform.window.clone();
        let window_size = window.inner_size().into();

        let mut graphics = Graphics::new(window_size, window.clone());

        graphics
            .renderer
            .add_plugin(SpriteRenderer::<SpriteSheets>::new(
                &graphics.ctx,
                window_size,
            ));

        let mut network = Network::<Client>::new(protocol());
        network.set_connection(
            TcpStream::connect(&REMOTE).expect("Could not connect to remote server!"),
        );
        network.send(&[ClientPingPacket]);

        let mods = ModLoader::load();

        let world = World::<Client>::new();

        Self {
            window,
            graphics,
            network,
            mods,
            world,
        }
    }

    pub fn handle_event(&mut self, event: WindowPlatformEvent) {
        match event {
            WindowPlatformEvent::Update => self.update(),
            WindowPlatformEvent::Render => self.render(),
            WindowPlatformEvent::Resize => {
                self.graphics.resize(self.window.inner_size().into());
            }
        }
    }

    fn update(&mut self) {
        self.network.poll();
        self.network.on::<ServerPongPacket>(|network, _, _| {
            info!("Client successfully connected to server! Loading world...");
            network.send(&[ClientLoadWorldPacket]);
        });

        self.world.client_update(&mut self.network);
    }

    fn render(&mut self) {
        self.graphics.render(|frame| {
            self.world.render(frame);

            frame.draw(
                Sprite {
                    sheet: SpriteSheets::Characters,
                    position: Vector2::zero(),
                    size: Vector2::from_value(1),
                },
                SpriteParams {
                    transform: Matrix3::from_scale(0.1)
                        * Matrix3::from_translation(Vector2::from_value(-0.5)), // At -0.5, -0.5
                    tint: Color3::WHITE,
                    depth: 0.0,
                },
            );
        });
    }
}

impl App<Server> {
    pub fn new() -> Self {
        let mut network = Network::<Server>::new(protocol());
        let tcp_host_addr = format!("127.0.0.1:{}", SERVER_PORT);
        network
            .add_provider(TcpListener::bind(&tcp_host_addr).expect("Could not create tcp server!"));
        info!("Listening for tcp connections on: {tcp_host_addr}");

        let mods = ModLoader::load();

        let mut world = World::<Server>::new();
        world.server_generate();

        Self {
            window: PhantomData,
            graphics: PhantomData,
            network,
            mods,
            world,
        }
    }

    pub fn update(&mut self) {
        self.network.poll();

        self.network.on::<ClientPingPacket>(|network, _p, _conn| {
            debug!("Received ping from client!");
            network.send(&[ServerPongPacket], &network.all_connections())
        });

        self.world.server_update(&mut self.network);
    }
}

#[allow(unused)]
pub fn enable_backtrace() {
    std::env::set_var("RUST_BACKTRACE", "1");
}
