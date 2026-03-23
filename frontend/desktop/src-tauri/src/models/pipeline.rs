use serde::{Deserialize, Serialize};

/// Overall pipeline execution status.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Execution status of an individual stage.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

/// Verdict returned by the judge stage at the end of each iteration.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JudgeVerdict {
    Complete,
    NotComplete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_status_serialises_snake_case() {
        assert_eq!(serde_json::to_string(&PipelineStatus::Idle).unwrap(), r#""idle""#);
        assert_eq!(serde_json::to_string(&PipelineStatus::Running).unwrap(), r#""running""#);
        assert_eq!(serde_json::to_string(&PipelineStatus::Paused).unwrap(), r#""paused""#);
        assert_eq!(serde_json::to_string(&PipelineStatus::Completed).unwrap(), r#""completed""#);
        assert_eq!(serde_json::to_string(&PipelineStatus::Failed).unwrap(), r#""failed""#);
        assert_eq!(serde_json::to_string(&PipelineStatus::Cancelled).unwrap(), r#""cancelled""#);
    }

    #[test]
    fn stage_status_round_trip() {
        let statuses = vec![
            StageStatus::Pending,
            StageStatus::Running,
            StageStatus::Completed,
            StageStatus::Failed,
            StageStatus::Cancelled,
            StageStatus::Skipped,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let restored: StageStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, status);
        }
    }

    #[test]
    fn judge_verdict_serialises_snake_case() {
        assert_eq!(serde_json::to_string(&JudgeVerdict::Complete).unwrap(), r#""complete""#);
        assert_eq!(serde_json::to_string(&JudgeVerdict::NotComplete).unwrap(), r#""not_complete""#);
        // Verify round-trip
        let v: JudgeVerdict = serde_json::from_str(r#""not_complete""#).unwrap();
        assert_eq!(v, JudgeVerdict::NotComplete);
    }
}
