extern crate slog;
extern crate slog_term;
extern crate slog_async;

use std::sync::Mutex;
use slog::Drain;


lazy_static! {
    pub static ref LOGGER: Mutex<Vec<slog::Logger>> = Mutex::new(vec![]);
}

macro_rules! logger {
    () => ( LOGGER.lock().expect("Error while trying to acquire lock for LOGGER")[0] )
}

pub fn init_logger(log_level: slog::Level) {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog::LevelFilter::new(drain, log_level).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    LOGGER.lock().expect("Unable to acquire LOGGER lock in init_logger()").push(logger);
} 
