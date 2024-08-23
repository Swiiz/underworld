use std::{collections::HashMap, net::SocketAddr};

use common::network::{proto::v1_network_protocol, AnyPacket, Protocol};
use remote::NetRemoteClient;
use serde::Serialize;
use uflow::{server::Server, SendMode};

pub mod remote;

pub struct NetworkServer {
    protocol: Protocol,
    socket: Server,
    pub clients: HashMap<SocketAddr, NetRemoteClient>,
}

impl NetworkServer {
    pub fn new() -> Self {
        let server_address = "127.0.0.1:8888";
        let config = uflow::server::Config {
            max_active_connections: 16,
            ..Default::default()
        };

        let network =
            Server::bind(server_address, config).expect("Failed to bind/configure socket");
        let clients = HashMap::new();

        Self {
            protocol: v1_network_protocol(),
            socket: network,
            clients,
        }
    }

    pub fn handle_packets(&mut self, handler: impl Fn(SocketAddr, AnyPacket)) {
        for event in self.socket.step() {
            match event {
                uflow::server::Event::Connect(client_address) => {
                    println!("Client connected: {:?}", client_address);
                    self.clients.insert(client_address, NetRemoteClient::new());
                }
                uflow::server::Event::Disconnect(client_address) => {
                    println!("Client disconnected: {:?}", client_address);
                    self.clients.remove(&client_address);
                }
                uflow::server::Event::Error(client_address, error) => {
                    println!(
                        "Client disconnected: {:?} with error: {:?}",
                        client_address, error
                    );
                    self.clients.remove(&client_address);
                }
                uflow::server::Event::Receive(client_address, packet_data) => {
                    if let Some(p) = self.protocol.decode(&packet_data) {
                        handler(client_address, p);
                    }
                }
            }
        }
    }

    pub fn send_to<'a, T: Serialize + 'static>(
        &mut self,
        packet: T,
        addrs: impl IntoIterator<Item = &'a SocketAddr>,
        mode: SendMode,
    ) {
        let data = self.protocol.encode(&packet);
        addrs
            .into_iter()
            .map(|addr| self.socket.client(addr).unwrap().borrow_mut())
            .for_each(|mut client| client.send(data.clone(), 0, mode));
    }

    pub fn broadcast<T: Serialize + 'static>(&mut self, packet: T, mode: SendMode) {
        let data = self.protocol.encode(&packet);
        self.clients
            .keys()
            .map(|addr| self.socket.client(addr).unwrap().borrow_mut())
            .for_each(|mut client| client.send(data.clone(), 0, mode));
    }

    pub fn flush(&mut self) {
        self.socket.flush();
    }
}
