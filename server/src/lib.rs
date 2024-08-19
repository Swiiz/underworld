use std::{collections::HashSet, net::SocketAddr};

use uflow::server::Server;

pub fn run_server() {
    let mut server = GameServer::new();

    loop {
        server.update();
    }
}

pub struct GameServer {
    network: Server,
    clients: HashSet<SocketAddr>,
}

impl GameServer {
    pub fn new() -> Self {
        let server_address = "127.0.0.1:8888";
        let config = uflow::server::Config {
            max_active_connections: 16,
            ..Default::default()
        };

        let network =
            Server::bind(server_address, config).expect("Failed to bind/configure socket");
        let clients = HashSet::new();

        Self { network, clients }
    }

    pub fn update(&mut self) {
        for event in self.network.step() {
            match event {
                uflow::server::Event::Connect(client_address) => {
                    println!("Client connected: {:?}", client_address);
                    self.clients.insert(client_address);
                }
                uflow::server::Event::Disconnect(client_address) => {
                    println!("Client disconnected: {:?}", client_address);
                    self.clients.remove(&client_address);
                }
                uflow::server::Event::Error(client_address, error) => {
                    // TODO: Handle connection error
                }
                uflow::server::Event::Receive(client_address, packet_data) => {
                    // TODO: Handle incoming packet
                }
            }
        }

        // Send data, update server state

        self.network.flush();

        std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
    }
}
