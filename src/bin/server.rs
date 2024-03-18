use network::Server;
use underworld::{enable_backtrace, App};

fn main() {
    enable_backtrace();
    let mut app = App::<Server>::new();
    loop {
        app.update();
    }
}
