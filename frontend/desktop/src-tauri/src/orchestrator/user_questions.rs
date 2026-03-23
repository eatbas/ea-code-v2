use std::collections::HashMap;
use std::sync::Arc;

use tauri::AppHandle;
use tokio::sync::{oneshot, Mutex};
use tokio::time::{timeout, Duration};

use super::helpers::{emit_pipeline_question, now_iso, PipelineQuestionEvent};

fn answer_is_approval(answer: &str) -> bool {
    matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "approve" | "approved" | "yes" | "y" | "ok" | "continue" | "true"
    )
}

pub async fn request_user_approval(
    app: &AppHandle,
    question_answers: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
    run_id: &str,
    node_id: &str,
    question_text: String,
    timeout_secs: u64,
) -> Result<bool, String> {
    let question_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel::<String>();

    {
        let mut guard = question_answers.lock().await;
        guard.insert(question_id.clone(), tx);
    }

    emit_pipeline_question(
        app,
        &PipelineQuestionEvent {
            run_id: run_id.to_string(),
            question_id: question_id.clone(),
            node_id: node_id.to_string(),
            question_text,
            timestamp: now_iso(),
        },
    );

    let answer = timeout(Duration::from_secs(timeout_secs), rx).await;
    {
        let mut guard = question_answers.lock().await;
        let _ = guard.remove(&question_id);
    }

    match answer {
        Ok(Ok(value)) => Ok(answer_is_approval(&value)),
        Ok(Err(_)) => Err("Approval channel closed before answer was received.".into()),
        Err(_) => Ok(true), // auto-approve on timeout
    }
}
