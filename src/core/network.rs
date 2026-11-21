//! Clientes de rede: SMB, NFS, HTTP, UPnP.

use crate::error::{RusdiError, Result};
use reqwest::Client;
use std::time::Duration;

pub struct NetworkClient {
    http: Client,
}

impl NetworkClient {
    pub fn new() -> Self {
        Self { http: Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build().unwrap() }
    }

    pub async fn http_get(&self, url: &str) -> Result<Vec<u8>> {
        let resp = self.http.get(url).send().await?;
        Ok(resp.bytes().await?.to_vec())
    }

    pub async fn http_head(&self, url: &str) -> Result<reqwest::header::HeaderMap> {
        let resp = self.http.head(url).send().await?;
        Ok(resp.headers().clone())
    }

    pub async fn smb_list(&self, share: &str, user: &str, pass: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }

    pub async fn upnp_discover(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}
