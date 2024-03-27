use std::sync::RwLock;

use colored::{Color, Colorize};
use log::{Level, LevelFilter, Log};

pub mod headless;
pub mod window;

pub use colored;
pub use log::{debug, error, info, trace, warn};
static LOGGER: Logger = Logger;
static LOGGER_SIDE: RwLock<(String, Color)> = RwLock::new((String::new(), Color::White));

pub const IGNORE_LIST: &'static [&'static str] = &["wgpu", "naga"];

pub(crate) fn init_logger() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .expect("Could not set logger!")
}

pub fn set_log_side(name: String, color: Color) {
    *LOGGER_SIDE.write().expect("Could not change log side!") = (name, color);
}
pub(crate) fn get_log_side_prefix() -> String {
    let logger_side = LOGGER_SIDE.read().expect("Could not read logger side!");
    let mut r = String::new();
    if logger_side.0.len() > 0 {
        r = format!("[{}]", logger_side.0.color(logger_side.1));
    }
    r
}

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &log::Record) {
        let md = record.metadata();
        if self.enabled(md) {
            if let Some(path) = record.module_path() {
                for e in IGNORE_LIST {
                    if path.contains(e) {
                        return;
                    }
                }
            }

            let level = record.level().to_string();
            let level = match record.level() {
                Level::Trace => level.magenta(),
                Level::Debug => level.bright_green(),
                Level::Info => level.bright_blue(),
                Level::Warn => level.yellow(),
                Level::Error => level.red(),
            };

            let mut log_origin = String::new();
            if let Some(file) = record.file() {
                log_origin += &format!("{file}:");
                if let Some(line) = record.line() {
                    log_origin += &line.to_string();
                }
            }

            println!(
                "{} {} {} - {}",
                get_log_side_prefix(),
                log_origin,
                level,
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
