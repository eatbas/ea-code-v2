use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::Deserialize;
use tauri::State;

use crate::commands::AppState;
use crate::hive_client::streaming::ChatRequest;
use crate::models::templates::{PipelineTemplate, TemplateEdge, TemplateNode};
use crate::models::validation::validate_template;
use crate::storage::builtin_templates::builtin_templates;
use crate::storage::templates::{
    delete_template as storage_delete, list_user_templates, read_template as storage_read,
    write_template,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: String,
    pub max_iterations: u32,
    pub stop_on_first_pass: bool,
    pub nodes: Vec<TemplateNode>,
    #[serde(default)]
    pub edges: Vec<TemplateEdge>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTemplateRequest {
    pub name: String,
    pub description: String,
    pub max_iterations: u32,
    pub stop_on_first_pass: bool,
    pub nodes: Vec<TemplateNode>,
    #[serde(default)]
    pub edges: Vec<TemplateEdge>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloneTemplateRequest {
    pub new_name: String,
}

const ENHANCE_META_PROMPT: &str = r#"You are a prompt engineer. Improve this prompt for an AI coding agent.
Make it clearer, more structured, and more effective.
Preserve the user's intent exactly. Add specificity where vague.
Use available template variables: {{task}}, {{code_context}}, {{previous_output}},
{{file_list}}, {{iteration_number}}, {{test_results}}.
Return ONLY the improved prompt, no explanations.

---

"#;

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn find_builtin(id: &str) -> Option<PipelineTemplate> {
    builtin_templates().into_iter().find(|t| t.id == id)
}

async fn check_name_unique(name: &str, exclude_id: Option<&str>) -> Result<(), String> {
    let trimmed = name.trim();
    for t in builtin_templates() {
        if t.name.trim().eq_ignore_ascii_case(trimmed) && exclude_id != Some(t.id.as_str()) {
            return Err(format!(
                "A built-in template named '{}' already exists.",
                t.name
            ));
        }
    }
    for t in list_user_templates().await? {
        if t.name.trim().eq_ignore_ascii_case(trimmed) && exclude_id != Some(t.id.as_str()) {
            return Err(format!("A user template named '{}' already exists.", t.name));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn list_templates() -> Result<Vec<PipelineTemplate>, String> {
    let mut all = builtin_templates();
    all.extend(list_user_templates().await?);
    all.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(all)
}

#[tauri::command]
pub async fn get_template(id: String) -> Result<PipelineTemplate, String> {
    if let Some(t) = find_builtin(&id) {
        return Ok(t);
    }
    storage_read(&id).await
}

#[tauri::command]
pub async fn create_template(payload: CreateTemplateRequest) -> Result<PipelineTemplate, String> {
    let now = now_iso();
    let template = PipelineTemplate {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.name,
        description: payload.description,
        is_builtin: false,
        max_iterations: payload.max_iterations,
        stop_on_first_pass: payload.stop_on_first_pass,
        nodes: payload.nodes,
        edges: payload.edges,
        created_at: now.clone(),
        updated_at: now,
    };
    let errors = validate_template(&template);
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    check_name_unique(&template.name, None).await?;
    write_template(&template).await?;
    Ok(template)
}

#[tauri::command]
pub async fn update_template(
    id: String,
    payload: UpdateTemplateRequest,
) -> Result<PipelineTemplate, String> {
    let existing = get_template(id).await?;
    if existing.is_builtin {
        return Err("Cannot modify a built-in template. Clone it first.".into());
    }
    let template = PipelineTemplate {
        id: existing.id,
        name: payload.name,
        description: payload.description,
        is_builtin: false,
        max_iterations: payload.max_iterations,
        stop_on_first_pass: payload.stop_on_first_pass,
        nodes: payload.nodes,
        edges: payload.edges,
        created_at: existing.created_at,
        updated_at: now_iso(),
    };
    let errors = validate_template(&template);
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    check_name_unique(&template.name, Some(&template.id)).await?;
    write_template(&template).await?;
    Ok(template)
}

#[tauri::command]
pub async fn delete_template(id: String) -> Result<(), String> {
    let existing = get_template(id.clone()).await?;
    if existing.is_builtin {
        return Err("Cannot delete a built-in template.".into());
    }
    storage_delete(&id).await
}

#[tauri::command]
pub async fn clone_template(
    id: String,
    payload: CloneTemplateRequest,
) -> Result<PipelineTemplate, String> {
    let source = get_template(id).await?;
    let now = now_iso();
    let template = PipelineTemplate {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.new_name,
        description: source.description,
        is_builtin: false,
        max_iterations: source.max_iterations,
        stop_on_first_pass: source.stop_on_first_pass,
        nodes: source.nodes,
        edges: source.edges,
        created_at: now.clone(),
        updated_at: now,
    };
    let errors = validate_template(&template);
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    check_name_unique(&template.name, None).await?;
    write_template(&template).await?;
    Ok(template)
}

#[tauri::command]
pub async fn enhance_prompt(
    draft: String,
    provider: String,
    model: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let guard = state.hive_client.lock().await;
    let client = guard.as_ref().ok_or("hive-api client not initialised")?;
    let request = ChatRequest {
        provider,
        model,
        workspace_path: ".".into(),
        mode: "new".into(),
        prompt: format!("{ENHANCE_META_PROMPT}{draft}"),
        stream: true,
        provider_session_ref: None,
        provider_options: HashMap::new(),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let result = client.chat_stream(&request, cancel, |_event| {}).await?;
    Ok(result.final_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enhance_meta_prompt_contains_template_vars() {
        assert!(ENHANCE_META_PROMPT.contains("{{task}}"));
        assert!(ENHANCE_META_PROMPT.contains("{{code_context}}"));
        assert!(ENHANCE_META_PROMPT.contains("{{previous_output}}"));
        assert!(ENHANCE_META_PROMPT.contains("{{file_list}}"));
        assert!(ENHANCE_META_PROMPT.contains("{{iteration_number}}"));
        assert!(ENHANCE_META_PROMPT.contains("{{test_results}}"));
    }

    #[test]
    fn create_request_deserialises_camel_case_graph_payload() {
        let json = r#"{"name":"My Template","description":"Does stuff","maxIterations":5,"stopOnFirstPass":true,"nodes":[{"id":"n1","label":"Analyse","stageType":"analyse","handler":"analyse","provider":"claude","model":"opus","sessionGroup":"A","promptTemplate":"Analyse {{task}}","enabled":true,"executionIntent":"text","config":null,"uiPosition":{"x":0.0,"y":0.0}}],"edges":[{"id":"e1","sourceNodeId":"n1","targetNodeId":"n1","condition":"always","inputKey":null}]}"#;
        let req: CreateTemplateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "My Template");
        assert_eq!(req.max_iterations, 5);
        assert!(req.stop_on_first_pass);
        assert_eq!(req.nodes.len(), 1);
        assert_eq!(req.edges.len(), 1);
        assert_eq!(req.edges[0].source_node_id, "n1");
    }

    #[test]
    fn update_request_deserialises_camel_case_graph_payload() {
        let json = r#"{"name":"Updated","description":"Changed","maxIterations":2,"stopOnFirstPass":false,"nodes":[],"edges":[]}"#;
        let req: UpdateTemplateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Updated");
        assert_eq!(req.max_iterations, 2);
        assert!(!req.stop_on_first_pass);
        assert!(req.nodes.is_empty());
        assert!(req.edges.is_empty());
    }

    #[test]
    fn clone_request_deserialises_camel_case() {
        let req: CloneTemplateRequest = serde_json::from_str(r#"{"newName":"Cloned Pipeline"}"#).unwrap();
        assert_eq!(req.new_name, "Cloned Pipeline");
    }
}
