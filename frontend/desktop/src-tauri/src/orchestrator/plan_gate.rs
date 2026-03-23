use std::collections::HashMap;
use std::sync::Arc;

use tauri::AppHandle;
use tokio::sync::{oneshot, Mutex};

use crate::models::templates::TemplateNode;

use super::user_questions::request_user_approval;

const DEFAULT_APPROVAL_TIMEOUT_SECS: u64 = 60;

fn requires_plan_approval(node: &TemplateNode) -> bool {
    let Some(config) = &node.config else {
        return false;
    };
    config
        .get("requires_plan_approval")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn approval_timeout_secs(node: &TemplateNode) -> u64 {
    let Some(config) = &node.config else {
        return DEFAULT_APPROVAL_TIMEOUT_SECS;
    };
    config
        .get("plan_auto_approve_timeout_sec")
        .and_then(|value| value.as_u64())
        .unwrap_or(DEFAULT_APPROVAL_TIMEOUT_SECS)
}

pub async fn enforce_plan_gate(
    app: &AppHandle,
    question_answers: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
    run_id: &str,
    node: &TemplateNode,
) -> Result<(), String> {
    if !requires_plan_approval(node) {
        return Ok(());
    }

    let approved = request_user_approval(
        app,
        question_answers,
        run_id,
        &node.id,
        format!("Approve plan before executing '{}'?", node.label),
        approval_timeout_secs(node),
    )
    .await?;

    if approved {
        Ok(())
    } else {
        Err("Plan approval rejected by user.".into())
    }
}
