use std::net::TcpListener;

use commons::network::{protocol, ClientPingPacket, SERVER_PORT};
use network::{
    commons::{HandlePacket, ServerSide},
    Network,
};

pub struct Server {
    network: Network<ServerSide>,
}

impl Server {
    pub fn new() -> Self {
        let mut network = Network::<ServerSide>::new(protocol());

        let tcp_host_addr = format!("127.0.0.1:{}", SERVER_PORT);
        network
            .add_provider(TcpListener::bind(&tcp_host_addr).expect("Could not create tcp server!"));
        println!("Listening for tcp connections on: {tcp_host_addr}");

        Self { network }
    }

    fn on_ping(&mut self, packet: &ClientPingPacket) {
        println!("pong!")
    }

    pub fn update(&mut self) {
        self.network.poll();

        self.network.on(|p| self.on_ping(p));
    }
}
