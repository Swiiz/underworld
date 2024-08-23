pub mod network;

use network::NetworkServer;

pub fn run_server() {
    let mut server = GameServer::new();

    loop {
        server.update();
        std::thread::sleep(std::time::Duration::from_secs_f32(1. / 60.));
    }
}

pub struct GameServer {
    network: NetworkServer,
}

impl GameServer {
    pub fn new() -> Self {
        let network = NetworkServer::new();

        Self { network }
    }

    pub fn update(&mut self) {
        self.network.handle_packets(|client_addr, packet| {
            // handle packets
        });

        // Send data, update server state

        self.network.flush();
    }
}
