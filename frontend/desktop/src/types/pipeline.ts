export type PipelineStatus =
  | "idle"
  | "running"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

export type StageStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "cancelled"
  | "skipped";

export type JudgeVerdict = "complete" | "not_complete";
