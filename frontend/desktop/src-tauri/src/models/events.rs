use serde::{Deserialize, Serialize};

/// Status reported when a stage finishes execution.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StageEndStatus {
    pub stage_id: String,
    pub label: String,
    /// "completed" | "failed" | "cancelled" | "skipped"
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Overall status of a pipeline run.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

/// All events emitted during a pipeline run.
/// Persisted to `events.jsonl` as append-only log.
///
/// Variant tags are snake_case (via explicit `#[serde(rename)]`).
/// Field names are camelCase (via per-field `#[serde(rename)]`).
/// Note: serde's `rename_all` on internally-tagged enums does not
/// rename variant fields, so we rename them explicitly.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum RunEvent {
    #[serde(rename = "run_started")]
    RunStarted {
        #[serde(rename = "runId")]
        run_id: String,
        timestamp: String,
    },
    #[serde(rename = "stage_started")]
    StageStarted {
        #[serde(rename = "stageId")]
        stage_id: String,
        label: String,
        timestamp: String,
    },
    #[serde(rename = "stage_log")]
    StageLog {
        #[serde(rename = "stageId")]
        stage_id: String,
        text: String,
        timestamp: String,
    },
    #[serde(rename = "stage_ended")]
    StageEnded {
        #[serde(rename = "stageId")]
        stage_id: String,
        label: String,
        status: StageEndStatus,
        timestamp: String,
    },
    #[serde(rename = "artifact")]
    Artifact {
        #[serde(rename = "stageId")]
        stage_id: String,
        name: String,
        content: String,
        #[serde(rename = "artifactType")]
        artifact_type: String,
        timestamp: String,
    },
    #[serde(rename = "session_ref")]
    SessionRef {
        #[serde(rename = "sessionGroup")]
        session_group: String,
        #[serde(rename = "providerSessionRef")]
        provider_session_ref: String,
        timestamp: String,
    },
    #[serde(rename = "question")]
    Question {
        #[serde(rename = "questionId")]
        question_id: String,
        #[serde(rename = "questionText")]
        question_text: String,
        timestamp: String,
    },
    #[serde(rename = "answer")]
    Answer {
        #[serde(rename = "questionId")]
        question_id: String,
        #[serde(rename = "answerText")]
        answer_text: String,
        timestamp: String,
    },
    #[serde(rename = "iteration_completed")]
    IterationCompleted {
        iteration: u32,
        verdict: String,
        timestamp: String,
    },
    #[serde(rename = "run_ended")]
    RunEnded {
        status: RunStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
        timestamp: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_event_tagged_enum_serialises_with_type_field() {
        let event = RunEvent::StageStarted {
            stage_id: "s1".into(),
            label: "Analyse".into(),
            timestamp: "2026-03-23T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        // Type tag is snake_case, field names are camelCase
        assert!(json.contains(r#""type":"stage_started""#));
        assert!(json.contains(r#""stageId":"s1""#));
        assert!(json.contains(r#""label":"Analyse""#));
        // Must NOT contain snake_case field names
        assert!(!json.contains("stage_id"));
    }

    #[test]
    fn run_event_round_trip_all_variants() {
        let events: Vec<RunEvent> = vec![
            RunEvent::RunStarted {
                run_id: "r1".into(),
                timestamp: "2026-03-23T12:00:00Z".into(),
            },
            RunEvent::StageLog {
                stage_id: "s1".into(),
                text: "Processing...".into(),
                timestamp: "2026-03-23T12:00:01Z".into(),
            },
            RunEvent::SessionRef {
                session_group: "A".into(),
                provider_session_ref: "abc-123".into(),
                timestamp: "2026-03-23T12:00:02Z".into(),
            },
            RunEvent::IterationCompleted {
                iteration: 1,
                verdict: "complete".into(),
                timestamp: "2026-03-23T12:00:03Z".into(),
            },
            RunEvent::RunEnded {
                status: RunStatus::Completed,
                reason: Some("All iterations passed".into()),
                timestamp: "2026-03-23T12:00:04Z".into(),
            },
        ];
        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            let restored: RunEvent = serde_json::from_str(&json).unwrap();
            let re_json = serde_json::to_string(&restored).unwrap();
            assert_eq!(json, re_json);
        }
    }

    #[test]
    fn run_status_serialises_snake_case() {
        assert_eq!(serde_json::to_string(&RunStatus::Running).unwrap(), r#""running""#);
        assert_eq!(serde_json::to_string(&RunStatus::Completed).unwrap(), r#""completed""#);
        assert_eq!(serde_json::to_string(&RunStatus::Failed).unwrap(), r#""failed""#);
        assert_eq!(serde_json::to_string(&RunStatus::Cancelled).unwrap(), r#""cancelled""#);
        assert_eq!(serde_json::to_string(&RunStatus::Paused).unwrap(), r#""paused""#);
    }

    #[test]
    fn stage_end_status_nested_in_event() {
        let event = RunEvent::StageEnded {
            stage_id: "s1".into(),
            label: "Review".into(),
            status: StageEndStatus {
                stage_id: "s1".into(),
                label: "Review".into(),
                status: "completed".into(),
                output: Some("LGTM".into()),
                duration_ms: Some(12345),
            },
            timestamp: "2026-03-23T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let restored: RunEvent = serde_json::from_str(&json).unwrap();
        if let RunEvent::StageEnded { status, .. } = restored {
            assert_eq!(status.output, Some("LGTM".into()));
            assert_eq!(status.duration_ms, Some(12345));
        } else {
            panic!("Expected StageEnded variant");
        }
    }
}
