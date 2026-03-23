use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::templates::{EdgeCondition, PipelineTemplate, TemplateEdge, TemplateNode, UiPosition};

use super::{execute_graph, NodeExecutionResult, NodeRunner};

fn node(id: &str) -> TemplateNode {
    TemplateNode {
        id: id.into(),
        label: id.into(),
        stage_type: "analyse".into(),
        handler: "analyse".into(),
        provider: "claude".into(),
        model: "opus".into(),
        session_group: "A".into(),
        prompt_template: "{{task}}".into(),
        enabled: true,
        execution_intent: "text".into(),
        config: None,
        ui_position: UiPosition { x: 0.0, y: 0.0 },
    }
}

fn edge(id: &str, source: &str, target: &str, condition: EdgeCondition, loop_control: bool) -> TemplateEdge {
    TemplateEdge {
        id: id.into(),
        source_node_id: source.into(),
        target_node_id: target.into(),
        condition,
        input_key: None,
        loop_control,
    }
}

fn template(max_iterations: u32, nodes: Vec<TemplateNode>, edges: Vec<TemplateEdge>) -> PipelineTemplate {
    PipelineTemplate {
        id: "template".into(),
        name: "Template".into(),
        description: "Template".into(),
        is_builtin: false,
        max_iterations,
        stop_on_first_pass: true,
        nodes,
        edges,
        created_at: "2026-03-23T12:00:00Z".into(),
        updated_at: "2026-03-23T12:00:00Z".into(),
    }
}

fn runner(counts: Arc<Mutex<HashMap<String, usize>>>, failing_node: Option<&'static str>) -> NodeRunner {
    Arc::new(move |input| {
        let counts = counts.clone();
        Box::pin(async move {
            *counts.lock().expect("counts mutex").entry(input.node.id.clone()).or_insert(0) += 1;
            if failing_node == Some(input.node.id.as_str()) {
                NodeExecutionResult::failure(format!("node '{}' failed", input.node.id))
            } else {
                NodeExecutionResult::success(format!("{}@{}", input.node.id, input.wave), None)
            }
        })
    })
}

#[tokio::test]
async fn dag_executes_each_node_once() {
    let counts = Arc::new(Mutex::new(HashMap::new()));
    let summary = execute_graph(
        &template(
            1,
            vec![node("start"), node("middle"), node("end")],
            vec![
                edge("e1", "start", "middle", EdgeCondition::Always, false),
                edge("e2", "middle", "end", EdgeCondition::OnSuccess, false),
            ],
        ),
        runner(counts.clone(), None),
    )
    .await
    .expect("DAG execution should succeed");

    let counts = counts.lock().expect("counts mutex");
    assert_eq!(counts.get("start"), Some(&1));
    assert_eq!(counts.get("middle"), Some(&1));
    assert_eq!(counts.get("end"), Some(&1));
    assert_eq!(
        summary.execution_order,
        vec!["start".to_string(), "middle".to_string(), "end".to_string()]
    );
}

#[tokio::test]
async fn loop_control_cycle_executes_bounded_iterations() {
    let counts = Arc::new(Mutex::new(HashMap::new()));
    let summary = execute_graph(
        &template(
            3,
            vec![node("start"), node("a"), node("b"), node("end")],
            vec![
                edge("e1", "start", "a", EdgeCondition::Always, false),
                edge("e2", "a", "b", EdgeCondition::OnSuccess, true),
                edge("e3", "b", "a", EdgeCondition::OnSuccess, true),
                edge("e4", "b", "end", EdgeCondition::Always, false),
            ],
        ),
        runner(counts.clone(), None),
    )
    .await
    .expect("loop-control execution should not deadlock");

    let counts = counts.lock().expect("counts mutex");
    assert_eq!(counts.get("start"), Some(&1));
    assert_eq!(counts.get("a"), Some(&3));
    assert_eq!(counts.get("b"), Some(&3));
    assert_eq!(counts.get("end"), Some(&1));
    assert!(!summary.failed);
    assert!(!summary.cancelled);
    assert_eq!(summary.execution_order.iter().filter(|node_id| node_id.as_str() == "a").count(), 3);
    assert_eq!(summary.execution_order.iter().filter(|node_id| node_id.as_str() == "b").count(), 3);
}

#[tokio::test]
async fn unresolvable_graph_still_errors() {
    let counts = Arc::new(Mutex::new(HashMap::new()));
    let error = execute_graph(
        &template(
            3,
            vec![node("a"), node("b")],
            vec![
                edge("e1", "a", "b", EdgeCondition::Always, true),
                edge("e2", "b", "a", EdgeCondition::Always, true),
            ],
        ),
        runner(counts.clone(), None),
    )
    .await
    .expect_err("cycle without an entry node should deadlock");

    let counts = counts.lock().expect("counts mutex");
    assert!(counts.is_empty());
    assert!(error.contains("deadlock"));
}
