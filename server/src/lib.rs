pub mod assets;
pub mod network;
pub mod state;

use assets::ServerAssets;
use common::{
    network::proto::login::{ClientboundLoginSuccess, ServerboundLoginStart},
    utils::timer::Timer,
};
use network::{remote::NetRemoteClient, NetworkServer};
use state::ServerState;
use uflow::SendMode;

pub fn run_server() {
    let mut server = GameServer::new();

    loop {
        server.update();
        std::thread::sleep(std::time::Duration::from_secs_f32(1. / 60.));
    }
}

pub struct GameServer {
    assets: ServerAssets,
    timer: Timer,
    network: NetworkServer,

    state: ServerState,
}

impl GameServer {
    pub fn new() -> Self {
        let assets = ServerAssets::load();
        let timer = Timer::new();
        let network = NetworkServer::new();
        let state = ServerState::new(&assets);

        Self {
            assets,
            timer,
            network,
            state,
        }
    }

    pub fn update(&mut self) {
        let mut to_sync = Vec::new();
        self.network
            .handle_packets(|addr, client, packet| match client {
                &mut NetRemoteClient::Connecting => {
                    if let Some(ServerboundLoginStart { username }) = packet.try_decode() {
                        println!("Client connected: {:?}", username);
                        *client = NetRemoteClient::Online { username };
                        to_sync.push(addr);
                    }
                }
                NetRemoteClient::Online { .. } => {}
            });

        let full_sync_packet = ClientboundLoginSuccess {
            ecs_state: self.state.common.entities.save(),
        };
        for addr in to_sync {
            self.network
                .send_to(&full_sync_packet, &[addr], SendMode::Reliable);
        }

        // Send data, update server state

        self.network.flush();
    }
}
