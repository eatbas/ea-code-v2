use super::HiveClient;
use serde::{Deserialize, Serialize};

/// Information about a configured provider in hive-api.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub available: bool,
}

/// Information about an active drone in hive-api.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DroneInfo {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub status: String,
}

impl HiveClient {
    /// List available providers from hive-api.
    pub async fn list_providers(&self) -> Result<Vec<ProviderInfo>, String> {
        let url = format!("{}/v1/providers", self.base_url());
        let resp = self
            .client()
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to list providers: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "hive-api providers request failed: HTTP {}",
                resp.status()
            ));
        }

        resp.json::<Vec<ProviderInfo>>()
            .await
            .map_err(|e| format!("Failed to parse providers response: {}", e))
    }

    /// List active drones from hive-api.
    pub async fn list_drones(&self) -> Result<Vec<DroneInfo>, String> {
        let url = format!("{}/v1/drones", self.base_url());
        let resp = self
            .client()
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to list drones: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "hive-api drones request failed: HTTP {}",
                resp.status()
            ));
        }

        resp.json::<Vec<DroneInfo>>()
            .await
            .map_err(|e| format!("Failed to parse drones response: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_info_parsing() {
        let json = r#"{
            "name": "claude",
            "displayName": "Claude",
            "models": ["opus", "sonnet"],
            "available": true
        }"#;
        let info: ProviderInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "claude");
        assert_eq!(info.display_name, "Claude");
        assert_eq!(info.models, vec!["opus", "sonnet"]);
        assert!(info.available);
    }

    #[test]
    fn drone_info_parsing() {
        let json = r#"{
            "id": "drone-1",
            "provider": "claude",
            "model": "opus",
            "status": "running"
        }"#;
        let info: DroneInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, "drone-1");
        assert_eq!(info.provider, "claude");
        assert_eq!(info.status, "running");
    }
}
