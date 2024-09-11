use colored::Colorize;
use log::{Level, LevelFilter, Log};

pub use log::{debug, error, info, trace, warn};

static LOGGER: Logger = Logger;
pub const IGNORE_LIST: &'static [&'static str] = &["wgpu", "naga"];

pub fn init_logger() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .expect("Could not set logger!")
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
            #[cfg(debug_assertions)]
            {
                if let Some(file) = record.file() {
                    log_origin += &format!("{file}:");
                    if let Some(line) = record.line() {
                        log_origin += &line.to_string();
                    }
                }
                log_origin += " ";
            }

            println!("{}{} - {}", log_origin, level, record.args());
        }
    }

    fn flush(&self) {}
}
