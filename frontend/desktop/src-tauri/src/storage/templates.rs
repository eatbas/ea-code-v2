use std::path::PathBuf;

use crate::models::templates::PipelineTemplate;

fn parse_template(contents: &str) -> Result<PipelineTemplate, serde_json::Error> {
    serde_json::from_str::<PipelineTemplate>(contents)
}

pub fn templates_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not resolve home directory")
        .join(".ea-code")
        .join("pipeline-templates")
}

pub async fn ensure_templates_dir() -> Result<(), String> {
    tokio::fs::create_dir_all(templates_dir())
        .await
        .map_err(|e| format!("Failed to create templates directory: {e}"))
}

pub async fn list_user_templates() -> Result<Vec<PipelineTemplate>, String> {
    let dir = templates_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = tokio::fs::read_dir(&dir)
        .await
        .map_err(|e| format!("Failed to read templates directory: {e}"))?;
    let mut templates = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {e}"))?
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let contents = match tokio::fs::read_to_string(&path).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: skipping unreadable template {}: {e}", path.display());
                continue;
            }
        };
        match parse_template(&contents) {
            Ok(template) => templates.push(template),
            Err(e) => eprintln!("Warning: skipping malformed template {}: {e}", path.display()),
        }
    }
    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(templates)
}

pub async fn read_template(id: &str) -> Result<PipelineTemplate, String> {
    let path = templates_dir().join(format!("{id}.json"));
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Template '{id}' not found: {e}"))?;
    parse_template(&contents).map_err(|e| format!("Failed to parse template '{id}': {e}"))
}

pub async fn write_template(template: &PipelineTemplate) -> Result<(), String> {
    ensure_templates_dir().await?;
    let dir = templates_dir();
    let final_path = dir.join(format!("{}.json", template.id));
    let tmp_path = dir.join(format!("{}.json.tmp", template.id));
    let json = serde_json::to_string_pretty(template)
        .map_err(|e| format!("Failed to serialise template: {e}"))?;
    tokio::fs::write(&tmp_path, &json)
        .await
        .map_err(|e| format!("Failed to write temp file: {e}"))?;
    tokio::fs::rename(&tmp_path, &final_path)
        .await
        .map_err(|e| format!("Failed to rename temp file: {e}"))?;
    Ok(())
}

