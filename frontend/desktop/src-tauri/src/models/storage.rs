use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionMeta {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunSummary {
    pub id: String,
    pub session_id: String,
    pub status: String,
    pub prompt: String,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    pub iteration_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost: Option<f64>,

    // v2 extensions (Option + default for backward compat with v1 runs)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_template_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_template_name: Option<String>,
    /// Maps session_group ("A", "B", ...) to the provider session reference
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub session_refs: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitBaseline {
    pub branch: String,
    pub commit: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_summary_v1_compat_missing_v2_fields() {
        let v1_json = r#"{
            "id": "run-001",
            "sessionId": "sess-001",
            "status": "completed",
            "prompt": "Fix the login bug",
            "startedAt": "2026-03-23T12:00:00Z",
            "endedAt": "2026-03-23T12:05:00Z",
            "iterationCount": 2,
            "totalTokens": 5000,
            "totalCost": 0.15
        }"#;
        let summary: RunSummary = serde_json::from_str(v1_json).unwrap();
        assert_eq!(summary.id, "run-001");
        assert_eq!(summary.iteration_count, 2);
        assert!(summary.pipeline_template_id.is_none());
        assert!(summary.pipeline_template_name.is_none());
        assert!(summary.session_refs.is_empty());
    }

    #[test]
    fn run_summary_v2_with_all_fields() {
        let v2_json = r#"{
            "id": "run-002",
            "sessionId": "sess-001",
            "status": "completed",
            "prompt": "Add dark mode",
            "startedAt": "2026-03-23T12:00:00Z",
            "iterationCount": 1,
            "pipelineTemplateId": "full-review-loop",
            "pipelineTemplateName": "Full Review Loop",
            "sessionRefs": {"A": "abc-123", "B": "def-456"}
        }"#;
        let summary: RunSummary = serde_json::from_str(v2_json).unwrap();
        assert_eq!(summary.pipeline_template_id, Some("full-review-loop".into()));
        assert_eq!(summary.session_refs.len(), 2);
        assert_eq!(summary.session_refs.get("A").unwrap(), "abc-123");
    }

    #[test]
    fn run_summary_omits_none_fields_in_json() {
        let summary = RunSummary {
            id: "r1".into(),
            session_id: "s1".into(),
            status: "running".into(),
            prompt: "test".into(),
            started_at: "2026-03-23T12:00:00Z".into(),
            ended_at: None,
            iteration_count: 0,
            total_tokens: None,
            total_cost: None,
            pipeline_template_id: None,
            pipeline_template_name: None,
            session_refs: HashMap::new(),
        };
        let json = serde_json::to_string(&summary).unwrap();
        // None fields are omitted, not emitted as null
        assert!(!json.contains("endedAt"));
        assert!(!json.contains("totalTokens"));
        assert!(!json.contains("pipelineTemplateId"));
        assert!(!json.contains("sessionRefs"));
        // Required fields are present
        assert!(json.contains("sessionId"));
        assert!(json.contains("startedAt"));
    }

    #[test]
    fn run_summary_serialises_camel_case() {
        let summary = RunSummary {
            id: "r1".into(),
            session_id: "s1".into(),
            status: "running".into(),
            prompt: "test".into(),
            started_at: "2026-03-23T12:00:00Z".into(),
            ended_at: Some("2026-03-23T12:01:00Z".into()),
            iteration_count: 1,
            total_tokens: Some(100),
            total_cost: Some(0.01),
            pipeline_template_id: Some("tpl-1".into()),
            pipeline_template_name: Some("Test".into()),
            session_refs: HashMap::from([("A".into(), "ref-1".into())]),
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("sessionId"));
        assert!(json.contains("startedAt"));
        assert!(json.contains("pipelineTemplateId"));
        assert!(!json.contains("session_id"));
        assert!(!json.contains("started_at"));
    }
}
