use super::HiveClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
    pub drones_booted: bool,
    #[serde(default)]
    pub providers: Vec<String>,
}

impl HiveClient {
    /// Single health check. Returns Ok(response) or Err(message).
    pub async fn check_health(&self) -> Result<HealthResponse, String> {
        let url = format!("{}/health", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("hive-api connection failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "hive-api health check failed: HTTP {}",
                resp.status()
            ));
        }

        resp.json::<HealthResponse>()
            .await
            .map_err(|e| format!("Failed to parse health response: {}", e))
    }

    /// Poll health until drones_booted is true or timeout.
    /// Returns Ok(HealthResponse) when ready, Err if timeout.
    pub async fn wait_until_ready(
        &self,
        poll_interval_ms: u64,
        timeout_ms: u64,
    ) -> Result<HealthResponse, String> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let interval = std::time::Duration::from_millis(poll_interval_ms);

        loop {
            if start.elapsed() > timeout {
                return Err(format!(
                    "hive-api did not become ready within {} ms",
                    timeout_ms
                ));
            }
            match self.check_health().await {
                Ok(health) if health.drones_booted => return Ok(health),
                Ok(_) => {}  // still booting
                Err(_) => {} // not reachable yet
            }
            tokio::time::sleep(interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_parsing() {
        let json = r#"{
            "status": "ok",
            "dronesBooted": true,
            "providers": ["claude", "gemini"]
        }"#;
        let resp: HealthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "ok");
        assert!(resp.drones_booted);
        assert_eq!(resp.providers, vec!["claude", "gemini"]);
    }

    #[test]
    fn test_health_response_without_providers() {
        let json = r#"{"status": "ok", "dronesBooted": false}"#;
        let resp: HealthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "ok");
        assert!(!resp.drones_booted);
        assert!(resp.providers.is_empty());
    }

    #[test]
    fn test_health_response_serialises_camel_case() {
        let resp = HealthResponse {
            status: "ok".into(),
            drones_booted: true,
            providers: vec!["claude".into()],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("dronesBooted"));
        assert!(!json.contains("drones_booted"));
    }
}
