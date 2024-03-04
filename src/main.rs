use platform::{Platform, WindowBuilder};
use underworld::App;

fn main() {
    let platform = Platform::new(WindowBuilder::new().with_title("Underworld"));

    let mut app = App::new(&platform);
    platform.run(&mut app, App::handle_event)
}
