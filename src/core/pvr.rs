//! PVR / Live TV backend agnóstico.

use crate::error::{RusdiError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvrChannel {
    pub id: String, pub name: String, pub number: u32,
    pub icon: Option<String>, pub stream_url: String,
    pub genre: Option<String>, pub is_radio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpgEvent {
    pub channel_id: String, pub title: String, pub description: String,
    pub start_time: i64, pub end_time: i64, pub genre: Option<String>,
}

pub trait PvrBackend: Send + Sync {
    fn name(&self) -> &str;
    fn connect(&mut self, config: &HashMap<String, String>) -> Result<()>;
    fn get_channels(&self) -> Result<Vec<PvrChannel>>;
    fn get_epg(&self, channel_id: &str, start: i64, end: i64) -> Result<Vec<EpgEvent>>;
    fn open_stream(&self, channel_id: &str) -> Result<String>;
    fn close(&mut self) -> Result<()>;
}

pub struct IptvSimpleBackend { channels: Vec<PvrChannel> }
impl IptvSimpleBackend {
    pub fn new() -> Self { Self { channels: vec![] } }
}
impl PvrBackend for IptvSimpleBackend {
    fn name(&self) -> &str { "IPTV Simple" }
    fn connect(&mut self, config: &HashMap<String, String>) -> Result<()> { Ok(()) }
    fn get_channels(&self) -> Result<Vec<PvrChannel>> { Ok(self.channels.clone()) }
    fn get_epg(&self, _ch: &str, _s: i64, _e: i64) -> Result<Vec<EpgEvent>> { Ok(vec![]) }
    fn open_stream(&self, channel_id: &str) -> Result<String> {
        self.channels.iter().find(|c| c.id == channel_id).map(|c| c.stream_url.clone())
            .ok_or(RusdiError::NotFound("Channel not found".into()))
    }
    fn close(&mut self) -> Result<()> { Ok(()) }
}

pub struct PvrManager {
    backends: HashMap<String, Box<dyn PvrBackend>>,
    active_backend: Option<String>,
}

impl PvrManager {
    pub fn new() -> Self { Self { backends: HashMap::new(), active_backend: None } }
    pub fn register(&mut self, backend: Box<dyn PvrBackend>) {
        self.backends.insert(backend.name().to_string(), backend);
    }
    pub fn set_active(&mut self, name: &str) -> Result<()> {
        if self.backends.contains_key(name) { self.active_backend = Some(name.into()); Ok(()) }
        else { Err(RusdiError::NotFound(format!("Backend {}", name))) }
    }
    pub fn get_channels(&self) -> Result<Vec<PvrChannel>> {
        if let Some(name) = &self.active_backend {
            self.backends[name].get_channels()
        } else { Err(RusdiError::Config("No active PVR backend".into())) }
    }
}
