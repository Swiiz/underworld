use std::{collections::HashMap, net::SocketAddr};

use common::network::{proto::network_protocol, AnyPacket, Protocol};
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
        println!("Server listening on {server_address}");
        let clients = HashMap::new();

        Self {
            protocol: network_protocol(),
            socket: network,
            clients,
        }
    }

    pub fn handle_packets(
        &mut self,
        mut handler: impl FnMut(SocketAddr, &mut NetRemoteClient, AnyPacket),
    ) {
        for event in self.socket.step() {
            match event {
                uflow::server::Event::Connect(client_address) => {
                    self.clients
                        .insert(client_address, NetRemoteClient::Connecting);
                }
                uflow::server::Event::Disconnect(client_address) => {
                    self.clients.remove(&client_address);
                }
                uflow::server::Event::Error(client_address, error) => {
                    println!("Client disconnected with error: {:?}", error);
                    self.clients.remove(&client_address);
                }
                uflow::server::Event::Receive(addr, packet_data) => {
                    if let Some(p) = self.protocol.decode(&packet_data) {
                        let remote = self.clients.get_mut(&addr).unwrap();
                        handler(addr, remote, p);
                    }
                }
            }
        }
    }

    pub fn send_to<'a, T: Serialize + 'static>(
        &mut self,
        packet: &T,
        addrs: impl IntoIterator<Item = &'a SocketAddr>,
        mode: SendMode,
    ) {
        let data = self.protocol.encode(packet);
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
