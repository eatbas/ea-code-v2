use serde::{Deserialize, Serialize};

/// A reusable pipeline configuration that defines the sequence of stages,
/// their providers, models, and execution settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_builtin: bool,
    pub max_iterations: u32,
    pub stop_on_first_pass: bool,
    pub stages: Vec<StageDefinition>,
    pub created_at: String,
    pub updated_at: String,
}

/// A single stage within a pipeline template.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StageDefinition {
    pub id: String,
    pub label: String,
    /// "analyse" | "review" | "implement" | "test" | "custom"
    pub stage_type: String,
    pub position: u32,
    /// "claude" | "gemini" | "codex" | "kimi" | "copilot" | "opencode"
    pub provider: String,
    /// Exact model identifier, e.g. "opus", "gemini-3.1-pro-preview"
    pub model: String,
    /// Same group letter = resume within the same session. "A" | "B" | ...
    pub session_group: String,
    /// Stages sharing the same parallel_group value run concurrently.
    pub parallel_group: Option<String>,
    /// Full prompt with {{variables}} placeholders.
    pub prompt_template: String,
    pub enabled: bool,
    /// "text" (read-only analysis) | "code" (writes files)
    pub execution_intent: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_template_round_trip() {
        let template = PipelineTemplate {
            id: "test-template-001".into(),
            name: "Test Pipeline".into(),
            description: "A pipeline for testing serialisation".into(),
            is_builtin: false,
            max_iterations: 3,
            stop_on_first_pass: true,
            stages: vec![
                StageDefinition {
                    id: "stage-1".into(),
                    label: "Analyse".into(),
                    stage_type: "analyse".into(),
                    position: 0,
                    provider: "claude".into(),
                    model: "opus".into(),
                    session_group: "A".into(),
                    parallel_group: None,
                    prompt_template: "Analyse the following: {{prompt}}".into(),
                    enabled: true,
                    execution_intent: "text".into(),
                },
                StageDefinition {
                    id: "stage-2".into(),
                    label: "Implement".into(),
                    stage_type: "implement".into(),
                    position: 1,
                    provider: "gemini".into(),
                    model: "gemini-3.1-pro-preview".into(),
                    session_group: "B".into(),
                    parallel_group: Some("p1".into()),
                    prompt_template: "Implement: {{plan}}".into(),
                    enabled: true,
                    execution_intent: "code".into(),
                },
            ],
            created_at: "2026-03-23T12:00:00Z".into(),
            updated_at: "2026-03-23T12:00:00Z".into(),
        };

        let json = serde_json::to_string_pretty(&template).unwrap();
        let deserialized: PipelineTemplate = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, template.id);
        assert_eq!(deserialized.name, template.name);
        assert_eq!(deserialized.is_builtin, template.is_builtin);
        assert_eq!(deserialized.max_iterations, template.max_iterations);
        assert_eq!(deserialized.stop_on_first_pass, template.stop_on_first_pass);
        assert_eq!(deserialized.stages.len(), 2);
        assert_eq!(deserialized.stages[0].provider, "claude");
        assert_eq!(deserialized.stages[1].parallel_group, Some("p1".into()));
        assert_eq!(deserialized.stages[1].execution_intent, "code");
    }

    #[test]
    fn stage_definition_camel_case_keys() {
        let stage = StageDefinition {
            id: "s1".into(),
            label: "Review".into(),
            stage_type: "review".into(),
            position: 0,
            provider: "claude".into(),
            model: "sonnet".into(),
            session_group: "A".into(),
            parallel_group: None,
            prompt_template: "Review: {{code}}".into(),
            enabled: true,
            execution_intent: "text".into(),
        };

        let json = serde_json::to_string(&stage).unwrap();
        assert!(json.contains("stageType"));
        assert!(json.contains("sessionGroup"));
        assert!(json.contains("parallelGroup"));
        assert!(json.contains("promptTemplate"));
        assert!(json.contains("executionIntent"));
        assert!(!json.contains("stage_type"));
    }
}
