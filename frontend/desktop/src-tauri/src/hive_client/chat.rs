use serde::{Deserialize, Serialize};

/// Events received over the SSE stream from hive-api during a chat/run.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HiveSseEvent {
    RunStarted {
        provider: String,
        model: String,
        job_id: String,
    },
    ProviderSession {
        provider_session_ref: String,
    },
    OutputDelta {
        text: String,
    },
    Completed {
        final_text: String,
        exit_code: i32,
        #[serde(default)]
        provider_session_ref: Option<String>,
        #[serde(default)]
        warnings: Vec<String>,
    },
    Failed {
        error: String,
        exit_code: i32,
        #[serde(default)]
        warnings: Vec<String>,
    },
    Stopped {
        provider: String,
        model: String,
        job_id: String,
    },
}
