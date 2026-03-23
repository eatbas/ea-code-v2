use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::{Map, Value};
use tauri::AppHandle;
use tokio::sync::{oneshot, Mutex};

use crate::hive_client::HiveClient;
use crate::models::templates::{PipelineTemplate, TemplateNode, UiPosition};

use super::graph_executor::{execute_graph, NodeExecutionInput, NodeOutcome, NodeRunner};
use super::helpers::{
    emit_pipeline_completed, emit_pipeline_error, emit_pipeline_started, now_iso,
    PipelineCompletedEvent, PipelineErrorEvent, PipelineStartedEvent,
};
use super::session_manager::SessionManager;
use super::stage_runner::{run_node, StageRunnerContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunTerminalStatus {
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct DirectTaskConfig {
    pub provider: String,
    pub model: String,
    pub execution_intent: String,
    pub provider_options: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct PipelineRunRequest {
    pub run_id: String,
    pub prompt: String,
    pub workspace_path: String,
    pub template: Option<PipelineTemplate>,
    pub direct_task: Option<DirectTaskConfig>,
    pub extra_vars: HashMap<String, String>,
}

#[derive(Clone)]
pub struct PipelineRuntimeContext {
    pub app: AppHandle,
    pub hive_client: HiveClient,
    pub cancel_flag: Arc<AtomicBool>,
    pub pause_flag: Arc<AtomicBool>,
    pub active_job_id: Arc<Mutex<Option<String>>>,
    pub question_answers: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
}

pub async fn execute_run(ctx: PipelineRuntimeContext, request: PipelineRunRequest) -> RunTerminalStatus {
    emit_pipeline_started(
        &ctx.app,
        &PipelineStartedEvent {
            run_id: request.run_id.clone(),
            mode: if request.direct_task.is_some() {
                "direct_task".into()
            } else {
                "graph".into()
            },
            template_id: request.template.as_ref().map(|template| template.id.clone()),
            timestamp: now_iso(),
        },
    );

    let status = if let Some(config) = request.direct_task.clone() {
        run_direct_task(ctx.clone(), request.clone(), config).await
    } else {
        run_graph_template(ctx.clone(), request.clone()).await
    };

    emit_pipeline_completed(
        &ctx.app,
        &PipelineCompletedEvent {
            run_id: request.run_id,
            status: match status {
                RunTerminalStatus::Completed => "completed".into(),
                RunTerminalStatus::Failed => "failed".into(),
                RunTerminalStatus::Cancelled => "cancelled".into(),
            },
            timestamp: now_iso(),
        },
    );

    status
}

async fn run_direct_task(
    ctx: PipelineRuntimeContext,
    request: PipelineRunRequest,
    config: DirectTaskConfig,
) -> RunTerminalStatus {
    let node = TemplateNode {
        id: "direct-task".into(),
        label: "Direct Task".into(),
        stage_type: "direct".into(),
        handler: "chat".into(),
        provider: config.provider,
        model: config.model,
        session_group: "A".into(),
        prompt_template: "{{task}}".into(),
        enabled: true,
        execution_intent: config.execution_intent,
        config: Some(Value::Object(
            config
                .provider_options
                .into_iter()
                .collect::<Map<String, Value>>(),
        )),
        ui_position: UiPosition { x: 0.0, y: 0.0 },
    };

    let manager = Arc::new(Mutex::new(SessionManager::default()));
    let result = run_node(
        stage_ctx(&ctx),
        request.run_id,
        request.prompt,
        request.workspace_path,
        request.extra_vars,
        manager,
        NodeExecutionInput {
            node,
            inbound: vec![],
            wave: 0,
        },
    )
    .await;

    match result.outcome {
        NodeOutcome::Success => RunTerminalStatus::Completed,
        NodeOutcome::Cancelled => RunTerminalStatus::Cancelled,
        _ => RunTerminalStatus::Failed,
    }
}

async fn run_graph_template(ctx: PipelineRuntimeContext, request: PipelineRunRequest) -> RunTerminalStatus {
    let Some(template) = request.template.clone() else {
        emit_pipeline_error(
            &ctx.app,
            &PipelineErrorEvent {
                run_id: request.run_id,
                message: "Missing template for graph run".into(),
                timestamp: now_iso(),
                node_id: None,
            },
        );
        return RunTerminalStatus::Failed;
    };

    let manager = Arc::new(Mutex::new(SessionManager::default()));
    let runner: NodeRunner = {
        let ctx = stage_ctx(&ctx);
        let run_id = request.run_id.clone();
        let task = request.prompt.clone();
        let workspace = request.workspace_path.clone();
        let extra_vars = request.extra_vars.clone();
        let manager = manager.clone();
        Arc::new(move |input| {
            Box::pin(run_node(
                ctx.clone(),
                run_id.clone(),
                task.clone(),
                workspace.clone(),
                extra_vars.clone(),
                manager.clone(),
                input,
            ))
        })
    };

    let summary = match execute_graph(&template, runner).await {
        Ok(summary) => summary,
        Err(error) => {
            emit_pipeline_error(
                &ctx.app,
                &PipelineErrorEvent {
                    run_id: request.run_id,
                    message: error,
                    timestamp: now_iso(),
                    node_id: None,
                },
            );
            return RunTerminalStatus::Failed;
        }
    };

    if summary.cancelled || ctx.cancel_flag.load(Ordering::Relaxed) {
        return RunTerminalStatus::Cancelled;
    }
    if summary.failed {
        return RunTerminalStatus::Failed;
    }

    RunTerminalStatus::Completed
}

fn stage_ctx(ctx: &PipelineRuntimeContext) -> StageRunnerContext {
    StageRunnerContext {
        app: ctx.app.clone(),
        hive_client: ctx.hive_client.clone(),
        cancel_flag: ctx.cancel_flag.clone(),
        pause_flag: ctx.pause_flag.clone(),
        active_job_id: ctx.active_job_id.clone(),
        question_answers: ctx.question_answers.clone(),
    }
}
