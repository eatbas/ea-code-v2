use std::collections::HashMap;

use crate::models::graph_analysis::strongly_connected_components;
use crate::models::templates::TemplateNode;

use super::IndexedEdge;

/// Builds the loop-component lookup used by the executor.
pub(crate) fn loop_component_map(
    nodes: &HashMap<String, TemplateNode>,
    edges: &[IndexedEdge],
) -> HashMap<String, usize> {
    let adjacency: HashMap<&str, Vec<&str>> = build_graph(nodes, edges, true);
    let reverse_adjacency: HashMap<&str, Vec<&str>> = build_graph(nodes, edges, false);
    let mut component_map = HashMap::new();
    for (index, component) in strongly_connected_components(&adjacency, &reverse_adjacency).into_iter().enumerate() {
        if component.len() > 1 {
            for node_id in component {
                component_map.insert(node_id.to_string(), index);
            }
        }
    }
    component_map
}

fn build_graph<'a>(
    nodes: &'a HashMap<String, TemplateNode>,
    edges: &'a [IndexedEdge],
    forward: bool,
) -> HashMap<&'a str, Vec<&'a str>> {
    let mut graph: HashMap<&'a str, Vec<&'a str>> = nodes.keys().map(|node_id| (node_id.as_str(), Vec::new())).collect();
    for edge in edges {
        let source = edge.edge.source_node_id.as_str();
        let target = edge.edge.target_node_id.as_str();
        let key = if forward { source } else { target };
        let value = if forward { target } else { source };
        graph.entry(key).or_default().push(value);
    }
    graph
}

