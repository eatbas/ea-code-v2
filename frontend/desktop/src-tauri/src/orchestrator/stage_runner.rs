use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::AppHandle;
use tokio::sync::{oneshot, Mutex};

use crate::hive_client::streaming::ChatRequest;
use crate::hive_client::HiveClient;
use crate::models::templates::TemplateNode;

use super::graph_executor::{NodeExecutionInput, NodeExecutionResult, NodeOutcome};
use super::helpers::{
    capture_git_diff, emit_pipeline_artifact, emit_pipeline_error, emit_pipeline_log,
    emit_pipeline_stage, now_iso, provider_options_from_config, stream_chat_node, wait_if_paused,
    PipelineArtifactEvent, PipelineErrorEvent, PipelineLogEvent, PipelineStageEvent,
};
use super::plan_gate::enforce_plan_gate;
use super::prompt_renderer::{render_node_prompt, PromptContext, UpstreamOutput};
use super::session_manager::{SessionCandidate, SessionDecision, SessionManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandlerKind {
    Chat,
    Judge,
    Summary,
    SkillSelect,
    SkillRun,
}

#[derive(Clone)]
pub struct StageRunnerContext {
    pub app: AppHandle,
    pub hive_client: HiveClient,
    pub cancel_flag: Arc<AtomicBool>,
    pub pause_flag: Arc<AtomicBool>,
    pub active_job_id: Arc<Mutex<Option<String>>>,
    pub question_answers: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
}

fn handler_registry() -> HashMap<&'static str, HandlerKind> {
    HashMap::from([
        ("chat", HandlerKind::Chat),
        // Backward-compatible aliases used by built-in and legacy templates.
        ("analyse", HandlerKind::Chat),
        ("review", HandlerKind::Chat),
        ("implement", HandlerKind::Chat),
        ("test", HandlerKind::Chat),
        ("custom", HandlerKind::Chat),
        ("judge", HandlerKind::Judge),
        ("summary", HandlerKind::Summary),
        ("skill_select", HandlerKind::SkillSelect),
        ("skill_run", HandlerKind::SkillRun),
    ])
}

pub async fn run_node(
    ctx: StageRunnerContext,
    run_id: String,
    task: String,
    workspace_path: String,
    extra_vars: HashMap<String, String>,
    manager: Arc<Mutex<SessionManager>>,
    input: NodeExecutionInput,
) -> NodeExecutionResult {
    wait_if_paused(&ctx.pause_flag, &ctx.cancel_flag).await;
    if ctx.cancel_flag.load(Ordering::Relaxed) {
        return NodeExecutionResult::cancelled(Some("Run cancelled".into()));
    }

    if let Err(error) = enforce_plan_gate(&ctx.app, ctx.question_answers.clone(), &run_id, &input.node).await {
        emit_pipeline_error(
            &ctx.app,
            &PipelineErrorEvent {
                run_id,
                message: error.clone(),
                timestamp: now_iso(),
                node_id: Some(input.node.id.clone()),
            },
        );
        return NodeExecutionResult::failure(error);
    }

    emit_pipeline_stage(
        &ctx.app,
        &PipelineStageEvent {
            run_id: run_id.clone(),
            node_id: input.node.id.clone(),
            node_label: input.node.label.clone(),
            status: "running".into(),
            timestamp: now_iso(),
            detail: None,
        },
    );

    let mut edge_inputs: HashMap<String, String> = HashMap::new();
    for inbound in &input.inbound {
        if let Some(key) = &inbound.input_key {
            edge_inputs
                .entry(key.clone())
                .and_modify(|value| {
                    value.push_str("\n\n");
                    value.push_str(&inbound.output);
                })
                .or_insert_with(|| inbound.output.clone());
        }
    }

    let candidates: Vec<SessionCandidate> = input
        .inbound
        .iter()
        .filter_map(|inbound| {
            inbound
                .source_provider_session_ref
                .clone()
                .map(|provider_session_ref| SessionCandidate {
                    source_node_id: inbound.source_node_id.clone(),
                    session_group: inbound.source_session_group.clone(),
                    provider: inbound.source_provider.clone(),
                    model: inbound.source_model.clone(),
                    provider_session_ref,
                })
        })
        .collect();

    let (mode, resume_ref, warning) = {
        let guard = manager.lock().await;
        match guard.decide_mode(&input.node, &candidates) {
            SessionDecision::New { warning } => ("new".to_string(), None, warning),
            SessionDecision::Resume(reference) => ("resume".to_string(), Some(reference), None),
        }
    };

    if let Some(warning_text) = warning {
        emit_pipeline_log(
            &ctx.app,
            &PipelineLogEvent {
                run_id: run_id.clone(),
                node_id: input.node.id.clone(),
                text: warning_text,
                timestamp: now_iso(),
            },
        );
    }

    let chat_request = ChatRequest {
        provider: input.node.provider.clone(),
        model: input.node.model.clone(),
        workspace_path: workspace_path.clone(),
        mode,
        prompt: render_node_prompt(
            &input.node.prompt_template,
            &PromptContext {
                task,
                workspace_path: workspace_path.clone(),
                iteration_number: input.wave + 1,
                max_iterations: 1,
                upstream_outputs: input
                    .inbound
                    .iter()
                    .map(|item| UpstreamOutput {
                        node_id: item.source_node_id.clone(),
                        output: item.output.clone(),
                    })
                    .collect(),
                edge_inputs,
                extra_vars,
            },
        ),
        stream: true,
        provider_session_ref: resume_ref.clone(),
        provider_options: provider_options_from_config(input.node.config.as_ref()),
    };

    let result = dispatch_handler(&ctx, &run_id, &input.node, &chat_request).await;
    handle_node_completion(&ctx, &run_id, &workspace_path, &manager, &input.node, &resume_ref, &result).await;
    result
}

