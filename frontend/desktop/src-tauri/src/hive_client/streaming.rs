use super::chat::HiveSseEvent;
use super::error::map_hive_error;
use super::sse::SseLineParser;
use super::HiveClient;
use futures_util::StreamExt;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// UTF-8 safe byte buffer that handles multibyte chars split across chunks.
struct Utf8Buffer {
    pending: Vec<u8>,
}

impl Utf8Buffer {
    fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    /// Push raw bytes and return as much valid UTF-8 as possible.
    /// Incomplete trailing sequences are kept in the buffer for the next push.
    fn push(&mut self, bytes: &[u8]) -> Result<String, String> {
        self.pending.extend_from_slice(bytes);
        match String::from_utf8(self.pending.clone()) {
            Ok(s) => {
                self.pending.clear();
                Ok(s)
            }
            Err(e) => {
                let valid_up_to = e.utf8_error().valid_up_to();
                if valid_up_to == 0 && self.pending.len() >= 4 {
                    // 4+ bytes and still invalid — not a split char, genuinely bad data
                    return Err("Invalid UTF-8 in SSE stream".to_string());
                }
                let valid = String::from_utf8(self.pending[..valid_up_to].to_vec())
                    .map_err(|e| format!("UTF-8 decode error: {}", e))?;
                self.pending = self.pending[valid_up_to..].to_vec();
                Ok(valid)
            }
        }
    }
}

/// Request body for POST /v1/chat.
#[derive(Debug, Serialize, Clone)]
pub struct ChatRequest {
    pub provider: String,
    pub model: String,
    pub workspace_path: String,
    pub mode: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_session_ref: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub provider_options: HashMap<String, serde_json::Value>,
}

/// Result of a completed chat request.
#[derive(Debug, Clone)]
pub struct ChatResult {
    pub final_text: String,
    pub provider_session_ref: Option<String>,
    pub job_id: Option<String>,
}

impl HiveClient {
    /// Send a chat request and consume the SSE stream.
    /// Calls `on_event` for each parsed SSE event.
    /// Checks cancel_flag between events.
    /// Returns ChatResult on success.
    pub async fn chat_stream(
        &self,
        request: &ChatRequest,
        cancel_flag: Arc<AtomicBool>,
        on_event: impl Fn(&HiveSseEvent),
    ) -> Result<ChatResult, String> {
        let url = format!("{}/v1/chat", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    "hive-api is not running. Check that it's started.".to_string()
                } else {
                    format!("Failed to connect to hive-api: {}", e)
                }
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(map_hive_error(status, &body));
        }

