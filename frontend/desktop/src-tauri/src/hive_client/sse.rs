use super::chat::HiveSseEvent;
use serde::Deserialize;

/// Parse a single SSE event from its event name and JSON data payload.
pub fn parse_sse_event(event_name: &str, data: &str) -> Result<HiveSseEvent, String> {
    match event_name {
        "run_started" => {
            #[derive(Deserialize)]
            struct D {
                provider: String,
                model: String,
                job_id: String,
            }
            let d: D = serde_json::from_str(data).map_err(|e| e.to_string())?;
            Ok(HiveSseEvent::RunStarted {
                provider: d.provider,
                model: d.model,
                job_id: d.job_id,
            })
        }
        "provider_session" => {
            #[derive(Deserialize)]
            struct D {
                provider_session_ref: String,
            }
            let d: D = serde_json::from_str(data).map_err(|e| e.to_string())?;
            Ok(HiveSseEvent::ProviderSession {
                provider_session_ref: d.provider_session_ref,
            })
        }
        "output_delta" => {
            #[derive(Deserialize)]
            struct D {
                text: String,
            }
            let d: D = serde_json::from_str(data).map_err(|e| e.to_string())?;
            Ok(HiveSseEvent::OutputDelta { text: d.text })
        }
        "completed" => {
            #[derive(Deserialize)]
            struct D {
                final_text: String,
                exit_code: i32,
                #[serde(default)]
                provider_session_ref: Option<String>,
                #[serde(default)]
                warnings: Vec<String>,
            }
            let d: D = serde_json::from_str(data).map_err(|e| e.to_string())?;
            Ok(HiveSseEvent::Completed {
                final_text: d.final_text,
                exit_code: d.exit_code,
                provider_session_ref: d.provider_session_ref,
                warnings: d.warnings,
            })
        }
        "failed" => {
            #[derive(Deserialize)]
            struct D {
                error: String,
                exit_code: i32,
                #[serde(default)]
                warnings: Vec<String>,
            }
            let d: D = serde_json::from_str(data).map_err(|e| e.to_string())?;
            Ok(HiveSseEvent::Failed {
                error: d.error,
                exit_code: d.exit_code,
                warnings: d.warnings,
            })
        }
        "stopped" => {
            #[derive(Deserialize)]
            struct D {
                provider: String,
                model: String,
                job_id: String,
            }
            let d: D = serde_json::from_str(data).map_err(|e| e.to_string())?;
            Ok(HiveSseEvent::Stopped {
                provider: d.provider,
                model: d.model,
                job_id: d.job_id,
            })
        }
        _ => Err(format!("Unknown SSE event: {}", event_name)),
    }
}

/// Buffered SSE line parser. Accumulates lines and emits complete events.
pub struct SseLineParser {
    current_event: Option<String>,
    current_data: Vec<String>,
}

impl SseLineParser {
    pub fn new() -> Self {
        Self {
            current_event: None,
            current_data: Vec::new(),
        }
    }

