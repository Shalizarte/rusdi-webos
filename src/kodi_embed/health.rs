//! Monitoramento de saúde avançado (CPU, memória, logs Rusdi).

use crate::error::{RusdiError, Result};
use std::fs;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

pub struct HealthMonitor {
    rusdi_log_path: PathBuf,
    last_log_size: u64,
    crash_keywords: Vec<String>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self { rusdi_log_path: PathBuf::from("/media/developer/apps/usr/palm/applications/org.xbmc.kodi/userdata/temp/rusdi.log"),
               last_log_size: 0,
               crash_keywords: vec![
                   "FATAL".into(), "CRASH".into(), "Segmentation fault".into(),
                   "Aborted".into(), "terminate called".into(),
               ] }
    }

    pub async fn check_logs(&mut self) -> Result<bool> {
        if !self.rusdi_log_path.exists() { return Ok(false); }
        let metadata = fs::metadata(&self.rusdi_log_path)?;
        let current_size = metadata.len();
        if current_size <= self.last_log_size { self.last_log_size = current_size; return Ok(false); }

        let content = fs::read_to_string(&self.rusdi_log_path)?;
        let new_content = &content[self.last_log_size as usize..];
        self.last_log_size = current_size;

        for line in new_content.lines() {
            for kw in &self.crash_keywords {
                if line.contains(kw) {
                    log_error!("Crash detectado no log Rusdi: {}", line);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub async fn run(&mut self, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) -> Result<()> {
        while !*shutdown_rx.borrow() {
            if self.check_logs().await? { return Ok(()); }
            sleep(Duration::from_secs(2)).await;
        }
        Ok(())
    }
}