        self.consume_sse_stream(resp, cancel_flag, on_event).await
    }

    /// Cancel a running or queued job.
    pub async fn cancel_job(&self, job_id: &str) -> Result<(), String> {
        let url = format!("{}/v1/chat/{}/stop", self.base_url, job_id);
        let resp = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to cancel job: {}", e))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("Cancel failed: HTTP {}", resp.status()))
        }
    }

    /// Consume an SSE response stream, parsing events and invoking the callback.
    async fn consume_sse_stream(
        &self,
        resp: reqwest::Response,
        cancel_flag: Arc<AtomicBool>,
        on_event: impl Fn(&HiveSseEvent),
    ) -> Result<ChatResult, String> {
        let mut stream = resp.bytes_stream();
        let mut parser = SseLineParser::new();
        let mut utf8_buf = Utf8Buffer::new();
        let mut line_buf = String::new();
        let mut job_id: Option<String> = None;
        let mut result: Option<ChatResult> = None;

        while let Some(chunk) = stream.next().await {
            if cancel_flag.load(Ordering::Relaxed) {
                if let Some(ref jid) = job_id {
                    let _ = self.cancel_job(jid).await;
                }
                return Err("Pipeline cancelled".to_string());
            }

            let bytes = chunk.map_err(|e| format!("Stream read error: {}", e))?;
            let text = utf8_buf.push(&bytes)?;
            line_buf.push_str(&text);

            while let Some(newline_pos) = line_buf.find('\n') {
                let line = line_buf[..newline_pos].trim_end_matches('\r').to_string();
                line_buf = line_buf[newline_pos + 1..].to_string();

                if let Some(event) = parser.feed_line(&line)? {
                    match &event {
                        HiveSseEvent::RunStarted { job_id: jid, .. } => {
                            job_id = Some(jid.clone());
                        }
                        HiveSseEvent::Completed {
                            final_text,
                            provider_session_ref,
                            ..
                        } => {
                            result = Some(ChatResult {
                                final_text: final_text.clone(),
                                provider_session_ref: provider_session_ref.clone(),
                                job_id: job_id.clone(),
                            });
                        }
                        HiveSseEvent::Failed { error, .. } => {
                            on_event(&event);
                            return Err(format!("Agent execution failed: {}", error));
                        }
                        HiveSseEvent::Stopped { .. } => {
                            on_event(&event);
                            return Err("Job was stopped".to_string());
                        }
                        _ => {}
                    }
                    on_event(&event);
                }
            }
        }

        result.ok_or_else(|| "Stream ended without a completed event".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_serialisation() {
        let req = ChatRequest {
            provider: "claude".into(),
            model: "opus-4".into(),
            workspace_path: "/home/user/project".into(),
            mode: "new".into(),
            prompt: "Fix the bug".into(),
            stream: true,
            provider_session_ref: None,
            provider_options: HashMap::new(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["provider"], "claude");
        assert_eq!(json["model"], "opus-4");
        assert_eq!(json["stream"], true);
        assert!(json.get("provider_session_ref").is_none());
        assert!(json.get("provider_options").is_none());
    }

    #[test]
    fn test_chat_request_with_session_ref() {
        let req = ChatRequest {
            provider: "gemini".into(),
            model: "pro".into(),
            workspace_path: "/tmp".into(),
            mode: "resume".into(),
            prompt: "Continue".into(),
            stream: true,
            provider_session_ref: Some("sess-123".into()),
            provider_options: HashMap::new(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["provider_session_ref"], "sess-123");
        assert_eq!(json["mode"], "resume");
    }

    #[test]
    fn test_chat_request_with_provider_options() {
        let mut opts = HashMap::new();
        opts.insert(
            "temperature".into(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()),
        );
        let req = ChatRequest {
            provider: "claude".into(),
            model: "opus-4".into(),
            workspace_path: "/tmp".into(),
            mode: "new".into(),
            prompt: "Hello".into(),
            stream: true,
            provider_session_ref: None,
            provider_options: opts,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["provider_options"]["temperature"], 0.7);
    }

    #[test]
    fn test_utf8_buffer_handles_split_multibyte() {
        let mut buf = Utf8Buffer::new();
        // "é" is 0xC3 0xA9 in UTF-8 — split across two pushes
        let full = "café".as_bytes();
        let split_at = full.len() - 1; // split in the middle of é
        let part1 = &full[..split_at];
        let part2 = &full[split_at..];

        let s1 = buf.push(part1).unwrap();
        assert_eq!(s1, "caf"); // valid up to the incomplete char

        let s2 = buf.push(part2).unwrap();
        assert_eq!(s2, "é"); // completes the char
    }

    #[test]
    fn test_utf8_buffer_handles_complete_chunks() {
        let mut buf = Utf8Buffer::new();
        let s = buf.push("hello world".as_bytes()).unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_utf8_buffer_handles_emoji_split() {
        let mut buf = Utf8Buffer::new();
        // 🚀 is 4 bytes: F0 9F 9A 80
        let emoji_bytes = "🚀".as_bytes();
        assert_eq!(emoji_bytes.len(), 4);

        // Feed 2 bytes at a time
        let s1 = buf.push(&emoji_bytes[..2]).unwrap();
        assert_eq!(s1, ""); // incomplete, buffered

        let s2 = buf.push(&emoji_bytes[2..]).unwrap();
        assert_eq!(s2, "🚀"); // now complete
    }

    #[test]
    fn test_chat_request_uses_snake_case_for_hive_api() {
        let req = ChatRequest {
            provider: "claude".into(),
            model: "opus".into(),
            workspace_path: "/project".into(),
            mode: "resume".into(),
            prompt: "test".into(),
            stream: true,
            provider_session_ref: Some("ref-1".into()),
            provider_options: HashMap::new(),
        };
        let json = serde_json::to_string(&req).unwrap();
        // hive-api expects snake_case field names
        assert!(json.contains("workspace_path"));
        assert!(json.contains("provider_session_ref"));
        // Must NOT contain camelCase
        assert!(!json.contains("workspacePath"));
        assert!(!json.contains("providerSessionRef"));
    }
}
