//! Entry point nativo Rusdi webOS.
//! cargo build --release --target=armv7-unknown-linux-gnueabihf

use crate::error::Result;
use crate::lifecycle::LifeCycleManager;
use crate::logging::{init_logger, log_info, log_error};
use crate::settings::Settings;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let verbose = args.iter().any(|a| a == "-v" || a == "--verbose");

    let logger = init_logger(verbose);
    log_info!(logger, "=== Rusdi webOS v{} ===", env!("CARGO_PKG_VERSION"));
    log_info!(logger, "Target: {}", env!("CARGO_PKG_TARGET"));

    let settings = Settings::load().unwrap_or_default();
    log_info!(logger, "Settings: {:?}", settings);

    let mut lifecycle = LifeCycleManager::new(settings)?;
    lifecycle.setup_signal_handlers().await?;
    lifecycle.initialize().await?;

    let mut shutdown_rx = lifecycle.shutdown_tx.subscribe();
    tokio::select! {
        _ = shutdown_rx.changed() => { if *shutdown_rx.borrow() { log_info!(logger, "Shutdown"); } }
        _ = tokio::signal::ctrl_c() => { log_info!(logger, "Ctrl-C"); }
    }

    lifecycle.terminate().await?;
    log_info!(logger, "=== Rusdi webOS finalizado ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_settings() { let s = Settings::default(); assert!(s.hardware_acceleration); }
    #[test] fn test_keycodes() { use crate::input::keycodes::*; assert_eq!(keycode_to_action(461), RusdiAction::Back); }
}
