use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

use crate::hive_client::chat::HiveSseEvent;
use crate::hive_client::streaming::ChatRequest;
use crate::hive_client::HiveClient;
use crate::models::templates::TemplateNode;
use crate::orchestrator::graph_executor::NodeExecutionResult;

pub const PIPELINE_STARTED_EVENT: &str = "pipeline:started";
pub const PIPELINE_STAGE_EVENT: &str = "pipeline:stage";
pub const PIPELINE_LOG_EVENT: &str = "pipeline:log";
pub const PIPELINE_ARTIFACT_EVENT: &str = "pipeline:artifact";
pub const PIPELINE_QUESTION_EVENT: &str = "pipeline:question";
pub const PIPELINE_COMPLETED_EVENT: &str = "pipeline:completed";
pub const PIPELINE_ERROR_EVENT: &str = "pipeline:error";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStartedEvent {
    pub run_id: String,
    pub mode: String,
    pub template_id: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStageEvent {
    pub run_id: String,
    pub node_id: String,
    pub node_label: String,
    pub status: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineLogEvent {
    pub run_id: String,
    pub node_id: String,
    pub text: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineArtifactEvent {
    pub run_id: String,
    pub node_id: String,
    pub name: String,
    pub artifact_type: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineQuestionEvent {
    pub run_id: String,
    pub question_id: String,
    pub node_id: String,
    pub question_text: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCompletedEvent {
    pub run_id: String,
    pub status: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineErrorEvent {
    pub run_id: String,
    pub message: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn emit_pipeline_started(app: &AppHandle, payload: &PipelineStartedEvent) {
    let _ = app.emit(PIPELINE_STARTED_EVENT, payload);
}

pub fn emit_pipeline_stage(app: &AppHandle, payload: &PipelineStageEvent) {
    let _ = app.emit(PIPELINE_STAGE_EVENT, payload);
}

pub fn emit_pipeline_log(app: &AppHandle, payload: &PipelineLogEvent) {
    let _ = app.emit(PIPELINE_LOG_EVENT, payload);
}

pub fn emit_pipeline_artifact(app: &AppHandle, payload: &PipelineArtifactEvent) {
    let _ = app.emit(PIPELINE_ARTIFACT_EVENT, payload);
}

pub fn emit_pipeline_question(app: &AppHandle, payload: &PipelineQuestionEvent) {
    let _ = app.emit(PIPELINE_QUESTION_EVENT, payload);
}

pub fn emit_pipeline_completed(app: &AppHandle, payload: &PipelineCompletedEvent) {
    let _ = app.emit(PIPELINE_COMPLETED_EVENT, payload);
}

pub fn emit_pipeline_error(app: &AppHandle, payload: &PipelineErrorEvent) {
    let _ = app.emit(PIPELINE_ERROR_EVENT, payload);
}

pub async fn wait_if_paused(pause_flag: &AtomicBool, cancel_flag: &AtomicBool) {
    while pause_flag.load(Ordering::Relaxed) && !cancel_flag.load(Ordering::Relaxed) {
        sleep(Duration::from_millis(150)).await;
    }
}

pub async fn capture_git_diff(workspace_path: &str) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_path)
        .arg("diff")
        .arg("--no-color")
        .output()
        .await
        .map_err(|e| format!("Failed to capture git diff: {e}"))?;

    if !output.status.success() {
        return Ok(None);
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

pub fn provider_options_from_config(config: Option<&Value>) -> std::collections::HashMap<String, Value> {
    config
        .and_then(|v| v.as_object())
        .map(|map| map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default()
}

pub async fn stream_chat_node(
    app: &AppHandle,
    hive_client: &HiveClient,
    cancel_flag: std::sync::Arc<AtomicBool>,
    active_job_id: std::sync::Arc<Mutex<Option<String>>>,
    run_id: &str,
    node: &TemplateNode,
    request: &ChatRequest,
) -> NodeExecutionResult {
    let app_for_logs = app.clone();
    let run = run_id.to_string();
    let node_id = node.id.clone();
    let cancel_for_cb = cancel_flag.clone();
    let active_for_cb = active_job_id.clone();

    let response = hive_client
        .chat_stream(request, cancel_flag.clone(), move |event| {
            if cancel_for_cb.load(Ordering::Relaxed) {
                return;
            }
            match event {
                HiveSseEvent::RunStarted { job_id, .. } => {
                    let active = active_for_cb.clone();
                    let next = job_id.clone();
                    tokio::spawn(async move {
                        *active.lock().await = Some(next);
                    });
                }
                HiveSseEvent::OutputDelta { text } => {
                    emit_pipeline_log(
                        &app_for_logs,
                        &PipelineLogEvent {
                            run_id: run.clone(),
                            node_id: node_id.clone(),
                            text: text.clone(),
                            timestamp: now_iso(),
                        },
                    );
                }
                _ => {}
            }
        })
        .await;

    *active_job_id.lock().await = None;

    match response {
        Ok(result) => NodeExecutionResult::success(result.final_text, result.provider_session_ref),
        Err(error) => {
            let cancelled = cancel_flag.load(Ordering::Relaxed)
                || error.to_lowercase().contains("cancel")
                || error.to_lowercase().contains("stopped");
            if cancelled {
                NodeExecutionResult::cancelled(Some(error))
            } else {
                NodeExecutionResult::failure(error)
            }
        }
    }
}
