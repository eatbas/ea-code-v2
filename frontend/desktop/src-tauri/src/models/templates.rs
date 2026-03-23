use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// A reusable pipeline configuration represented as a directed graph.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_builtin: bool,
    pub max_iterations: u32,
    pub stop_on_first_pass: bool,
    pub nodes: Vec<TemplateNode>,
    pub edges: Vec<TemplateEdge>,
    pub created_at: String,
    pub updated_at: String,
}

impl<'de> Deserialize<'de> for PipelineTemplate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PipelineTemplateWire::deserialize(deserializer)?;
        let (nodes, edges) = if !wire.nodes.is_empty() || !wire.edges.is_empty() {
            (wire.nodes, wire.edges)
        } else {
            migrate_stages_to_graph(wire.stages)
        };

        Ok(Self {
            id: wire.id,
            name: wire.name,
            description: wire.description,
            is_builtin: wire.is_builtin,
            max_iterations: wire.max_iterations,
            stop_on_first_pass: wire.stop_on_first_pass,
            nodes,
            edges,
            created_at: wire.created_at,
            updated_at: wire.updated_at,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PipelineTemplateWire {
    id: String,
    name: String,
    description: String,
    is_builtin: bool,
    max_iterations: u32,
    stop_on_first_pass: bool,
    #[serde(default)]
    nodes: Vec<TemplateNode>,
    #[serde(default)]
    edges: Vec<TemplateEdge>,
    #[serde(default)]
    stages: Vec<StageDefinition>,
    created_at: String,
    updated_at: String,
}

/// A single executable node in a template graph.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateNode {
    pub id: String,
    pub label: String,
    /// "analyse" | "review" | "implement" | "test" | "custom"
    pub stage_type: String,
    /// Execution handler identifier.
    pub handler: String,
    /// "claude" | "gemini" | "codex" | "kimi" | "copilot" | "opencode"
    pub provider: String,
    /// Exact model identifier, e.g. "opus", "gemini-3.1-pro-preview"
    pub model: String,
    /// Same group letter = resume within the same session. "A" | "B" | ...
    pub session_group: String,
    /// Full prompt with {{variables}} placeholders.
    pub prompt_template: String,
    pub enabled: bool,
    /// "text" (read-only analysis) | "code" (writes files)
    pub execution_intent: String,
    #[serde(default)]
    pub config: Option<Value>,
    pub ui_position: UiPosition,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EdgeCondition {
    #[default]
    Always,
    OnSuccess,
    OnFailure,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateEdge {
    pub id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    #[serde(default)]
    pub condition: EdgeCondition,
    #[serde(default)]
    pub input_key: Option<String>,
    #[serde(default)]
    pub loop_control: bool,
}

/// Legacy stage model retained only for backward-compatible deserialisation.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StageDefinition {
    pub id: String,
    pub label: String,
    pub stage_type: String,
    pub position: u32,
    pub provider: String,
    pub model: String,
    pub session_group: String,
    pub parallel_group: Option<String>,
    pub prompt_template: String,
    pub enabled: bool,
    pub execution_intent: String,
}

fn migrate_stages_to_graph(mut stages: Vec<StageDefinition>) -> (Vec<TemplateNode>, Vec<TemplateEdge>) {
    stages.sort_by(|a, b| a.position.cmp(&b.position).then_with(|| a.id.cmp(&b.id)));

    let nodes: Vec<TemplateNode> = stages
        .into_iter()
        .enumerate()
        .map(|(index, stage)| TemplateNode {
            id: stage.id,
            label: stage.label,
            stage_type: stage.stage_type.clone(),
            handler: stage.stage_type,
            provider: stage.provider,
            model: stage.model,
            session_group: stage.session_group,
            prompt_template: stage.prompt_template,
            enabled: stage.enabled,
            execution_intent: stage.execution_intent,
            config: None,
            ui_position: UiPosition {
                x: index as f64 * 320.0,
                y: 0.0,
            },
        })
        .collect();

    let edges = nodes
        .windows(2)
        .enumerate()
        .map(|(index, pair)| TemplateEdge {
            id: format!("edge-{}", index + 1),
            source_node_id: pair[0].id.clone(),
            target_node_id: pair[1].id.clone(),
            condition: EdgeCondition::Always,
            input_key: None,
            loop_control: false,
        })
        .collect();

    (nodes, edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_template_round_trip_graph() {
        let template = PipelineTemplate {
            id: "test-template-001".into(),
            name: "Test Pipeline".into(),
            description: "A pipeline for testing serialisation".into(),
            is_builtin: false,
            max_iterations: 3,
            stop_on_first_pass: true,
            nodes: vec![TemplateNode {
                id: "node-1".into(),
                label: "Analyse".into(),
                stage_type: "analyse".into(),
                handler: "analyse".into(),
                provider: "claude".into(),
                model: "opus".into(),
                session_group: "A".into(),
                prompt_template: "Analyse the following: {{prompt}}".into(),
                enabled: true,
                execution_intent: "text".into(),
                config: None,
                ui_position: UiPosition { x: 0.0, y: 0.0 },
            }],
            edges: vec![],
            created_at: "2026-03-23T12:00:00Z".into(),
            updated_at: "2026-03-23T12:00:00Z".into(),
        };

        let json = serde_json::to_string_pretty(&template).unwrap();
        let deserialized: PipelineTemplate = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.nodes.len(), 1);
        assert_eq!(deserialized.nodes[0].stage_type, "analyse");
        assert!(deserialized.edges.is_empty());
        assert!(!json.contains("\"stages\""));
    }

    #[test]
    fn legacy_stages_payload_migrates_to_graph() {
        let json = r#"{
            "id": "legacy-1",
            "name": "Legacy",
            "description": "Legacy stage list",
            "isBuiltin": false,
            "maxIterations": 2,
            "stopOnFirstPass": true,
            "stages": [
                {
                    "id": "s2",
                    "label": "Review",
                    "stageType": "review",
                    "position": 1,
                    "provider": "claude",
                    "model": "opus",
                    "sessionGroup": "A",
                    "parallelGroup": null,
                    "promptTemplate": "Review",
                    "enabled": true,
                    "executionIntent": "text"
                },
                {
                    "id": "s1",
                    "label": "Analyse",
                    "stageType": "analyse",
                    "position": 0,
                    "provider": "claude",
                    "model": "opus",
                    "sessionGroup": "A",
                    "parallelGroup": null,
                    "promptTemplate": "Analyse",
                    "enabled": true,
                    "executionIntent": "text"
                }
            ],
            "createdAt": "2026-03-23T12:00:00Z",
            "updatedAt": "2026-03-23T12:00:00Z"
        }"#;

        let template: PipelineTemplate = serde_json::from_str(json).unwrap();
        assert_eq!(template.nodes.len(), 2);
        assert_eq!(template.nodes[0].id, "s1");
        assert_eq!(template.nodes[0].handler, "analyse");
        assert_eq!(template.nodes[1].id, "s2");
        assert_eq!(template.edges.len(), 1);
        assert_eq!(template.edges[0].source_node_id, "s1");
        assert_eq!(template.edges[0].target_node_id, "s2");
        assert!(!template.edges[0].loop_control);

        let serialised = serde_json::to_string(&template).unwrap();
        assert!(serialised.contains("\"nodes\""));
        assert!(serialised.contains("\"edges\""));
        assert!(!serialised.contains("\"stages\""));
    }
}
