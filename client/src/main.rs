use client::{GameClient, GameClientConfig};

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let mut config = GameClientConfig::default();

    let args = std::env::args();

    if let Some(username) = args.skip(1).next() {
        config.username = username;
    } else {
        // Ask for username in terminal
        println!(">> Enter username: ");
        let mut buff = String::new();
        std::io::stdin()
            .read_line(&mut buff)
            .expect("Failed to read line");
        config.username = buff.trim().to_string();
    }

    client::platform::run_app::<GameClient>(config);
}
