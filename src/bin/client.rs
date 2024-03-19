use network::Client;
use platform::Platform;
use underworld::{enable_backtrace, App};

fn main() {
    //enable_backtrace();
    let platform = Platform::new_with_window();
    let mut app = App::<Client>::new(&platform);
    platform.run(&mut app, App::handle_event);
}
