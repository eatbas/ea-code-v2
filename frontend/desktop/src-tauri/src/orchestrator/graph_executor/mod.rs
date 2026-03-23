use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::models::templates::{PipelineTemplate, TemplateEdge, TemplateNode};

mod scheduler;
mod topology;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeOutcome {
    Success,
    Failure,
    Cancelled,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct NodeExecutionResult {
    pub outcome: NodeOutcome,
    pub output: Option<String>,
    pub error: Option<String>,
    pub provider_session_ref: Option<String>,
}

impl NodeExecutionResult {
    pub fn success(output: String, provider_session_ref: Option<String>) -> Self {
        Self {
            outcome: NodeOutcome::Success,
            output: Some(output),
            error: None,
            provider_session_ref,
        }
    }

    pub fn failure(error: String) -> Self {
        Self { outcome: NodeOutcome::Failure, output: None, error: Some(error), provider_session_ref: None }
    }

    pub fn cancelled(error: Option<String>) -> Self {
        Self { outcome: NodeOutcome::Cancelled, output: None, error, provider_session_ref: None }
    }

    pub fn skipped() -> Self {
        Self { outcome: NodeOutcome::Skipped, output: None, error: None, provider_session_ref: None }
    }
}

#[derive(Debug, Clone)]
pub struct NodeInboundInput {
    pub edge_id: String,
    pub source_node_id: String,
    pub input_key: Option<String>,
    pub output: String,
    pub source_provider: String,
    pub source_model: String,
    pub source_session_group: String,
    pub source_provider_session_ref: Option<String>,
    edge_index: usize,
}

#[derive(Debug, Clone)]
pub struct NodeExecutionInput {
    pub node: TemplateNode,
    pub inbound: Vec<NodeInboundInput>,
    pub wave: u32,
}

pub type NodeRunner =
    Arc<dyn Fn(NodeExecutionInput) -> Pin<Box<dyn Future<Output = NodeExecutionResult> + Send>> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct NodeExecutionState {
    pub outcome: NodeOutcome,
    pub output: Option<String>,
    pub error: Option<String>,
    pub wave: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GraphExecutionSummary {
    pub node_states: std::collections::HashMap<String, NodeExecutionState>,
    pub execution_order: Vec<String>,
    pub cancelled: bool,
    pub failed: bool,
}

#[derive(Debug, Clone)]
struct IndexedEdge {
    index: usize,
    edge: TemplateEdge,
}

/// Executes a template graph and returns the execution summary.
pub async fn execute_graph(
    template: &PipelineTemplate,
    runner: NodeRunner,
) -> Result<GraphExecutionSummary, String> {
    scheduler::execute_graph(template, runner).await
}
