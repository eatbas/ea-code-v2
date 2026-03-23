use std::collections::{HashMap, HashSet};

/// Returns the strongly connected components of a directed graph.
pub(crate) fn strongly_connected_components<'a>(
    adjacency: &HashMap<&'a str, Vec<&'a str>>,
    reverse_adjacency: &HashMap<&'a str, Vec<&'a str>>,
) -> Vec<Vec<&'a str>> {
    let mut visited: HashSet<&'a str> = HashSet::new();
    let mut order = Vec::new();
    for &node in adjacency.keys() {
        dfs_order(node, adjacency, &mut visited, &mut order);
    }

    visited.clear();
    let mut components = Vec::new();
    while let Some(node) = order.pop() {
        if visited.contains(node) {
            continue;
        }
        let mut component = Vec::new();
        dfs_component(node, reverse_adjacency, &mut visited, &mut component);
        components.push(component);
    }
    components
}

/// Collects the finishing order for Kosaraju's algorithm.
fn dfs_order<'a>(
    node: &'a str,
    adjacency: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    order: &mut Vec<&'a str>,
) {
    if !visited.insert(node) {
        return;
    }
    if let Some(neighbours) = adjacency.get(node) {
        for &next in neighbours {
            dfs_order(next, adjacency, visited, order);
        }
    }
    order.push(node);
}

/// Collects one strongly connected component from the reversed graph.
fn dfs_component<'a>(
    node: &'a str,
    reverse_adjacency: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    component: &mut Vec<&'a str>,
) {
    if !visited.insert(node) {
        return;
    }
    component.push(node);
    if let Some(neighbours) = reverse_adjacency.get(node) {
        for &next in neighbours {
            dfs_component(next, reverse_adjacency, visited, component);
        }
    }
}

