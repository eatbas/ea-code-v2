/// Maps hive-api HTTP errors to user-facing error messages.
pub fn map_hive_error(status_code: u16, body: &str) -> String {
    match status_code {
        400 => format!("Provider not available: {}", extract_message(body)),
        404 => {
            "No drone available for this provider/model. Check provider configuration.".into()
        }
        500 => format!("Agent execution failed: {}", extract_message(body)),
        502 | 503 => "hive-api is temporarily unavailable. Retrying...".into(),
        _ => format!(
            "Unexpected hive-api error ({}): {}",
            status_code,
            extract_message(body)
        ),
    }
}

/// Try to extract the "detail" field from a JSON error response body.
fn extract_message(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("detail").and_then(|d| d.as_str().map(String::from)))
        .unwrap_or_else(|| body.chars().take(200).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_hive_error_400() {
        let body = r#"{"detail": "Model not supported"}"#;
        let msg = map_hive_error(400, body);
        assert_eq!(msg, "Provider not available: Model not supported");
    }

    #[test]
    fn test_map_hive_error_404() {
        let msg = map_hive_error(404, "");
        assert!(msg.contains("No drone available"));
    }

    #[test]
    fn test_map_hive_error_500() {
        let body = r#"{"detail": "Process exited with code 1"}"#;
        let msg = map_hive_error(500, body);
        assert_eq!(msg, "Agent execution failed: Process exited with code 1");
    }

    #[test]
    fn test_map_hive_error_502() {
        let msg = map_hive_error(502, "");
        assert!(msg.contains("temporarily unavailable"));
    }

    #[test]
    fn test_map_hive_error_503() {
        let msg = map_hive_error(503, "");
        assert!(msg.contains("temporarily unavailable"));
    }

    #[test]
    fn test_map_hive_error_unknown_status() {
        let msg = map_hive_error(418, "I'm a teapot");
        assert!(msg.contains("418"));
        assert!(msg.contains("I'm a teapot"));
    }

    #[test]
    fn test_extract_message_json() {
        let body = r#"{"detail": "Something went wrong"}"#;
        assert_eq!(extract_message(body), "Something went wrong");
    }

    #[test]
    fn test_extract_message_plain_text() {
        let body = "plain error text";
        assert_eq!(extract_message(body), "plain error text");
    }

    #[test]
    fn test_extract_message_truncates_long_body() {
        let body = "a".repeat(500);
        let msg = extract_message(&body);
        assert_eq!(msg.len(), 200);
    }
}
