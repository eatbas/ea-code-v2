use std::collections::{HashMap, HashSet, VecDeque};

use tokio::task::JoinSet;

use crate::models::templates::{EdgeCondition, PipelineTemplate, TemplateNode};

use super::{
    GraphExecutionSummary, IndexedEdge, NodeExecutionInput, NodeExecutionResult, NodeExecutionState,
    NodeOutcome, NodeRunner, NodeInboundInput,
};
use super::topology::loop_component_map;

#[derive(Debug, Clone, Copy)]
struct NodePlan {
    initial_required: usize,
    subsequent_required: usize,
    loop_node: bool,
}

/// Executes the graph using bounded re-arming for validated loop-control cycles.
pub async fn execute_graph(template: &PipelineTemplate, runner: NodeRunner) -> Result<GraphExecutionSummary, String> {
    let nodes: Vec<TemplateNode> = template.nodes.iter().filter(|node| node.enabled).cloned().collect();
    if nodes.is_empty() {
        return Err("Template has no enabled nodes.".into());
    }

    let node_ids: HashSet<String> = nodes.iter().map(|node| node.id.clone()).collect();
    let edges: Vec<IndexedEdge> = template
        .edges
        .iter()
        .enumerate()
        .filter(|(_, edge)| node_ids.contains(&edge.source_node_id) && node_ids.contains(&edge.target_node_id))
        .map(|(index, edge)| IndexedEdge { index, edge: edge.clone() })
        .collect();

    let rank: HashMap<String, usize> = nodes.iter().enumerate().map(|(index, node)| (node.id.clone(), index)).collect();
    let node_by_id: HashMap<String, TemplateNode> = nodes.into_iter().map(|node| (node.id.clone(), node)).collect();
    let incoming = grouped_edges(&edges, true);
    let outgoing = grouped_edges(&edges, false);
    let loop_components = loop_component_map(&node_by_id, &edges);
    let plans = build_plans(&node_by_id, &incoming, &loop_components);

    let mut unresolved: HashMap<String, usize> = node_by_id
        .keys()
        .map(|node_id| {
            let required = plans.get(node_id).map(|plan| plan.initial_required).unwrap_or_default();
            (node_id.clone(), required)
        })
        .collect();
    let mut executions: HashMap<String, u32> = node_by_id.keys().map(|node_id| (node_id.clone(), 0)).collect();
    let mut active_inputs: HashMap<String, Vec<NodeInboundInput>> = HashMap::new();
    let mut pending: HashSet<String> = node_by_id.keys().cloned().collect();
    let mut ready = sort_ids(
        unresolved
            .iter()
            .filter(|(node_id, count)| {
                **count == 0 && incoming.get(node_id.as_str()).map(|edges| edges.is_empty()).unwrap_or(true)
            })
            .map(|(node_id, _)| node_id.clone())
            .collect(),
        &rank,
    );

    let mut summary = GraphExecutionSummary::default();
    let mut wave: u32 = 0;

    while !pending.is_empty() {
        if ready.is_empty() {
            return Err(format!(
                "Graph deadlock or cycle detected for nodes: {}",
                sort_ids(pending.iter().cloned().collect(), &rank).join(", ")
            ));
        }

        let mut set: JoinSet<(String, NodeExecutionResult)> = JoinSet::new();
        for node_id in &ready {
            let node = node_by_id.get(node_id).ok_or_else(|| format!("Missing node '{node_id}'"))?.clone();
            let input = NodeExecutionInput { node: node.clone(), inbound: active_inputs.remove(node_id).unwrap_or_default(), wave };
            let run = runner.clone();
            set.spawn(async move { (node.id, run(input).await) });
        }
        ready.clear();

        let mut results = Vec::new();
        while let Some(item) = set.join_next().await {
            results.push(item.map_err(|error| format!("Wave task failed: {error}"))?);
        }
        results.sort_by_key(|(node_id, _)| *rank.get(node_id).unwrap_or(&usize::MAX));

        let mut activations: Vec<String> = Vec::new();
        for (node_id, result) in results {
            summary.failed |= result.outcome == NodeOutcome::Failure;
            summary.cancelled |= result.outcome == NodeOutcome::Cancelled;
            summary.execution_order.push(node_id.clone());
            summary.node_states.insert(
                node_id.clone(),
                NodeExecutionState {
                    outcome: result.outcome,
                    output: result.output.clone(),
                    error: result.error.clone(),
                    wave,
                },
            );
            collect_activations(
                &node_by_id,
                &outgoing,
                &mut unresolved,
                &mut active_inputs,
                &mut activations,
                &node_id,
                &result,
            )?;

            let execution_count = executions.entry(node_id.clone()).or_default();
            *execution_count += 1;
            let plan = plans.get(&node_id).ok_or_else(|| format!("Missing execution plan for '{node_id}'"))?;
            if plan.loop_node && *execution_count < template.max_iterations {
                unresolved.insert(node_id.clone(), plan.subsequent_required);
                pending.insert(node_id.clone());
            } else {
                pending.remove(&node_id);
            }
        }

        if summary.cancelled {
            break;
        }

        let mut queue: VecDeque<String> = sort_ids(activations, &rank).into();
        let mut next_wave: Vec<String> = Vec::new();
        while let Some(node_id) = queue.pop_front() {
            if !pending.contains(&node_id) {
                continue;
            }
            let has_incoming = incoming.get(&node_id).map(|edges| !edges.is_empty()).unwrap_or(false);
            let has_active_input = active_inputs.get(&node_id).map(|inputs| !inputs.is_empty()).unwrap_or(false);
            if !has_incoming || has_active_input {
                if let Some(inputs) = active_inputs.get_mut(&node_id) {
                    inputs.sort_by_key(|input| input.edge_index);
                }
                next_wave.push(node_id);
                continue;
            }

            pending.remove(&node_id);
            summary.execution_order.push(node_id.clone());
            summary.node_states.insert(
                node_id.clone(),
                NodeExecutionState {
                    outcome: NodeOutcome::Skipped,
                    output: None,
                    error: None,
                    wave: wave + 1,
                },
            );
            let mut chained = Vec::new();
            collect_activations(
                &node_by_id,
                &outgoing,
                &mut unresolved,
                &mut active_inputs,
                &mut chained,
                &node_id,
                &NodeExecutionResult::skipped(),
            )?;
            for chained_node_id in sort_ids(chained, &rank) {
                queue.push_back(chained_node_id);
            }
        }

        ready = sort_ids(next_wave, &rank);
        wave += 1;
    }

    if summary.cancelled {
        for node_id in sort_ids(pending.into_iter().collect(), &rank) {
            summary.execution_order.push(node_id.clone());
            summary.node_states.insert(
                node_id,
                NodeExecutionState {
                    outcome: NodeOutcome::Cancelled,
                    output: None,
                    error: Some("Run cancelled".into()),
                    wave: wave + 1,
                },
            );
        }
    }

    Ok(summary)
}

