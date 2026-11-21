//! Persistência de configurações (settings.json) usando serde + dirs.

use crate::error::{RusdiError, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_true")]
    pub follow_iframes: bool,

    #[serde(default = "default_true")]
    pub show_embedded_player: bool,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub rusdi_args: Vec<String>,

    #[serde(default = "default_auto_restart")]
    pub auto_restart_on_crash: bool,

    #[serde(default = "default_restart_delay")]
    pub restart_delay_secs: u64,

    #[serde(default = "default_hw_accel")]
    pub hardware_acceleration: bool,

    #[serde(default = "default_cache_size")]
    pub cache_size_mb: u64,

    #[serde(default)]
    pub preferred_audio_language: String,

    #[serde(default)]
    pub preferred_subtitle_language: String,
}

fn default_true() -> bool { true }
fn default_log_level() -> String { "info".into() }
fn default_auto_restart() -> bool { true }
fn default_restart_delay() -> u64 { 5 }
fn default_hw_accel() -> bool { true }
fn default_cache_size() -> u64 { 512 }

impl Default for Settings {
    fn default() -> Self {
        Self {
            follow_iframes: true,
            show_embedded_player: true,
            log_level: "info".into(),
            rusdi_args: vec![],
            auto_restart_on_crash: true,
            restart_delay_secs: 5,
            hardware_acceleration: true,
            cache_size_mb: 512,
            preferred_audio_language: "por".into(),
            preferred_subtitle_language: "por".into(),
        }
    }
}

impl Settings {
    pub fn config_path() -> PathBuf {
        let mut path = config_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        path.push("rusdi-webos");
        fs::create_dir_all(&path).ok();
        path.join("settings.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }
}