async fn dispatch_handler(
    ctx: &StageRunnerContext,
    run_id: &str,
    node: &TemplateNode,
    chat_request: &ChatRequest,
) -> NodeExecutionResult {
    let Some(handler) = handler_registry().get(node.handler.as_str()).copied() else {
        return NodeExecutionResult::failure(format!("Unknown node handler '{}'.", node.handler));
    };

    match handler {
        HandlerKind::Chat | HandlerKind::Summary | HandlerKind::SkillSelect | HandlerKind::SkillRun => {
            stream_chat_node(
                &ctx.app,
                &ctx.hive_client,
                ctx.cancel_flag.clone(),
                ctx.active_job_id.clone(),
                run_id,
                node,
                chat_request,
            )
            .await
        }
        HandlerKind::Judge => run_judge_handler(ctx, run_id, node, chat_request).await,
    }
}

async fn run_judge_handler(
    ctx: &StageRunnerContext,
    run_id: &str,
    node: &TemplateNode,
    chat_request: &ChatRequest,
) -> NodeExecutionResult {
    let result = stream_chat_node(
        &ctx.app,
        &ctx.hive_client,
        ctx.cancel_flag.clone(),
        ctx.active_job_id.clone(),
        run_id,
        node,
        chat_request,
    )
    .await;

    if result.outcome == NodeOutcome::Success {
        let verdict = result
            .output
            .as_ref()
            .and_then(|output| extract_judge_verdict(output))
            .unwrap_or("UNKNOWN");
        emit_pipeline_log(
            &ctx.app,
            &PipelineLogEvent {
                run_id: run_id.to_string(),
                node_id: node.id.clone(),
                text: format!("Judge verdict: {verdict}"),
                timestamp: now_iso(),
            },
        );
    }

    result
}

fn extract_judge_verdict(output: &str) -> Option<&'static str> {
    let upper = output.to_ascii_uppercase();
    if upper.contains("NOT_COMPLETE") {
        return Some("NOT_COMPLETE");
    }
    if upper.contains("COMPLETE") {
        return Some("COMPLETE");
    }
    None
}

async fn handle_node_completion(
    ctx: &StageRunnerContext,
    run_id: &str,
    workspace_path: &str,
    manager: &Arc<Mutex<SessionManager>>,
    node: &TemplateNode,
    resume_ref: &Option<String>,
    result: &NodeExecutionResult,
) {
    match result.outcome {
        NodeOutcome::Success => {
            emit_pipeline_stage(&ctx.app, &PipelineStageEvent { run_id: run_id.into(), node_id: node.id.clone(), node_label: node.label.clone(), status: "completed".into(), timestamp: now_iso(), detail: None });
            if node.execution_intent == "code" {
                if let Ok(Some(diff)) = capture_git_diff(workspace_path).await {
                    emit_pipeline_artifact(&ctx.app, &PipelineArtifactEvent { run_id: run_id.into(), node_id: node.id.clone(), name: "git-diff.patch".into(), artifact_type: "git_diff".into(), content: diff, timestamp: now_iso() });
                }
            }
            manager
                .lock()
                .await
                .remember(node, result.provider_session_ref.as_deref().or(resume_ref.as_deref()));
        }
        NodeOutcome::Failure | NodeOutcome::Cancelled => {
            let failed = result.outcome == NodeOutcome::Failure;
            emit_pipeline_stage(&ctx.app, &PipelineStageEvent { run_id: run_id.into(), node_id: node.id.clone(), node_label: node.label.clone(), status: if failed { "failed" } else { "cancelled" }.into(), timestamp: now_iso(), detail: result.error.clone() });
            if failed {
                emit_pipeline_error(&ctx.app, &PipelineErrorEvent { run_id: run_id.into(), message: result.error.clone().unwrap_or_else(|| "Node failed".into()), timestamp: now_iso(), node_id: Some(node.id.clone()) });
            }
        }
        NodeOutcome::Skipped => {}
    }
}
