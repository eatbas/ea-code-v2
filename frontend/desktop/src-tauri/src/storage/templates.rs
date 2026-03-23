use std::path::PathBuf;

use crate::models::templates::PipelineTemplate;

/// Returns the directory where user pipeline templates are stored.
pub fn templates_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not resolve home directory")
        .join(".ea-code")
        .join("pipeline-templates")
}

/// Creates the templates directory if it does not exist.
pub async fn ensure_templates_dir() -> Result<(), String> {
    let dir = templates_dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("Failed to create templates directory: {e}"))
}

/// Lists all user-created pipeline templates from disk.
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
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let contents = match tokio::fs::read_to_string(&path).await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: skipping unreadable template {}: {e}", path.display());
                    continue;
                }
            };
            match serde_json::from_str::<PipelineTemplate>(&contents) {
                Ok(template) => templates.push(template),
                Err(e) => {
                    eprintln!("Warning: skipping malformed template {}: {e}", path.display());
                    continue;
                }
            }
        }
    }

    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(templates)
}

/// Reads a single pipeline template by ID.
pub async fn read_template(id: &str) -> Result<PipelineTemplate, String> {
    let path = templates_dir().join(format!("{id}.json"));
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Template '{id}' not found: {e}"))?;
    serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse template '{id}': {e}"))
}

/// Writes a pipeline template to disk using atomic write (tmp + rename).
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

/// Deletes a pipeline template by ID.
pub async fn delete_template(id: &str) -> Result<(), String> {
    let path = templates_dir().join(format!("{id}.json"));
    tokio::fs::remove_file(&path)
        .await
        .map_err(|e| format!("Failed to delete template '{id}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::templates::{PipelineTemplate, StageDefinition};
    use std::env;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_stage(id: &str, position: u32) -> StageDefinition {
        StageDefinition {
            id: id.into(),
            label: format!("Stage {id}"),
            stage_type: "analyse".into(),
            position,
            provider: "claude".into(),
            model: "opus".into(),
            session_group: "A".into(),
            parallel_group: None,
            prompt_template: "Do the thing: {{task}}".into(),
            enabled: true,
            execution_intent: "text".into(),
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
            stages: vec![test_stage("s1", 0)],
            created_at: "2026-03-23T12:00:00Z".into(),
            updated_at: "2026-03-23T12:00:00Z".into(),
        }
    }

    /// Override the templates dir for test isolation via env var.
    fn setup_test_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = env::temp_dir()
            .join("ea-code-test-templates")
            .join(format!("test-{}-{}", std::process::id(), n));
        // Set env so templates_dir() isn't used directly; we override in tests.
        dir
    }

    /// Write template directly to a specific dir (bypasses templates_dir()).
    async fn write_to_dir(dir: &PathBuf, template: &PipelineTemplate) -> Result<(), String> {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(|e| format!("mkdir: {e}"))?;
        let path = dir.join(format!("{}.json", template.id));
        let json = serde_json::to_string_pretty(template)
            .map_err(|e| format!("ser: {e}"))?;
        tokio::fs::write(&path, &json)
            .await
            .map_err(|e| format!("write: {e}"))?;
        Ok(())
    }

    async fn read_from_dir(
        dir: &PathBuf,
        id: &str,
    ) -> Result<PipelineTemplate, String> {
        let path = dir.join(format!("{id}.json"));
        let contents = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| format!("read: {e}"))?;
        serde_json::from_str(&contents).map_err(|e| format!("parse: {e}"))
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
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let contents = match tokio::fs::read_to_string(&path).await {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                match serde_json::from_str::<PipelineTemplate>(&contents) {
                    Ok(t) => templates.push(t),
                    Err(_) => continue,
                }
            }
        }
        templates.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(templates)
    }

    async fn delete_from_dir(dir: &PathBuf, id: &str) -> Result<(), String> {
        let path = dir.join(format!("{id}.json"));
        tokio::fs::remove_file(&path)
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
        assert_eq!(loaded.stages.len(), 1);
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn list_returns_all_written() {
        let dir = setup_test_dir();
        write_to_dir(&dir, &test_template("list-a")).await.unwrap();
        write_to_dir(&dir, &test_template("list-b")).await.unwrap();
        let all = list_from_dir(&dir).await.unwrap();
        assert_eq!(all.len(), 2);
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn delete_removes_file() {
        let dir = setup_test_dir();
        write_to_dir(&dir, &test_template("del-1")).await.unwrap();
        delete_from_dir(&dir, "del-1").await.unwrap();
        let result = read_from_dir(&dir, "del-1").await;
        assert!(result.is_err());
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn read_nonexistent_returns_error() {
        let dir = setup_test_dir();
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let result = read_from_dir(&dir, "does-not-exist").await;
        assert!(result.is_err());
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn malformed_json_is_skipped_in_listing() {
        let dir = setup_test_dir();
        tokio::fs::create_dir_all(&dir).await.unwrap();

        // Write one valid template
        write_to_dir(&dir, &test_template("good-1")).await.unwrap();

        // Write a malformed JSON file
        let bad_path = dir.join("bad-template.json");
        tokio::fs::write(&bad_path, "{ this is not valid json }")
            .await
            .unwrap();

        // listing should return only the good template, not error
        let all = list_from_dir(&dir).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "good-1");
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
