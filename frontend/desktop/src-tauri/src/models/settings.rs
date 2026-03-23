use serde::{Deserialize, Serialize};

/// Application-wide settings for EA Code v2.
/// Persisted to `~/.ea-code/settings.json`.
///
/// v2 fields use `#[serde(default)]` so that v1 settings files
/// deserialise without error before migration runs.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    // Pipeline defaults
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    #[serde(default = "default_true")]
    pub require_git: bool,
    #[serde(default = "default_true")]
    pub require_plan_approval: bool,
    #[serde(default = "default_plan_timeout")]
    pub plan_auto_approve_timeout_sec: u32,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    #[serde(default = "default_retry_count")]
    pub agent_retry_count: u32,
    #[serde(default = "default_timeout_ms")]
    pub agent_timeout_ms: u64,
    #[serde(default = "default_max_turns")]
    pub agent_max_turns: u32,

    // hive-api connection (v2 — safe to default when loading v1 settings)
    #[serde(default = "default_hive_host")]
    pub hive_api_host: String,
    #[serde(default = "default_hive_port")]
    pub hive_api_port: u16,
    #[serde(default = "default_pipeline_id")]
    pub default_pipeline_id: String,
    #[serde(default = "default_true")]
    pub auto_start_hive_api: bool,

    // Schema version for migration detection (absent in v1 → defaults to 1)
    #[serde(default = "default_settings_version_v1")]
    pub settings_version: u32,
}

fn default_max_iterations() -> u32 { 5 }
fn default_true() -> bool { true }
fn default_plan_timeout() -> u32 { 30 }
fn default_retention_days() -> u32 { 30 }
fn default_retry_count() -> u32 { 2 }
fn default_timeout_ms() -> u64 { 300_000 }
fn default_max_turns() -> u32 { 25 }
fn default_hive_host() -> String { "127.0.0.1".into() }
fn default_hive_port() -> u16 { 8000 }
fn default_pipeline_id() -> String { "full-review-loop".into() }
fn default_settings_version_v1() -> u32 { 1 }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            require_git: true,
            require_plan_approval: true,
            plan_auto_approve_timeout_sec: 30,
            retention_days: 30,
            agent_retry_count: 2,
            agent_timeout_ms: 300_000,
            agent_max_turns: 25,
            hive_api_host: "127.0.0.1".into(),
            hive_api_port: 8000,
            default_pipeline_id: "full-review-loop".into(),
            auto_start_hive_api: true,
            settings_version: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_have_sensible_values() {
        let settings = AppSettings::default();
        assert_eq!(settings.max_iterations, 5);
        assert_eq!(settings.hive_api_host, "127.0.0.1");
        assert_eq!(settings.hive_api_port, 8000);
        assert_eq!(settings.default_pipeline_id, "full-review-loop");
        assert!(settings.auto_start_hive_api);
        assert_eq!(settings.settings_version, 2);
        assert!(settings.require_plan_approval);
        assert_eq!(settings.agent_timeout_ms, 300_000);
    }

    #[test]
    fn settings_round_trip_through_json() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.max_iterations, settings.max_iterations);
        assert_eq!(restored.hive_api_port, settings.hive_api_port);
        assert_eq!(restored.settings_version, settings.settings_version);
    }

    #[test]
    fn settings_serialises_camel_case() {
        let json = serde_json::to_string(&AppSettings::default()).unwrap();
        assert!(json.contains("maxIterations"));
        assert!(json.contains("hiveApiHost"));
        assert!(json.contains("hiveApiPort"));
        assert!(json.contains("defaultPipelineId"));
        assert!(json.contains("autoStartHiveApi"));
        assert!(json.contains("settingsVersion"));
        assert!(!json.contains("max_iterations"));
        assert!(!json.contains("hive_api_host"));
    }

    #[test]
    fn v1_settings_deserialise_without_v2_fields() {
        // v1 settings: no hive-api fields, no settings_version
        let v1_json = r#"{
            "maxIterations": 3,
            "requireGit": false,
            "requirePlanApproval": true,
            "planAutoApproveTimeoutSec": 60,
            "retentionDays": 14,
            "agentRetryCount": 1,
            "agentTimeoutMs": 120000,
            "agentMaxTurns": 10
        }"#;
        let settings: AppSettings = serde_json::from_str(v1_json).unwrap();
        // v1 values preserved
        assert_eq!(settings.max_iterations, 3);
        assert!(!settings.require_git);
        assert_eq!(settings.agent_max_turns, 10);
        // v2 fields get safe defaults
        assert_eq!(settings.hive_api_host, "127.0.0.1");
        assert_eq!(settings.hive_api_port, 8000);
        assert_eq!(settings.default_pipeline_id, "full-review-loop");
        assert!(settings.auto_start_hive_api);
        // Missing settings_version defaults to 1 (indicating v1)
        assert_eq!(settings.settings_version, 1);
    }

    #[test]
    fn empty_json_object_deserialises_with_all_defaults() {
        let settings: AppSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.max_iterations, 5);
        assert_eq!(settings.settings_version, 1);
        assert_eq!(settings.hive_api_port, 8000);
    }
}