    /// Feed a single line (without trailing newline) to the parser.
    /// Returns Some(event) when a complete event is ready.
    pub fn feed_line(&mut self, line: &str) -> Result<Option<HiveSseEvent>, String> {
        if line.is_empty() {
            // Blank line = end of event
            if let Some(event_name) = self.current_event.take() {
                let data = self.current_data.join("\n");
                self.current_data.clear();
                if data.is_empty() {
                    return Ok(None);
                }
                return parse_sse_event(&event_name, &data).map(Some);
            }
            self.current_data.clear();
            return Ok(None);
        }

        if let Some(rest) = line.strip_prefix("event:") {
            self.current_event = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("data:") {
            self.current_data.push(rest.trim().to_string());
        }
        // Ignore comment lines (starting with ':') and unknown fields

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_run_started() {
        let data = r#"{"provider": "claude", "model": "opus-4", "job_id": "abc-123"}"#;
        let event = parse_sse_event("run_started", data).unwrap();
        match event {
            HiveSseEvent::RunStarted {
                provider,
                model,
                job_id,
            } => {
                assert_eq!(provider, "claude");
                assert_eq!(model, "opus-4");
                assert_eq!(job_id, "abc-123");
            }
            _ => panic!("Expected RunStarted"),
        }
    }

    #[test]
    fn test_parse_provider_session() {
        let data = r#"{"provider_session_ref": "session-456"}"#;
        let event = parse_sse_event("provider_session", data).unwrap();
        match event {
            HiveSseEvent::ProviderSession {
                provider_session_ref,
            } => {
                assert_eq!(provider_session_ref, "session-456");
            }
            _ => panic!("Expected ProviderSession"),
        }
    }

    #[test]
    fn test_parse_output_delta() {
        let data = r#"{"text": "Hello world"}"#;
        let event = parse_sse_event("output_delta", data).unwrap();
        match event {
            HiveSseEvent::OutputDelta { text } => {
                assert_eq!(text, "Hello world");
            }
            _ => panic!("Expected OutputDelta"),
        }
    }

    #[test]
    fn test_parse_completed() {
        let data = r#"{
            "final_text": "Done!",
            "exit_code": 0,
            "provider_session_ref": "sess-1",
            "warnings": ["w1"]
        }"#;
        let event = parse_sse_event("completed", data).unwrap();
        match event {
            HiveSseEvent::Completed {
                final_text,
                exit_code,
                provider_session_ref,
                warnings,
            } => {
                assert_eq!(final_text, "Done!");
                assert_eq!(exit_code, 0);
                assert_eq!(provider_session_ref.unwrap(), "sess-1");
                assert_eq!(warnings, vec!["w1"]);
            }
            _ => panic!("Expected Completed"),
        }
    }

    #[test]
    fn test_parse_completed_without_optional_fields() {
        let data = r#"{"final_text": "Done!", "exit_code": 0}"#;
        let event = parse_sse_event("completed", data).unwrap();
        match event {
            HiveSseEvent::Completed {
                provider_session_ref,
                warnings,
                ..
            } => {
                assert!(provider_session_ref.is_none());
                assert!(warnings.is_empty());
            }
            _ => panic!("Expected Completed"),
        }
    }

    #[test]
    fn test_parse_failed() {
        let data = r#"{"error": "timeout", "exit_code": 1, "warnings": []}"#;
        let event = parse_sse_event("failed", data).unwrap();
        match event {
            HiveSseEvent::Failed {
                error, exit_code, ..
            } => {
                assert_eq!(error, "timeout");
                assert_eq!(exit_code, 1);
            }
            _ => panic!("Expected Failed"),
        }
    }

    #[test]
    fn test_parse_stopped() {
        let data = r#"{"provider": "gemini", "model": "pro", "job_id": "j-1"}"#;
        let event = parse_sse_event("stopped", data).unwrap();
        match event {
            HiveSseEvent::Stopped {
                provider,
                model,
                job_id,
            } => {
                assert_eq!(provider, "gemini");
                assert_eq!(model, "pro");
                assert_eq!(job_id, "j-1");
            }
            _ => panic!("Expected Stopped"),
        }
    }

    #[test]
    fn test_parse_unknown_event() {
        let result = parse_sse_event("unknown_event", "{}");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown SSE event"));
    }

    #[test]
    fn test_sse_line_parser_full_event() {
        let mut parser = SseLineParser::new();
        assert!(parser.feed_line("event: output_delta").unwrap().is_none());
        assert!(parser
            .feed_line(r#"data: {"text": "hi"}"#)
            .unwrap()
            .is_none());
        let event = parser.feed_line("").unwrap().unwrap();
        match event {
            HiveSseEvent::OutputDelta { text } => assert_eq!(text, "hi"),
            _ => panic!("Expected OutputDelta"),
        }
    }

    #[test]
    fn test_sse_line_parser_ignores_comments() {
        let mut parser = SseLineParser::new();
        assert!(parser.feed_line(": keep-alive").unwrap().is_none());
        assert!(parser.feed_line("").unwrap().is_none());
    }

    #[test]
    fn test_sse_line_parser_blank_without_event() {
        let mut parser = SseLineParser::new();
        // Blank line with no preceding event should not error
        assert!(parser.feed_line("").unwrap().is_none());
    }

    #[test]
    fn test_sse_line_parser_multi_line_data() {
        // SSE spec allows multiple data: lines, joined by newlines
        let mut parser = SseLineParser::new();
        parser.feed_line("event: output_delta").unwrap();
        // Multi-line data fields — only the last one matters for JSON,
        // but the parser joins them with newlines.
        // For our use case, hive-api sends single-line JSON, so this
        // tests edge case handling.
        parser.feed_line(r#"data: {"text":"#).unwrap();
        parser.feed_line(r#"data:  "hello"}"#).unwrap();
        let result = parser.feed_line("").unwrap();
        // Joined data will be `{"text":\n "hello"}` which is valid JSON
        assert!(result.is_some());
    }

    #[test]
    fn test_sse_full_event_sequence() {
        // Simulate a realistic hive-api SSE stream for a successful run
        let lines = vec![
            "event: run_started",
            r#"data: {"provider":"claude","model":"opus","job_id":"j-1"}"#,
            "",
            "event: provider_session",
            r#"data: {"provider_session_ref":"sess-abc"}"#,
            "",
            "event: output_delta",
            r#"data: {"text":"Analysing..."}"#,
            "",
            "event: output_delta",
            r#"data: {"text":" Done."}"#,
            "",
            "event: completed",
            r#"data: {"final_text":"Analysing... Done.","exit_code":0,"provider_session_ref":"sess-abc","warnings":[]}"#,
            "",
        ];

        let mut parser = SseLineParser::new();
        let mut events = Vec::new();
        for line in lines {
            if let Some(event) = parser.feed_line(line).unwrap() {
                events.push(event);
            }
        }

        assert_eq!(events.len(), 5);
        assert!(matches!(&events[0], HiveSseEvent::RunStarted { job_id, .. } if job_id == "j-1"));
        assert!(matches!(&events[1], HiveSseEvent::ProviderSession { provider_session_ref } if provider_session_ref == "sess-abc"));
        assert!(matches!(&events[2], HiveSseEvent::OutputDelta { text } if text == "Analysing..."));
        assert!(matches!(&events[3], HiveSseEvent::OutputDelta { text } if text == " Done."));
        assert!(matches!(&events[4], HiveSseEvent::Completed { exit_code: 0, .. }));
    }

    #[test]
    fn test_sse_failed_event_sequence() {
        let lines = vec![
            "event: run_started",
            r#"data: {"provider":"gemini","model":"pro","job_id":"j-2"}"#,
            "",
            "event: output_delta",
            r#"data: {"text":"Starting..."}"#,
            "",
            "event: failed",
            r#"data: {"error":"CLI process crashed","exit_code":1,"warnings":["partial output"]}"#,
            "",
        ];

        let mut parser = SseLineParser::new();
        let mut events = Vec::new();
        for line in lines {
            if let Some(event) = parser.feed_line(line).unwrap() {
                events.push(event);
            }
        }

        assert_eq!(events.len(), 3);
        assert!(matches!(&events[2], HiveSseEvent::Failed { error, exit_code: 1, warnings }
            if error == "CLI process crashed" && warnings.len() == 1));
    }

    #[test]
    fn test_sse_stopped_event_sequence() {
        let lines = vec![
            "event: run_started",
            r#"data: {"provider":"claude","model":"sonnet","job_id":"j-3"}"#,
            "",
            "event: stopped",
            r#"data: {"provider":"claude","model":"sonnet","job_id":"j-3"}"#,
            "",
        ];

        let mut parser = SseLineParser::new();
        let mut events = Vec::new();
        for line in lines {
            if let Some(event) = parser.feed_line(line).unwrap() {
                events.push(event);
            }
        }

        assert_eq!(events.len(), 2);
        assert!(matches!(&events[1], HiveSseEvent::Stopped { job_id, .. } if job_id == "j-3"));
    }
}
