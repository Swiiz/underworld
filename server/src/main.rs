use std::env;

use server::Server;

fn main() {
    let mut server = Server::new();

    loop {
        server.update();
    }
}
