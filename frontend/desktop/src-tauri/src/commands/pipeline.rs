use std::collections::HashMap;
use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, State};

use crate::commands::AppState;
use crate::models::templates::PipelineTemplate;
use crate::orchestrator::pipeline::{
    execute_run, DirectTaskConfig, PipelineRunRequest, PipelineRuntimeContext,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPipelineRunRequest {
    pub prompt: String,
    pub workspace_path: String,
    #[serde(default)]
    pub template_id: Option<String>,
    #[serde(default)]
    pub template: Option<PipelineTemplate>,
    #[serde(default)]
    pub direct_task: bool,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub execution_intent: Option<String>,
    #[serde(default)]
    pub provider_options: HashMap<String, Value>,
    #[serde(default)]
    pub extra_vars: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PipelineControlRequest {
    #[serde(default)]
    pub run_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerPipelineQuestionRequest {
    pub question_id: String,
    pub answer_text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineRunStateSnapshot {
    pub run_id: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPipelineRunResult {
    pub run_id: String,
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn start_pipeline_run(
    app: AppHandle,
    payload: StartPipelineRunRequest,
    state: State<'_, AppState>,
) -> Result<StartPipelineRunResult, String> {
    state
        .run_active
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .map_err(|_| "A pipeline run is already active.".to_string())?;

    state.cancel_flag.store(false, Ordering::Relaxed);
    state.pause_flag.store(false, Ordering::Relaxed);

    let client = {
        let guard = state.hive_client.lock().await;
        guard
            .as_ref()
            .cloned()
            .ok_or("hive-api client not initialised")
            .map_err(|e| {
                state.run_active.store(false, Ordering::SeqCst);
                e.to_string()
            })?
    };

    let template = if payload.direct_task {
        None
    } else if let Some(template) = payload.template {
        Some(template)
    } else if let Some(template_id) = payload.template_id {
        Some(crate::commands::templates::get_template(template_id).await.map_err(|e| {
            state.run_active.store(false, Ordering::SeqCst);
            e
        })?)
    } else {
        state.run_active.store(false, Ordering::SeqCst);
        return Err("Either template or templateId is required for graph runs.".into());
    };

    let direct_task = if payload.direct_task {
        Some(DirectTaskConfig {
            provider: payload
                .provider
                .ok_or("provider is required in direct task mode")
                .map_err(|e| {
                    state.run_active.store(false, Ordering::SeqCst);
                    e.to_string()
                })?,
            model: payload
                .model
                .ok_or("model is required in direct task mode")
                .map_err(|e| {
                    state.run_active.store(false, Ordering::SeqCst);
                    e.to_string()
                })?,
            execution_intent: payload.execution_intent.unwrap_or_else(|| "text".into()),
            provider_options: payload.provider_options,
        })
    } else {
        None
    };

    let run_id = uuid::Uuid::new_v4().to_string();
    *state.active_run_id.lock().await = Some(run_id.clone());

    let request = PipelineRunRequest {
        run_id: run_id.clone(),
        prompt: payload.prompt,
        workspace_path: payload.workspace_path,
        template,
        direct_task,
        extra_vars: payload.extra_vars,
    };

    let context = PipelineRuntimeContext {
        app,
        hive_client: client,
        cancel_flag: state.cancel_flag.clone(),
        pause_flag: state.pause_flag.clone(),
        active_job_id: state.active_job_id.clone(),
        question_answers: state.question_answers.clone(),
    };

    let run_active = state.run_active.clone();
    let active_run_id = state.active_run_id.clone();
    let active_job_id = state.active_job_id.clone();
    let pause_flag = state.pause_flag.clone();

    tokio::spawn(async move {
        let _ = execute_run(context, request).await;
        run_active.store(false, Ordering::SeqCst);
        pause_flag.store(false, Ordering::Relaxed);
        *active_run_id.lock().await = None;
        *active_job_id.lock().await = None;
    });

    let now = chrono::Utc::now().to_rfc3339();
    Ok(StartPipelineRunResult {
        run_id,
        status: "running".into(),
        started_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn pause_pipeline_run(
    payload: Option<PipelineControlRequest>,
    state: State<'_, AppState>,
) -> Result<PipelineRunStateSnapshot, String> {
    let _ = payload;
    if !state.run_active.load(Ordering::Relaxed) {
        return Err("No active pipeline run to pause.".into());
    }
    state.pause_flag.store(true, Ordering::Relaxed);

    let run_id = state
        .active_run_id
        .lock()
        .await
        .clone()
        .ok_or("No active run id found.")?;
    Ok(PipelineRunStateSnapshot {
        run_id,
        status: "paused".into(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub async fn resume_pipeline_run(
    payload: Option<PipelineControlRequest>,
    state: State<'_, AppState>,
) -> Result<PipelineRunStateSnapshot, String> {
    let _ = payload;
    if !state.run_active.load(Ordering::Relaxed) {
        return Err("No active pipeline run to resume.".into());
    }
    state.pause_flag.store(false, Ordering::Relaxed);

    let run_id = state
        .active_run_id
        .lock()
        .await
        .clone()
        .ok_or("No active run id found.")?;
    Ok(PipelineRunStateSnapshot {
        run_id,
        status: "running".into(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub async fn cancel_pipeline_run(
    payload: Option<PipelineControlRequest>,
    state: State<'_, AppState>,
) -> Result<PipelineRunStateSnapshot, String> {
    let _ = payload;
    let run_id = state
        .active_run_id
        .lock()
        .await
        .clone()
        .unwrap_or_else(|| "unknown".into());

    state.cancel_flag.store(true, Ordering::Relaxed);
    state.pause_flag.store(false, Ordering::Relaxed);

    let active_job_id = state.active_job_id.lock().await.clone();
    if let Some(job_id) = active_job_id {
        let client = {
            let guard = state.hive_client.lock().await;
            guard.as_ref().cloned()
        };
        if let Some(client) = client {
            let _ = client.cancel_job(&job_id).await;
        }
    }

    Ok(PipelineRunStateSnapshot {
        run_id,
        status: "cancelled".into(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub async fn answer_pipeline_question(
    payload: AnswerPipelineQuestionRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let sender = {
        let mut guard = state.question_answers.lock().await;
        guard
            .remove(&payload.question_id)
            .ok_or_else(|| format!("Unknown question id '{}'.", payload.question_id))?
    };

    sender
        .send(payload.answer_text)
        .map_err(|_| "Pipeline is no longer waiting for this answer.".to_string())
}