fn grouped_edges(edges: &[IndexedEdge], incoming: bool) -> HashMap<String, Vec<IndexedEdge>> {
    let mut grouped: HashMap<String, Vec<IndexedEdge>> = HashMap::new();
    for edge in edges {
        let key = if incoming { edge.edge.target_node_id.clone() } else { edge.edge.source_node_id.clone() };
        grouped.entry(key).or_default().push(edge.clone());
    }
    grouped
}

fn build_plans(
    nodes: &HashMap<String, TemplateNode>,
    incoming: &HashMap<String, Vec<IndexedEdge>>,
    loop_components: &HashMap<String, usize>,
) -> HashMap<String, NodePlan> {
    let mut plans = HashMap::new();
    for node_id in nodes.keys() {
        let incoming_edges = incoming.get(node_id).cloned().unwrap_or_default();
        let Some(component_id) = loop_components.get(node_id) else {
            plans.insert(
                node_id.clone(),
                NodePlan {
                    initial_required: incoming_edges.len(),
                    subsequent_required: 0,
                    loop_node: false,
                },
            );
            continue;
        };

        let external_required = incoming_edges
            .iter()
            .filter(|edge| loop_components.get(&edge.edge.source_node_id) != Some(component_id))
            .count();
        let internal_required = incoming_edges.len().saturating_sub(external_required);
        let initial_required = if external_required > 0 { external_required } else { internal_required };
        plans.insert(
            node_id.clone(),
            NodePlan {
                initial_required,
                subsequent_required: internal_required,
                loop_node: true,
            },
        );
    }
    plans
}

fn collect_activations(
    nodes: &HashMap<String, TemplateNode>,
    outgoing: &HashMap<String, Vec<IndexedEdge>>,
    unresolved: &mut HashMap<String, usize>,
    active_inputs: &mut HashMap<String, Vec<NodeInboundInput>>,
    activations: &mut Vec<String>,
    source_node_id: &str,
    result: &NodeExecutionResult,
) -> Result<(), String> {
    let source = nodes.get(source_node_id).ok_or_else(|| format!("Missing source node '{source_node_id}'"))?;
    for edge in outgoing.get(source_node_id).cloned().unwrap_or_default() {
        let Some(count) = unresolved.get_mut(&edge.edge.target_node_id) else {
            continue;
        };
        if *count > 0 {
            *count -= 1;
        }

        if edge_matches(&edge.edge.condition, result.outcome) {
            active_inputs.entry(edge.edge.target_node_id.clone()).or_default().push(NodeInboundInput {
                edge_id: edge.edge.id.clone(),
                source_node_id: source.id.clone(),
                input_key: edge.edge.input_key.clone(),
                output: result.output.clone().unwrap_or_default(),
                source_provider: source.provider.clone(),
                source_model: source.model.clone(),
                source_session_group: source.session_group.clone(),
                source_provider_session_ref: result.provider_session_ref.clone(),
                edge_index: edge.index,
            });
        }

        if *count == 0 {
            activations.push(edge.edge.target_node_id.clone());
        }
    }
    Ok(())
}

fn edge_matches(condition: &EdgeCondition, outcome: NodeOutcome) -> bool {
    match condition {
        EdgeCondition::Always => matches!(outcome, NodeOutcome::Success | NodeOutcome::Failure),
        EdgeCondition::OnSuccess => outcome == NodeOutcome::Success,
        EdgeCondition::OnFailure => outcome == NodeOutcome::Failure,
    }
}

fn sort_ids(ids: Vec<String>, rank: &HashMap<String, usize>) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut list: Vec<String> = ids.into_iter().filter(|id| seen.insert(id.clone())).collect();
    list.sort_by_key(|id| *rank.get(id).unwrap_or(&usize::MAX));
    list
}
