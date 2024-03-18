use std::net::TcpListener;

use commons::network::{protocol, ClientPingPacket, SERVER_PORT};
use network::ctx::Network;

pub struct Server {
    network: Network<Server>,
}

impl Server {
    pub fn new() -> Self {
        let mut network = Network::<Server>::new(protocol());

        let tcp_host_addr = format!("127.0.0.1:{}", SERVER_PORT);
        network
            .add_provider(TcpListener::bind(&tcp_host_addr).expect("Could not create tcp server!"));
        println!("Listening for tcp connections on: {tcp_host_addr}");

        Self { network }
    }
}
