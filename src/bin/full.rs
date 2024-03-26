use network::{Client, NetworkSide, Server};
use platform::{
    colored::Color,
    set_log_side,
    window::{WindowPlatform, WindowPlatformEvent},
};
use underworld::App;

fn set_common_log_side() {
    set_log_side("COMMON".to_string(), Color::Red);
}

fn main() {
    //enable_backtrace();
    set_common_log_side();
    let platform = WindowPlatform::new();
    Server::set_log_side();
    let mut server = App::<Server>::new();
    Client::set_log_side();
    let mut client = App::<Client>::new(&platform);
    set_common_log_side();
    platform.run(&mut client, |client, e| {
        if e == WindowPlatformEvent::Update {
            Server::set_log_side();
            server.update();
        }
        Client::set_log_side();
        client.handle_event(e);
        set_common_log_side();
    });
}
