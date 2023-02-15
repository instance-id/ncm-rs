use std::env::var;
use std::fs::File;
use std::str::FromStr;
use log::{Level, Metadata, Record};
use log::{LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TerminalMode, TermLogger, WriteLogger};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub (crate) fn initialize(){
    let log_level = var("NCM_LOG_LEVEL").unwrap_or_else(|_| "Info".to_string());

    let mut temp_dir = std::env::temp_dir();
    temp_dir.push("ncm.log");

    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::from_str(&log_level).unwrap(), Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create(&temp_dir).unwrap()),
        ]
    ).expect("Could not initialize logger"); 
}
