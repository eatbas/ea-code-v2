use std::collections::HashSet;
use super::graph_validation::validate_graph_structure;
use super::templates::{PipelineTemplate, TemplateEdge, TemplateNode};
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
    if template.nodes.is_empty() {
        errors.push("Template must contain at least one node.".into());
    }
    let enabled_nodes = template.nodes.iter().filter(|node| node.enabled).count();
    if enabled_nodes == 0 {
        errors.push("At least one node must be enabled.".into());
    }
    let mut seen_node_ids: HashSet<&str> = HashSet::new();
    for node in &template.nodes {
        if node.id.trim().is_empty() {
            errors.push("Node id must not be empty.".into());
        } else if !seen_node_ids.insert(node.id.as_str()) {
            errors.push(format!("Duplicate node id: '{}'.", node.id));
        }
        validate_node(node, &mut errors);
    }
    let node_ids: HashSet<String> = template.nodes.iter().map(|node| node.id.clone()).collect();
    let mut seen_edge_ids: HashSet<String> = HashSet::new();
    for edge in &template.edges {
        validate_edge(edge, &node_ids, &mut seen_edge_ids, &mut errors);
    }
    if errors.is_empty() {
        validate_graph_structure(template, &mut errors);
    }
    errors
}
fn validate_node(node: &TemplateNode, errors: &mut Vec<String>) {
    if node.label.trim().is_empty() {
        errors.push(format!("Node '{}': label must not be empty.", node.id));
    }
    if node.stage_type.trim().is_empty() {
        errors.push(format!("Node '{}': stage_type must not be empty.", node.id));
    }
    if node.handler.trim().is_empty() {
        errors.push(format!("Node '{}': handler must not be empty.", node.id));
    }
    if node.provider.trim().is_empty() {
        errors.push(format!("Node '{}': provider must not be empty.", node.id));
    }
    if node.model.trim().is_empty() {
        errors.push(format!("Node '{}': model must not be empty.", node.id));
    }
    if node.session_group.trim().is_empty() {
        errors.push(format!("Node '{}': session_group must not be empty.", node.id));
    }
    if node.prompt_template.trim().is_empty() {
        errors.push(format!("Node '{}': prompt_template must not be empty.", node.id));
    }
    if !VALID_EXECUTION_INTENTS.contains(&node.execution_intent.as_str()) {
        errors.push(format!(
            "Node '{}': execution_intent must be 'text' or 'code', got '{}'.",
            node.id, node.execution_intent
        ));
    }
    if !node.ui_position.x.is_finite() || !node.ui_position.y.is_finite() {
        errors.push(format!(
            "Node '{}': ui_position coordinates must be finite numbers.",
            node.id
        ));
    }
}
fn validate_edge(
    edge: &TemplateEdge,
    node_ids: &HashSet<String>,
    seen_edge_ids: &mut HashSet<String>,
    errors: &mut Vec<String>,
) {
    if edge.id.trim().is_empty() {
        errors.push("Edge id must not be empty.".into());
    } else if !seen_edge_ids.insert(edge.id.clone()) {
        errors.push(format!("Duplicate edge id: '{}'.", edge.id));
    }
    if edge.source_node_id.trim().is_empty() {
        errors.push(format!("Edge '{}': source_node_id must not be empty.", edge.id));
    } else if !node_ids.contains(&edge.source_node_id) {
        errors.push(format!(
            "Edge '{}': source_node_id '{}' does not exist.",
            edge.id, edge.source_node_id
        ));
    }
    if edge.target_node_id.trim().is_empty() {
        errors.push(format!("Edge '{}': target_node_id must not be empty.", edge.id));
    } else if !node_ids.contains(&edge.target_node_id) {
        errors.push(format!(
            "Edge '{}': target_node_id '{}' does not exist.",
            edge.id, edge.target_node_id
        ));
    }
    if edge.source_node_id == edge.target_node_id && !edge.source_node_id.is_empty() {
        errors.push(format!("Edge '{}': self-referential edges are not allowed.", edge.id));
    }
    if let Some(input_key) = &edge.input_key {
        if input_key.trim().is_empty() {
            errors.push(format!(
                "Edge '{}': input_key, when provided, must not be empty.",
                edge.id
            ));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::templates::{
        EdgeCondition, PipelineTemplate, TemplateEdge, TemplateNode, UiPosition,
    };
    fn make_node(id: &str) -> TemplateNode {
        TemplateNode {
            id: id.into(),
            label: format!("Node {id}"),
            stage_type: "analyse".into(),
            handler: "analyse".into(),
            provider: "claude".into(),
            model: "opus".into(),
            session_group: "A".into(),
            prompt_template: "Do the thing: {{task}}".into(),
            enabled: true,
            execution_intent: "text".into(),
            config: None,
            ui_position: UiPosition { x: 0.0, y: 0.0 },
        }
    }
    fn make_edge(id: &str, source: &str, target: &str) -> TemplateEdge {
        TemplateEdge {
            id: id.into(),
            source_node_id: source.into(),
            target_node_id: target.into(),
            condition: EdgeCondition::Always,
            input_key: None,
            loop_control: false,
        }
    }
    fn make_loop_edge(id: &str, source: &str, target: &str) -> TemplateEdge {
        TemplateEdge {
            loop_control: true,
            ..make_edge(id, source, target)
        }
    }
    fn make_template(nodes: Vec<TemplateNode>, edges: Vec<TemplateEdge>) -> PipelineTemplate {
        PipelineTemplate {
            id: "tpl-test".into(),
            name: "Test Template".into(),
            description: "For testing".into(),
            is_builtin: false,
            max_iterations: 3,
            stop_on_first_pass: true,
            nodes,
            edges,
            created_at: "2026-03-23T12:00:00Z".into(),
            updated_at: "2026-03-23T12:00:00Z".into(),
        }
    }
    #[test]
    fn valid_template_passes() {
        let t = make_template(
            vec![make_node("n1"), make_node("n2")],
            vec![make_edge("e1", "n1", "n2")],
        );
        assert!(validate_template(&t).is_empty());
    }
    #[test]
    fn empty_name_rejected() {
        let mut t = make_template(vec![make_node("n1")], vec![]);
        t.name = "  ".into();
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("name must not be empty")));
    }
    #[test]
    fn zero_max_iterations_rejected() {
        let mut t = make_template(vec![make_node("n1")], vec![]);
        t.max_iterations = 0;
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("Max iterations")));
    }
    #[test]
    fn no_enabled_nodes_rejected() {
        let mut node = make_node("n1");
        node.enabled = false;
        let t = make_template(vec![node], vec![]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("At least one node")));
    }
    #[test]
    fn duplicate_node_ids_rejected() {
        let t = make_template(vec![make_node("n1"), make_node("n1")], vec![]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("Duplicate node id")));
    }
    #[test]
    fn empty_handler_rejected() {
        let mut node = make_node("n1");
        node.handler = "".into();
        let t = make_template(vec![node], vec![]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("handler must not be empty")));
    }
    #[test]
    fn invalid_execution_intent_rejected() {
        let mut node = make_node("n1");
        node.execution_intent = "execute".into();
        let t = make_template(vec![node], vec![]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("execution_intent must be")));
    }
    #[test]
    fn missing_edge_target_rejected() {
        let t = make_template(vec![make_node("n1")], vec![make_edge("e1", "n1", "n2")]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("target_node_id 'n2' does not exist")));
    }
    #[test]
    fn self_referential_edge_rejected() {
        let t = make_template(vec![make_node("n1")], vec![make_edge("e1", "n1", "n1")]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("self-referential")));
    }
    #[test]
    fn empty_edge_input_key_rejected() {
        let mut edge = make_edge("e1", "n1", "n2");
        edge.input_key = Some("   ".into());
        let t = make_template(vec![make_node("n1"), make_node("n2")], vec![edge]);
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("input_key")));
    }
    #[test]
    fn loop_control_cycle_allowed() {
        let t = make_template(
            vec![
                make_node("start"),
                make_node("n1"),
                make_node("n2"),
                make_node("end"),
            ],
            vec![
                make_edge("e0", "start", "n1"),
                make_loop_edge("e1", "n1", "n2"),
                make_loop_edge("e2", "n2", "n1"),
                make_edge("e3", "n2", "end"),
            ],
        );
        let errs = validate_template(&t);
        assert!(errs.is_empty(), "{errs:?}");
    }
    #[test]
    fn cyclic_graph_rejected() {
        let t = make_template(
            vec![
                make_node("start"),
                make_node("n1"),
                make_node("n2"),
                make_node("end"),
            ],
            vec![
                make_edge("e0", "start", "n1"),
                make_edge("e1", "n1", "n2"),
                make_loop_edge("e2", "n2", "n1"),
                make_edge("e3", "n2", "end"),
            ],
        );
        let errs = validate_template(&t);
        assert!(errs.iter().any(|e| e.contains("loop-control")));
    }
}
