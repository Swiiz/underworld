use network::Server;
use underworld::App;

fn main() {
    let mut app = App::<Server>::new();
    loop {
        app.update();
    }
}
