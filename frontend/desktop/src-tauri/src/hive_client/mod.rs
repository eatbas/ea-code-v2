pub mod chat;
pub mod error;
pub mod health;
pub mod lifecycle;
pub mod providers;
pub mod sse;
pub mod streaming;
pub mod versions;

use reqwest::Client;
use std::time::Duration;

/// Central HTTP client for communicating with hive-api.
#[derive(Clone)]
pub struct HiveClient {
    client: Client,
    base_url: String,
}

impl HiveClient {
    pub fn new(host: &str, port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(600))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            client,
            base_url: format!("http://{}:{}", host, port),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
