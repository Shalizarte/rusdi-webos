//! Cliente JSON-RPC para comunicação com Kodi (HTTP + WebSocket).

use crate::error::{RusdiError, Result};
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::ws_client::WsClient;
use jsonrpsee::core::client::ClientT;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String, pub method: String,
    pub params: Option<Value>, pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String, pub result: Option<Value>,
    pub error: Option<JsonRpcError>, pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32, pub message: String, pub data: Option<Value>,
}

pub struct KodiJsonRpc {
    http_client: HttpClient,
    ws_client: Option<Arc<Mutex<WsClient>>>,
    base_url: String,
    next_id: Arc<Mutex<u64>>,
}

impl KodiJsonRpc {
    pub fn new_http(host: &str, port: u16) -> Result<Self> {
        let url = format!("http://{}:{}/jsonrpc", host, port);
        let client = HttpClient::builder()
            .request_timeout(Duration::from_secs(5))
            .build(url.parse()?)?;
        Ok(Self { http_client: client, ws_client: None,
                 base_url: format!("http://{}:{}", host, port), next_id: Arc::new(Mutex::new(1)) })
    }

    pub async fn connect_ws(&mut self, host: &str, port: u16) -> Result<()> {
        let url = format!("ws://{}:{}/jsonrpc", host, port);
        let client = WsClient::builder().build(url.parse()?).await?;
        self.ws_client = Some(Arc::new(Mutex::new(client)));
        Ok(())
    }

    async fn next_id(&self) -> u64 {
        let mut id = self.next_id.lock().await;
        let current = *id; *id += 1; current
    }

    pub async fn call(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.next_id().await;
        let req = JsonRpcRequest { jsonrpc: "2.0".into(), method: method.into(), params, id };
        let resp: JsonRpcResponse = self.http_client.request("POST", "", req).await?;
        if let Some(err) = resp.error {
            return Err(RusdiError::JsonRpc(format!("{}: {}", err.code, err.message)));
        }
        Ok(resp.result.unwrap_or(Value::Null))
    }

    pub async fn execute_action(&self, action: &str) -> Result<()> {
        let params = serde_json::json!({ "action": action });
        self.call("Input.ExecuteAction", Some(params)).await?;
        Ok(())
    }

    pub async fn back(&self) -> Result<()> { self.execute_action("Back").await }
    pub async fn home(&self) -> Result<()> { self.execute_action("Home").await }
    pub async fn play_pause(&self) -> Result<()> { self.execute_action("PlayPause").await }
    pub async fn stop(&self) -> Result<()> { self.execute_action("Stop").await }
    pub async fn volume_up(&self) -> Result<()> { self.execute_action("VolumeUp").await }
    pub async fn volume_down(&self) -> Result<()> { self.execute_action("VolumeDown").await }
    pub async fn mute(&self) -> Result<()> { self.execute_action("Mute").await }
    pub async fn quit(&self) -> Result<()> { self.call("Application.Quit", None).await?; Ok(()) }
    pub async fn ping(&self) -> Result<bool> {
        match self.call("JSONRPC.Ping", None).await { Ok(val) => Ok(val == "pong"), Err(_) => Ok(false) }
    }
    pub async fn get_active_player(&self) -> Result<Option<Value>> {
        let params = serde_json::json!({ "properties": ["playerid", "type", "speed"] });
        let result = self.call("Player.GetActivePlayers", Some(params)).await?;
        if let Some(arr) = result.as_array() { Ok(arr.first().cloned()) } else { Ok(None) }
    }
    pub async fn subscribe_notifications(&self, method: &str) -> Result<()> {
        if let Some(ws) = &self.ws_client {
            let mut client = ws.lock().await;
            let params = serde_json::json!({ "method": method });
            client.request("JSONRPC.Subscribe", params).await?;
        }
        Ok(())
    }
}
