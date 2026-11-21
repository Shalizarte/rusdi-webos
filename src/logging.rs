//! Logging estruturado para webOS (journald compatible).

use slog::{Drain, Logger, Level, o};
use slog_async::Async;
use slog_term::{FullFormat, TermDecorator};
use std::sync::{Mutex, OnceLock};

static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

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

    let _ = GLOBAL_LOGGER.set(logger.clone());
    slog::set_global_logger(logger.clone());
    logger
}

pub fn get_logger() -> &'static Logger {
    GLOBAL_LOGGER.get_or_init(|| init_logger(false))
}

#[macro_export]
macro_rules! log_info { ($($args:tt)*) => { slog::info!(crate::logging::get_logger(), $($args)*) }; }
#[macro_export]
macro_rules! log_warn { ($($args:tt)*) => { slog::warn!(crate::logging::get_logger(), $($args)*) }; }
#[macro_export]
macro_rules! log_error { ($($args:tt)*) => { slog::error!(crate::logging::get_logger(), $($args)*) }; }
#[macro_export]
macro_rules! log_debug { ($($args:tt)*) => { slog::debug!(crate::logging::get_logger(), $($args)*) }; }