pub async fn delete_template(id: &str) -> Result<(), String> {
    tokio::fs::remove_file(templates_dir().join(format!("{id}.json")))
        .await
        .map_err(|e| format!("Failed to delete template '{id}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::templates::{
        EdgeCondition, PipelineTemplate, TemplateEdge, TemplateNode, UiPosition,
    };
    use std::env;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_node(id: &str, x: f64) -> TemplateNode {
        TemplateNode {
            id: id.into(),
            label: format!("Node {id}"),
            stage_type: "analyse".into(),
            handler: "analyse".into(),
            provider: "claude".into(),
            model: "opus".into(),
            session_group: "A".into(),
            prompt_template: "Do the thing: {{task}}".into(),
            enabled: true,
            execution_intent: "text".into(),
            config: None,
            ui_position: UiPosition { x, y: 0.0 },
        }
    }

    fn test_template(id: &str) -> PipelineTemplate {
        PipelineTemplate {
            id: id.into(),
            name: format!("Test {id}"),
            description: "Test template".into(),
            is_builtin: false,
            max_iterations: 3,
            stop_on_first_pass: true,
            nodes: vec![test_node("n1", 0.0)],
            edges: vec![TemplateEdge {
                id: "e1".into(),
                source_node_id: "n1".into(),
                target_node_id: "n1".into(),
                condition: EdgeCondition::Always,
                input_key: None,
                loop_control: false,
            }],
            created_at: "2026-03-23T12:00:00Z".into(),
            updated_at: "2026-03-23T12:00:00Z".into(),
        }
    }

    fn setup_test_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        env::temp_dir()
            .join("ea-code-test-templates")
            .join(format!("test-{}-{}", std::process::id(), n))
    }

    async fn write_to_dir(dir: &PathBuf, template: &PipelineTemplate) -> Result<(), String> {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(|e| format!("mkdir: {e}"))?;
        let path = dir.join(format!("{}.json", template.id));
        let json = serde_json::to_string_pretty(template).map_err(|e| format!("ser: {e}"))?;
        tokio::fs::write(&path, &json)
            .await
            .map_err(|e| format!("write: {e}"))?;
        Ok(())
    }

    async fn read_from_dir(dir: &PathBuf, id: &str) -> Result<PipelineTemplate, String> {
        let path = dir.join(format!("{id}.json"));
        let contents = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| format!("read: {e}"))?;
        parse_template(&contents).map_err(|e| format!("parse: {e}"))
    }

    async fn list_from_dir(dir: &PathBuf) -> Result<Vec<PipelineTemplate>, String> {
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut entries = tokio::fs::read_dir(dir)
            .await
            .map_err(|e| format!("readdir: {e}"))?;
        let mut templates = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("entry: {e}"))?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let contents = match tokio::fs::read_to_string(&path).await {
                Ok(c) => c,
                Err(_) => continue,
            };
            match parse_template(&contents) {
                Ok(t) => templates.push(t),
                Err(_) => continue,
            }
        }
        templates.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(templates)
    }

    async fn delete_from_dir(dir: &PathBuf, id: &str) -> Result<(), String> {
        tokio::fs::remove_file(dir.join(format!("{id}.json")))
            .await
            .map_err(|e| format!("delete: {e}"))
    }

    #[tokio::test]
    async fn round_trip_write_read() {
        let dir = setup_test_dir();
        let tpl = test_template("rt-1");
        write_to_dir(&dir, &tpl).await.unwrap();
        let loaded = read_from_dir(&dir, "rt-1").await.unwrap();
        assert_eq!(loaded.id, "rt-1");
        assert_eq!(loaded.name, tpl.name);
        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.edges.len(), 1);
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn list_returns_all_written() {
        let dir = setup_test_dir();
        write_to_dir(&dir, &test_template("list-a")).await.unwrap();
        write_to_dir(&dir, &test_template("list-b")).await.unwrap();
        assert_eq!(list_from_dir(&dir).await.unwrap().len(), 2);
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn delete_removes_file() {
        let dir = setup_test_dir();
        write_to_dir(&dir, &test_template("del-1")).await.unwrap();
        delete_from_dir(&dir, "del-1").await.unwrap();
        assert!(read_from_dir(&dir, "del-1").await.is_err());
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn read_nonexistent_returns_error() {
        let dir = setup_test_dir();
        tokio::fs::create_dir_all(&dir).await.unwrap();
        assert!(read_from_dir(&dir, "does-not-exist").await.is_err());
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn malformed_json_is_skipped_in_listing() {
        let dir = setup_test_dir();
        tokio::fs::create_dir_all(&dir).await.unwrap();
        write_to_dir(&dir, &test_template("good-1")).await.unwrap();
        tokio::fs::write(dir.join("bad-template.json"), "{ this is not valid json }")
            .await
            .unwrap();
        let all = list_from_dir(&dir).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "good-1");
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn legacy_template_json_is_readable_and_listed() {
        let dir = setup_test_dir();
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let legacy_json = r#"{"id":"legacy-1","name":"Legacy Template","description":"Legacy stage-based payload","isBuiltin":false,"maxIterations":2,"stopOnFirstPass":true,"stages":[{"id":"s1","label":"Analyse","stageType":"analyse","position":0,"provider":"claude","model":"opus","sessionGroup":"A","parallelGroup":null,"promptTemplate":"Do the thing: {{task}}","enabled":true,"executionIntent":"text"}],"createdAt":"2026-03-23T12:00:00Z","updatedAt":"2026-03-23T12:00:00Z"}"#;
        tokio::fs::write(dir.join("legacy-1.json"), legacy_json)
            .await
            .unwrap();
        let loaded = read_from_dir(&dir, "legacy-1").await.unwrap();
        assert_eq!(loaded.id, "legacy-1");
        assert_eq!(loaded.nodes.len(), 1);
        let listed = list_from_dir(&dir).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "legacy-1");
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
