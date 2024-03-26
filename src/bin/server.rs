use network::{NetworkSide, Server};

use underworld::App;

fn main() {
    //enable_backtrace();
    Server::set_log_side();
    let mut app = App::<Server>::new();
    loop {
        app.update();
    }
}
