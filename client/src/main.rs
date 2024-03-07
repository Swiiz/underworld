use client::App;
use platform::Platform;

fn main() {
    let platform = Platform::new_with_window();
    let mut app = App::new(&platform);
    platform.run(&mut app, App::handle_event);
}
