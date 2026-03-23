use crate::models::templates::{
    EdgeCondition, PipelineTemplate, TemplateEdge, TemplateNode, UiPosition,
};

use super::builtin_prompts::*;

/// Returns all built-in pipeline templates shipped with the application.
pub fn builtin_templates() -> Vec<PipelineTemplate> {
    vec![
        full_review_loop(),
        quick_fix(),
        research_only(),
        multi_brain_review(),
        security_audit(),
    ]
}

fn full_review_loop() -> PipelineTemplate {
    let node_ids = ["frl-analyse", "frl-review", "frl-implement", "frl-test"];
    PipelineTemplate {
        id: "full-review-loop".into(),
        name: "Full Review Loop".into(),
        description: "Analyse, review, implement, and test with iterative feedback.".into(),
        is_builtin: true,
        max_iterations: 5,
        stop_on_first_pass: true,
        nodes: vec![
            node("frl-analyse", "Analyse", "analyse", "claude", "opus", "A", ANALYSE_PROMPT, "text", 0.0),
            node("frl-review", "Review", "review", "claude", "opus", "A", REVIEW_PROMPT, "text", 320.0),
            node("frl-implement", "Implement", "implement", "claude", "sonnet", "B", IMPLEMENT_PROMPT, "code", 640.0),
            node("frl-test", "Test", "test", "claude", "sonnet", "B", TEST_PROMPT, "code", 960.0),
        ],
        edges: linear_edges(&node_ids),
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn quick_fix() -> PipelineTemplate {
    let node_ids = ["qf-implement", "qf-test"];
    PipelineTemplate {
        id: "quick-fix".into(),
        name: "Quick Fix".into(),
        description: "Fast single-pass implement and test cycle.".into(),
        is_builtin: true,
        max_iterations: 1,
        stop_on_first_pass: true,
        nodes: vec![
            node("qf-implement", "Implement", "implement", "claude", "sonnet", "A", QF_IMPLEMENT_PROMPT, "code", 0.0),
            node("qf-test", "Test", "test", "claude", "sonnet", "A", QF_TEST_PROMPT, "code", 320.0),
        ],
        edges: linear_edges(&node_ids),
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn research_only() -> PipelineTemplate {
    let node_ids = ["ro-analyse", "ro-review"];
    PipelineTemplate {
        id: "research-only".into(),
        name: "Research Only".into(),
        description: "Analyse and review without code changes.".into(),
        is_builtin: true,
        max_iterations: 1,
        stop_on_first_pass: true,
        nodes: vec![
            node("ro-analyse", "Analyse", "analyse", "claude", "opus", "A", RO_ANALYSE_PROMPT, "text", 0.0),
            node("ro-review", "Review", "review", "claude", "opus", "A", RO_REVIEW_PROMPT, "text", 320.0),
        ],
        edges: linear_edges(&node_ids),
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn multi_brain_review() -> PipelineTemplate {
    let node_ids = [
        "mbr-analyse",
        "mbr-review-gemini",
        "mbr-review-codex",
        "mbr-implement",
        "mbr-test",
    ];
    PipelineTemplate {
        id: "multi-brain-review".into(),
        name: "Multi-Brain Review".into(),
        description: "Three independent AI perspectives before implementation.".into(),
        is_builtin: true,
        max_iterations: 3,
        stop_on_first_pass: true,
        nodes: vec![
            node("mbr-analyse", "Analyse", "analyse", "claude", "opus", "A", ANALYSE_PROMPT, "text", 0.0),
            node(
                "mbr-review-gemini",
                "Review",
                "review",
                "gemini",
                "gemini-3.1-pro-preview",
                "B",
                MBR_GEMINI_REVIEW,
                "text",
                320.0,
            ),
            node(
                "mbr-review-codex",
                "Review 2",
                "review",
                "codex",
                "codex-5.3",
                "C",
                MBR_CODEX_REVIEW,
                "text",
                640.0,
            ),
            node(
                "mbr-implement",
                "Implement",
                "implement",
                "claude",
                "sonnet",
                "D",
                IMPLEMENT_PROMPT,
                "code",
                960.0,
            ),
            node("mbr-test", "Test", "test", "claude", "sonnet", "D", TEST_PROMPT, "code", 1280.0),
        ],
        edges: linear_edges(&node_ids),
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn security_audit() -> PipelineTemplate {
    let node_ids = [
        "sa-analyse",
        "sa-security-review",
        "sa-review",
        "sa-implement",
        "sa-test",
    ];
    PipelineTemplate {
        id: "security-audit".into(),
        name: "Security Audit".into(),
        description: "OWASP-focused security review with remediation.".into(),
        is_builtin: true,
        max_iterations: 2,
        stop_on_first_pass: true,
        nodes: vec![
            node("sa-analyse", "Analyse", "analyse", "claude", "opus", "A", ANALYSE_PROMPT, "text", 0.0),
            node(
                "sa-security-review",
                "Security Review",
                "review",
                "claude",
                "opus",
                "A",
                SECURITY_REVIEW_PROMPT,
                "text",
                320.0,
            ),
            node("sa-review", "Review", "review", "claude", "opus", "A", REVIEW_PROMPT, "text", 640.0),
            node("sa-implement", "Implement", "implement", "claude", "sonnet", "B", IMPLEMENT_PROMPT, "code", 960.0),
            node("sa-test", "Test", "test", "claude", "sonnet", "B", TEST_PROMPT, "code", 1280.0),
        ],
        edges: linear_edges(&node_ids),
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

#[allow(clippy::too_many_arguments)]
fn node(
    id: &str,
    label: &str,
    stage_type: &str,
    provider: &str,
    model: &str,
    session_group: &str,
    prompt_template: &str,
    execution_intent: &str,
    x: f64,
) -> TemplateNode {
    TemplateNode {
        id: id.into(),
        label: label.into(),
        stage_type: stage_type.into(),
        handler: stage_type.into(),
        provider: provider.into(),
        model: model.into(),
        session_group: session_group.into(),
        prompt_template: prompt_template.into(),
        enabled: true,
        execution_intent: execution_intent.into(),
        config: None,
        ui_position: UiPosition { x, y: 0.0 },
    }
}

fn linear_edges(node_ids: &[&str]) -> Vec<TemplateEdge> {
    node_ids
        .windows(2)
        .map(|pair| TemplateEdge {
            id: format!("{}-to-{}", pair[0], pair[1]),
            source_node_id: pair[0].into(),
            target_node_id: pair[1].into(),
            condition: EdgeCondition::Always,
            input_key: None,
            loop_control: false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::validation::validate_template;

    #[test]
    fn all_builtin_templates_pass_validation() {
        for template in builtin_templates() {
            let errors = validate_template(&template);
            assert!(
                errors.is_empty(),
                "Template '{}' failed validation: {:?}",
                template.id,
                errors,
            );
        }
    }

    #[test]
    fn builtin_templates_count() {
        assert_eq!(builtin_templates().len(), 5);
    }

    #[test]
    fn all_builtins_marked_as_builtin() {
        for template in builtin_templates() {
            assert!(template.is_builtin, "{} should be is_builtin", template.id);
        }
    }

    #[test]
    fn full_review_loop_has_expected_graph_shape() {
        let frl = full_review_loop();
        assert_eq!(frl.nodes.len(), 4);
        assert_eq!(frl.edges.len(), 3);
        assert_eq!(frl.nodes[0].stage_type, "analyse");
        assert_eq!(frl.nodes[1].stage_type, "review");
        assert_eq!(frl.nodes[2].stage_type, "implement");
        assert_eq!(frl.nodes[3].stage_type, "test");
        assert_eq!(frl.edges[0].source_node_id, "frl-analyse");
        assert_eq!(frl.edges[0].target_node_id, "frl-review");
    }
}
