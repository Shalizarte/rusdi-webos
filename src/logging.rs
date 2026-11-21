//! Logging estruturado para webOS (journald compatible).

use slog::{Drain, Logger, Level, o};
use slog_async::Async;
use slog_term::{FullFormat, TermDecorator};
use std::sync::Mutex;

pub fn init_logger(verbose: bool) -> Logger {
    let level = if verbose { Level::Debug } else { Level::Info };
    let decorator = TermDecorator::new().build();
    let drain = FullFormat::new(decorator).build().fuse();
    let drain = Async::new(drain).build().fuse();

    let logger = Logger::root(Mutex::new(drain).fuse(), o!(
        "app" => "rusdi-webos",
        "version" => env!("CARGO_PKG_VERSION"),
        "pid" => std::process::id(),
    ));

    slog::set_global_logger(logger.clone());
    logger
}

#[macro_export]
macro_rules! log_info { ($logger:expr, $($args:tt)*) => { slog::info!($logger, $($args)*) }; }
#[macro_export]
macro_rules! log_warn { ($logger:expr, $($args:tt)*) => { slog::warn!($logger, $($args)*) }; }
#[macro_export]
macro_rules! log_error { ($logger:expr, $($args:tt)*) => { slog::error!($logger, $($args)*) }; }
#[macro_export]
macro_rules! log_debug { ($logger:expr, $($args:tt)*) => { slog::debug!($logger, $($args)*) }; }
