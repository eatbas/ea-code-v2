use super::HiveClient;
use serde::{Deserialize, Serialize};

/// CLI version information for a specific provider.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CliVersionInfo {
    pub provider: String,
    #[serde(default)]
    pub installed_version: Option<String>,
    #[serde(default)]
    pub latest_version: Option<String>,
    #[serde(default)]
    pub update_available: bool,
    #[serde(default)]
    pub cli_found: bool,
}

impl HiveClient {
    /// Get CLI version information for all providers.
    pub async fn get_cli_versions(&self) -> Result<Vec<CliVersionInfo>, String> {
        let url = format!("{}/v1/cli/versions", self.base_url());
        let resp = self
            .client()
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to get CLI versions: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "hive-api CLI versions request failed: HTTP {}",
                resp.status()
            ));
        }

        resp.json::<Vec<CliVersionInfo>>()
            .await
            .map_err(|e| format!("Failed to parse CLI versions response: {}", e))
    }

    /// Trigger a CLI version check for a specific provider.
    pub async fn check_cli_version(&self, provider: &str) -> Result<CliVersionInfo, String> {
        let url = format!("{}/v1/cli/versions/{}", self.base_url(), provider);
        let resp = self
            .client()
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to check CLI version for {}: {}", provider, e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "hive-api CLI version check failed for {}: HTTP {}",
                provider,
                resp.status()
            ));
        }

        resp.json::<CliVersionInfo>()
            .await
            .map_err(|e| format!("Failed to parse CLI version response: {}", e))
    }

    /// Trigger a CLI update for a specific provider.
    pub async fn update_cli(&self, provider: &str) -> Result<String, String> {
        let url = format!("{}/v1/cli/update/{}", self.base_url(), provider);
        let resp = self
            .client()
            .post(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to trigger CLI update for {}: {}", provider, e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "hive-api CLI update failed for {}: HTTP {}",
                provider,
                resp.status()
            ));
        }

        resp.text()
            .await
            .map_err(|e| format!("Failed to read CLI update response: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_version_info_parsing() {
        let json = r#"{
            "provider": "claude",
            "installedVersion": "1.2.3",
            "latestVersion": "1.3.0",
            "updateAvailable": true,
            "cliFound": true
        }"#;
        let info: CliVersionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.provider, "claude");
        assert_eq!(info.installed_version, Some("1.2.3".into()));
        assert_eq!(info.latest_version, Some("1.3.0".into()));
        assert!(info.update_available);
        assert!(info.cli_found);
    }

    #[test]
    fn cli_version_info_minimal() {
        let json = r#"{"provider": "gemini"}"#;
        let info: CliVersionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.provider, "gemini");
        assert!(info.installed_version.is_none());
        assert!(!info.update_available);
        assert!(!info.cli_found);
    }
}
