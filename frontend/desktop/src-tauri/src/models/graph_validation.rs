use std::collections::{HashMap, HashSet};

use super::graph_analysis::strongly_connected_components;
use super::templates::PipelineTemplate;
pub fn validate_graph_structure(template: &PipelineTemplate, errors: &mut Vec<String>) {
    let enabled_nodes: Vec<_> = template.nodes.iter().filter(|n| n.enabled).collect();
    if enabled_nodes.is_empty() {
        return;
    }
    let mut enabled = HashMap::new();
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut reverse_adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in &enabled_nodes {
        let node_id = node.id.as_str();
        enabled.insert(node_id, (0usize, 0usize));
        adjacency.insert(node_id, Vec::new());
        reverse_adjacency.insert(node_id, Vec::new());
    }
    let mut enabled_edges: Vec<(&str, &str, bool)> = Vec::new();
    for edge in &template.edges {
        let source_enabled = enabled.contains_key(edge.source_node_id.as_str());
        let target_enabled = enabled.contains_key(edge.target_node_id.as_str());
        if source_enabled && !target_enabled {
            errors.push(format!(
                "Edge '{}': enabled source '{}' cannot target disabled node '{}'.",
                edge.id, edge.source_node_id, edge.target_node_id
            ));
        }
        if !(source_enabled && target_enabled) {
            continue;
        }
        if let Some((_, out_degree)) = enabled.get_mut(edge.source_node_id.as_str()) {
            *out_degree += 1;
        }
        if let Some((in_degree, _)) = enabled.get_mut(edge.target_node_id.as_str()) {
            *in_degree += 1;
        }
        adjacency
            .entry(edge.source_node_id.as_str())
            .or_default()
            .push(edge.target_node_id.as_str());
        reverse_adjacency
            .entry(edge.target_node_id.as_str())
            .or_default()
            .push(edge.source_node_id.as_str());
        enabled_edges.push((
            edge.source_node_id.as_str(),
            edge.target_node_id.as_str(),
            edge.loop_control,
        ));
    }
    let entry_count = enabled.values().filter(|(in_d, _)| *in_d == 0).count();
    if entry_count == 0 {
        errors.push("Template must have at least one enabled entry node.".into());
    }
    let terminal_count = enabled.values().filter(|(_, out_d)| *out_d == 0).count();
    if terminal_count == 0 {
        errors.push("Template must have at least one enabled terminal node.".into());
    }
    if enabled.len() > 1 {
        for node in &enabled_nodes {
            if let Some((in_degree, out_degree)) = enabled.get(node.id.as_str()) {
                if *in_degree == 0 && *out_degree == 0 {
                    errors.push(format!(
                        "Enabled node '{}' is orphaned (no inbound or outbound enabled edges).",
                        node.id
                    ));
                }
            }
        }
    }
    if let Some(error) = find_cycle_error(
        template.max_iterations,
        &adjacency,
        &reverse_adjacency,
        &enabled_edges,
    ) {
        errors.push(error);
    }
}
fn find_cycle_error(
    max_iterations: u32,
    adjacency: &HashMap<&str, Vec<&str>>,
    reverse_adjacency: &HashMap<&str, Vec<&str>>,
    edges: &[(&str, &str, bool)],
) -> Option<String> {
    let components = strongly_connected_components(adjacency, reverse_adjacency);
    if !components.iter().any(|component| component.len() > 1) {
        return None;
    }
    if max_iterations <= 1 {
        return Some(
            "Template graph contains a cycle, but max_iterations must be greater than 1 to allow loop-control cycles.".into(),
        );
    }
    for component in components.into_iter().filter(|component| component.len() > 1) {
        let component_nodes: HashSet<&str> = component.iter().copied().collect();
        if edges.iter().any(|(source, target, loop_control)| {
            component_nodes.contains(source)
                && component_nodes.contains(target)
                && !*loop_control
        }) {
            return Some(
                "Template graph contains a cycle that is not fully marked as loop-control.".into(),
            );
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::templates::{
        EdgeCondition, PipelineTemplate, TemplateEdge, TemplateNode, UiPosition,
    };
    fn node(id: &str, enabled: bool) -> TemplateNode {
        TemplateNode {
            id: id.into(),
            label: id.into(),
            stage_type: "analyse".into(),
            handler: "analyse".into(),
            provider: "claude".into(),
            model: "opus".into(),
            session_group: "A".into(),
            prompt_template: "{{task}}".into(),
            enabled,
            execution_intent: "text".into(),
            config: None,
            ui_position: UiPosition { x: 0.0, y: 0.0 },
        }
    }
    fn edge(id: &str, source: &str, target: &str) -> TemplateEdge {
        TemplateEdge {
            id: id.into(),
            source_node_id: source.into(),
            target_node_id: target.into(),
            condition: EdgeCondition::Always,
            input_key: None,
            loop_control: false,
        }
    }
    fn loop_edge(id: &str, source: &str, target: &str) -> TemplateEdge {
        TemplateEdge {
            loop_control: true,
            ..edge(id, source, target)
        }
    }
    fn template(nodes: Vec<TemplateNode>, edges: Vec<TemplateEdge>) -> PipelineTemplate {
        PipelineTemplate {
            id: "tpl".into(),
            name: "T".into(),
            description: "D".into(),
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
    fn allows_loop_control_cycle_with_entry_and_terminal_nodes() {
        let t = template(
            vec![
                node("start", true),
                node("a", true),
                node("b", true),
                node("end", true),
            ],
            vec![
                edge("e1", "start", "a"),
                loop_edge("e2", "a", "b"),
                loop_edge("e3", "b", "a"),
                edge("e4", "b", "end"),
            ],
        );
        let mut errors = Vec::new();
        validate_graph_structure(&t, &mut errors);
        assert!(errors.is_empty(), "{errors:?}");
    }
    #[test]
    fn rejects_cycle_without_loop_control_edges() {
        let t = template(
            vec![
                node("start", true),
                node("a", true),
                node("b", true),
                node("end", true),
            ],
            vec![
                edge("e1", "start", "a"),
                edge("e2", "a", "b"),
                loop_edge("e3", "b", "a"),
                edge("e4", "b", "end"),
            ],
        );
        let mut errors = Vec::new();
        validate_graph_structure(&t, &mut errors);
        assert!(errors.iter().any(|e| e.contains("loop-control")));
    }
    #[test]
    fn rejects_loop_control_cycle_when_max_iterations_is_one() {
        let mut t = template(
            vec![
                node("start", true),
                node("a", true),
                node("b", true),
                node("end", true),
            ],
            vec![
                edge("e1", "start", "a"),
                loop_edge("e2", "a", "b"),
                loop_edge("e3", "b", "a"),
                edge("e4", "b", "end"),
            ],
        );
        t.max_iterations = 1;
        let mut errors = Vec::new();
        validate_graph_structure(&t, &mut errors);
        assert!(errors.iter().any(|e| e.contains("greater than 1")));
    }
    #[test]
    fn rejects_enabled_to_disabled_target() {
        let t = template(
            vec![node("a", true), node("b", false)],
            vec![edge("e1", "a", "b")],
        );
        let mut errors = Vec::new();
        validate_graph_structure(&t, &mut errors);
        assert!(errors.iter().any(|e| e.contains("cannot target disabled node")));
    }
}
