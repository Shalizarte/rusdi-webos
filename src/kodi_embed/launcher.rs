//! Lançador e gerenciador do processo Rusdi nativo.

use crate::error::{RusdiError, Result};
use crate::settings::Settings;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tokio::process::Child;
use tokio::time::{sleep, Duration, timeout};

pub struct RusdiLauncher {
    rusdi_binary: PathBuf,
    settings: Settings,
    child: Option<Child>,
    restart_count: u32,
    max_restarts: u32,
}

impl RusdiLauncher {
    pub fn new(settings: Settings) -> Result<Self> {
        let mut binary = PathBuf::from("/media/developer/apps/usr/palm/applications/org.xbmc.kodi/rusdi-webos");
        if !binary.exists() { binary = PathBuf::from("./rusdi-webos"); }
        Ok(Self { rusdi_binary: binary, settings, child: None, restart_count: 0, max_restarts: 5 })
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.child.is_some() { return Err(RusdiError::Config("Rusdi já está rodando".into())); }

        let mut cmd = Command::new(&self.rusdi_binary);
        cmd.args(&self.settings.rusdi_args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("HOME", "/media/developer/apps/usr/palm/applications/org.xbmc.kodi")
            .env("RUSDI_HOME", "/media/developer/apps/usr/palm/applications/org.xbmc.kodi")
            .env("PYTHONPATH", "/media/developer/apps/usr/palm/applications/org.xbmc.kodi/lib/python3.11/site-packages")
            .env("LD_LIBRARY_PATH", "/media/developer/apps/usr/palm/applications/org.xbmc.kodi/lib:/usr/lib");

        log_info!(crate::logging::init_logger(false), "Iniciando Rusdi: {:?} {}", self.rusdi_binary, self.settings.rusdi_args.join(" "));

        let child = tokio::process::Command::from(cmd)
            .spawn().map_err(|e| RusdiError::Config(e.to_string()))?;

        self.child = Some(child); self.restart_count = 0;
        self.wait_ready().await?;
        Ok(())
    }

    async fn wait_ready(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let url = "http://localhost:8080/jsonrpc";
        let start = std::time::Instant::now(); let max_wait = Duration::from_secs(30);

        while start.elapsed() < max_wait {
            let req = serde_json::json!({ "jsonrpc": "2.0", "method": "JSONRPC.Ping", "id": 1 });
            if let Ok(resp) = client.post(url).json(&req).send().await {
                if resp.status().is_success() {
                    let val: serde_json::Value = resp.json().await.unwrap_or_default();
                    if val.get("result").and_then(|v| v.as_str()) == Some("pong") {
                        log_info!(crate::logging::init_logger(false), "Rusdi pronto em {:.1}s", start.elapsed().as_secs_f32());
                        return Ok(());
                    }
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
        Err(RusdiError::Config("Rusdi não ficou pronto a tempo".into()))
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = tokio::process::Command::new("pkill")
                .args(["-TERM", "-f", "rusdi-webos"])
                .status().await;
            match timeout(Duration::from_secs(10), child.wait()).await {
                Ok(Ok(status)) => { log_info!(crate::logging::init_logger(false), "Rusdi finalizado: {:?}", status); }
                Ok(Err(e)) => { log_error!(crate::logging::init_logger(false), "Erro ao aguardar Rusdi: {}", e); }
                Err(_) => {
                    log_warn!(crate::logging::init_logger(false), "Timeout matando Rusdi, force kill");
                    let _ = tokio::process::Command::new("pkill")
                        .args(["-KILL", "-f", "rusdi-webos"]).status().await;
                }
            }
        }
        Ok(())
    }

    pub async fn restart(&mut self) -> Result<()> {
        log_warn!(crate::logging::init_logger(false), "Reiniciando Rusdi (tentativa {}/{})", self.restart_count + 1, self.max_restarts);
        self.stop().await?; sleep(Duration::from_secs(self.settings.restart_delay_secs)).await;
        self.restart_count += 1;
        if self.restart_count > self.max_restarts {
            return Err(RusdiError::Config("Máximo de reinícios atingido".into()));
        }
        self.start().await
    }

    pub async fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => { log_error!(crate::logging::init_logger(false), "Rusdi terminou inesperadamente: {:?}", status); self.child = None; false }
                Ok(None) => true, Err(e) => { log_error!(crate::logging::init_logger(false), "Erro ao verificar processo: {}", e); false }
            }
        } else { false }
    }

    pub async fn health_monitor(&mut self, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) -> Result<()> {
        while !*shutdown_rx.borrow() {
            if !self.is_running().await {
                if self.settings.auto_restart_on_crash {
                    log_warn!(crate::logging::init_logger(false), "Rusdi crashou, reiniciando...");
                    if let Err(e) = self.restart().await { log_error!(crate::logging::init_logger(false), "Falha ao reiniciar Rusdi: {}", e); break; }
                } else { log_error!(crate::logging::init_logger(false), "Rusdi parado e auto-restart desabilitado"); break; }
            }
            sleep(Duration::from_secs(5)).await;
        }
        Ok(())
    }
}
