use crate::models::templates::{PipelineTemplate, StageDefinition};

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
    PipelineTemplate {
        id: "full-review-loop".into(),
        name: "Full Review Loop".into(),
        description: "Analyse, review, implement, and test with iterative feedback.".into(),
        is_builtin: true,
        max_iterations: 5,
        stop_on_first_pass: true,
        stages: vec![
            stage("frl-analyse", "Analyse", "analyse", 0, "claude", "opus", "A", ANALYSE_PROMPT, "text"),
            stage("frl-review", "Review", "review", 1, "claude", "opus", "A", REVIEW_PROMPT, "text"),
            stage("frl-implement", "Implement", "implement", 2, "claude", "sonnet", "B", IMPLEMENT_PROMPT, "code"),
            stage("frl-test", "Test", "test", 3, "claude", "sonnet", "B", TEST_PROMPT, "code"),
        ],
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn quick_fix() -> PipelineTemplate {
    PipelineTemplate {
        id: "quick-fix".into(),
        name: "Quick Fix".into(),
        description: "Fast single-pass implement and test cycle.".into(),
        is_builtin: true,
        max_iterations: 1,
        stop_on_first_pass: true,
        stages: vec![
            stage("qf-implement", "Implement", "implement", 0, "claude", "sonnet", "A", QF_IMPLEMENT_PROMPT, "code"),
            stage("qf-test", "Test", "test", 1, "claude", "sonnet", "A", QF_TEST_PROMPT, "code"),
        ],
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn research_only() -> PipelineTemplate {
    PipelineTemplate {
        id: "research-only".into(),
        name: "Research Only".into(),
        description: "Analyse and review without code changes.".into(),
        is_builtin: true,
        max_iterations: 1,
        stop_on_first_pass: true,
        stages: vec![
            stage("ro-analyse", "Analyse", "analyse", 0, "claude", "opus", "A", RO_ANALYSE_PROMPT, "text"),
            stage("ro-review", "Review", "review", 1, "claude", "opus", "A", RO_REVIEW_PROMPT, "text"),
        ],
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn multi_brain_review() -> PipelineTemplate {
    PipelineTemplate {
        id: "multi-brain-review".into(),
        name: "Multi-Brain Review".into(),
        description: "Three independent AI perspectives before implementation.".into(),
        is_builtin: true,
        max_iterations: 3,
        stop_on_first_pass: true,
        stages: vec![
            stage("mbr-analyse", "Analyse", "analyse", 0, "claude", "opus", "A", ANALYSE_PROMPT, "text"),
            stage(
                "mbr-review-gemini", "Review", "review", 1,
                "gemini", "gemini-3.1-pro-preview", "B", MBR_GEMINI_REVIEW, "text",
            ),
            stage(
                "mbr-review-codex", "Review 2", "review", 2,
                "codex", "codex-5.3", "C", MBR_CODEX_REVIEW, "text",
            ),
            stage("mbr-implement", "Implement", "implement", 3, "claude", "sonnet", "D", IMPLEMENT_PROMPT, "code"),
            stage("mbr-test", "Test", "test", 4, "claude", "sonnet", "D", TEST_PROMPT, "code"),
        ],
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn security_audit() -> PipelineTemplate {
    PipelineTemplate {
        id: "security-audit".into(),
        name: "Security Audit".into(),
        description: "OWASP-focused security review with remediation.".into(),
        is_builtin: true,
        max_iterations: 2,
        stop_on_first_pass: true,
        stages: vec![
            stage("sa-analyse", "Analyse", "analyse", 0, "claude", "opus", "A", ANALYSE_PROMPT, "text"),
            stage(
                "sa-security-review", "Security Review", "review", 1,
                "claude", "opus", "A", SECURITY_REVIEW_PROMPT, "text",
            ),
            stage("sa-review", "Review", "review", 2, "claude", "opus", "A", REVIEW_PROMPT, "text"),
            stage("sa-implement", "Implement", "implement", 3, "claude", "sonnet", "B", IMPLEMENT_PROMPT, "code"),
            stage("sa-test", "Test", "test", 4, "claude", "sonnet", "B", TEST_PROMPT, "code"),
        ],
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

#[allow(clippy::too_many_arguments)]
fn stage(
    id: &str,
    label: &str,
    stage_type: &str,
    position: u32,
    provider: &str,
    model: &str,
    session_group: &str,
    prompt_template: &str,
    execution_intent: &str,
) -> StageDefinition {
    StageDefinition {
        id: id.into(),
        label: label.into(),
        stage_type: stage_type.into(),
        position,
        provider: provider.into(),
        model: model.into(),
        session_group: session_group.into(),
        parallel_group: None,
        prompt_template: prompt_template.into(),
        enabled: true,
        execution_intent: execution_intent.into(),
    }
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
    fn full_review_loop_has_expected_stages() {
        let frl = full_review_loop();
        assert_eq!(frl.stages.len(), 4);
        assert_eq!(frl.stages[0].stage_type, "analyse");
        assert_eq!(frl.stages[1].stage_type, "review");
        assert_eq!(frl.stages[2].stage_type, "implement");
        assert_eq!(frl.stages[3].stage_type, "test");
    }
}
