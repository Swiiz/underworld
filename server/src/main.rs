use std::net::TcpListener;

use commons::network::{Protocol, SERVER_PORT};
use network::server::NetworkServer;

fn main() {
    let mut network = NetworkServer::new(Protocol);

    let tcp_host_addr = format!("127.0.0.1:{}", SERVER_PORT);
    network.add_provider(TcpListener::bind(&tcp_host_addr).expect("Could not create tcp server!"));
    println!("Listening for tcp connections on: {tcp_host_addr}");

    loop {
        network.update();
    }
}
