use network::{Client, Server};
use platform::{Event, Platform};
use underworld::{enable_backtrace, App};

fn main() {
    enable_backtrace();
    let platform = Platform::new_with_window();
    let mut server = App::<Server>::new();
    let mut client = App::<Client>::new(&platform);
    platform.run(&mut client, |client, e| {
        if e == Event::Update {
            server.update();
        }
        client.handle_event(e)
    });
}
