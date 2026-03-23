use std::collections::HashSet;

use super::templates::PipelineTemplate;

const VALID_EXECUTION_INTENTS: &[&str] = &["text", "code"];

/// Validates a pipeline template for correctness before save or execution.
/// Returns a list of error messages (empty = valid).
pub fn validate_template(template: &PipelineTemplate) -> Vec<String> {
    let mut errors = Vec::new();

    if template.name.trim().is_empty() {
        errors.push("Template name must not be empty.".into());
    }

    if template.max_iterations == 0 {
        errors.push("Max iterations must be at least 1.".into());
    }

    let enabled_stages: Vec<_> = template.stages.iter().filter(|s| s.enabled).collect();
    if enabled_stages.is_empty() {
        errors.push("At least one stage must be enabled.".into());
    }

    // Unique stage IDs
    let mut seen_ids = HashSet::new();
    for stage in &template.stages {
        if !seen_ids.insert(&stage.id) {
            errors.push(format!("Duplicate stage id: '{}'.", stage.id));
        }
    }

    // Contiguous 0-based positions
    let mut positions: Vec<u32> = template.stages.iter().map(|s| s.position).collect();
    positions.sort();
    for (i, pos) in positions.iter().enumerate() {
        if *pos != i as u32 {
            errors.push(format!(
                "Stage positions must be contiguous 0-based. Expected {} at index {}, got {}.",
                i, i, pos
            ));
            break;
        }
    }

    for stage in &template.stages {
        if stage.provider.trim().is_empty() {
            errors.push(format!("Stage '{}': provider must not be empty.", stage.id));
        }
        if stage.model.trim().is_empty() {
            errors.push(format!("Stage '{}': model must not be empty.", stage.id));
        }
        if stage.session_group.trim().is_empty() {
            errors.push(format!("Stage '{}': session_group must not be empty.", stage.id));
        }
        if stage.prompt_template.trim().is_empty() {
            errors.push(format!("Stage '{}': prompt_template must not be empty.", stage.id));
        }
        if !VALID_EXECUTION_INTENTS.contains(&stage.execution_intent.as_str()) {
            errors.push(format!(
                "Stage '{}': execution_intent must be 'text' or 'code', got '{}'.",
                stage.id, stage.execution_intent
            ));
        }
        if let Some(ref pg) = stage.parallel_group {
            if pg.trim().is_empty() {
                errors.push(format!(
                    "Stage '{}': parallel_group, when provided, must not be empty.",
                    stage.id
                ));
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::templates::{PipelineTemplate, StageDefinition};

    fn make_stage(id: &str, position: u32) -> StageDefinition {
        StageDefinition {
            id: id.into(),
            label: format!("Stage {}", id),
            stage_type: "analyse".into(),
            position,
            provider: "claude".into(),
            model: "opus".into(),
            session_group: "A".into(),
            parallel_group: None,
            prompt_template: "Do the thing: {{task}}".into(),
            enabled: true,
            execution_intent: "text".into(),
        }
    }

    fn make_template(stages: Vec<StageDefinition>) -> PipelineTemplate {
        PipelineTemplate {
            id: "tpl-test".into(),
            name: "Test Template".into(),
            description: "For testing".into(),
            is_builtin: false,
            max_iterations: 3,
            stop_on_first_pass: true,
            stages,
            created_at: "2026-03-23T12:00:00Z".into(),
            updated_at: "2026-03-23T12:00:00Z".into(),
        }
    }

    #[test]
    fn valid_template_passes() {
        let t = make_template(vec![make_stage("s1", 0), make_stage("s2", 1)]);
        assert!(validate_template(&t).is_empty());
    }

    #[test]
    fn empty_name_rejected() {
        let mut t = make_template(vec![make_stage("s1", 0)]);
        t.name = "  ".into();
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("name must not be empty")));
    }

    #[test]
    fn zero_max_iterations_rejected() {
        let mut t = make_template(vec![make_stage("s1", 0)]);
        t.max_iterations = 0;
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("Max iterations")));
    }

    #[test]
    fn no_enabled_stages_rejected() {
        let mut stage = make_stage("s1", 0);
        stage.enabled = false;
        let t = make_template(vec![stage]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("At least one stage")));
    }

    #[test]
    fn duplicate_stage_ids_rejected() {
        let t = make_template(vec![make_stage("s1", 0), make_stage("s1", 1)]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("Duplicate stage id")));
    }

    #[test]
    fn non_contiguous_positions_rejected() {
        let t = make_template(vec![make_stage("s1", 0), make_stage("s2", 2)]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("contiguous")));
    }

    #[test]
    fn empty_provider_rejected() {
        let mut stage = make_stage("s1", 0);
        stage.provider = "".into();
        let t = make_template(vec![stage]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("provider must not be empty")));
    }

    #[test]
    fn invalid_execution_intent_rejected() {
        let mut stage = make_stage("s1", 0);
        stage.execution_intent = "execute".into();
        let t = make_template(vec![stage]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("execution_intent must be")));
    }

    #[test]
    fn empty_parallel_group_rejected() {
        let mut stage = make_stage("s1", 0);
        stage.parallel_group = Some("  ".into());
        let t = make_template(vec![stage]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("parallel_group")));
    }

    #[test]
    fn empty_prompt_template_rejected() {
        let mut stage = make_stage("s1", 0);
        stage.prompt_template = "".into();
        let t = make_template(vec![stage]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("prompt_template must not be empty")));
    }
}
