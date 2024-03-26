use network::{Client, NetworkSide};
use platform::window::WindowPlatform;
use underworld::App;

fn main() {
    //enable_backtrace();
    Client::set_log_side();
    let platform = WindowPlatform::new();
    let mut app = App::<Client>::new(&platform);
    platform.run(&mut app, App::handle_event);
}
