export const PIPELINE_EVENTS = {
  STARTED: "pipeline:started",
  STAGE: "pipeline:stage",
  LOG: "pipeline:log",
  ARTIFACT: "pipeline:artifact",
  QUESTION: "pipeline:question",
  COMPLETED: "pipeline:completed",
  ERROR: "pipeline:error",
} as const;

export type PipelineMode = "graph" | "direct_task";
export type PipelineStageLiveStatus = "running" | "completed" | "failed" | "cancelled";
export type PipelineTerminalStatus = "completed" | "failed" | "cancelled";

/**
 * Payloads emitted in real time from Tauri on `pipeline:*` topics.
 * These are distinct from persisted `RunEvent` records.
 */
export interface PipelineStartedEventPayload {
  runId: string;
  mode: PipelineMode;
  templateId?: string;
  timestamp: string;
}

export interface PipelineStageEventPayload {
  runId: string;
  nodeId: string;
  nodeLabel: string;
  status: PipelineStageLiveStatus;
  timestamp: string;
  detail?: string;
}

export interface PipelineLogEventPayload {
  runId: string;
  nodeId: string;
  text: string;
  timestamp: string;
}

export interface PipelineArtifactEventPayload {
  runId: string;
  nodeId: string;
  name: string;
  artifactType: string;
  content: string;
  timestamp: string;
}

export interface PipelineQuestionEventPayload {
  runId: string;
  questionId: string;
  nodeId: string;
  questionText: string;
  timestamp: string;
}

export interface PipelineCompletedEventPayload {
  runId: string;
  status: PipelineTerminalStatus;
  timestamp: string;
}

export interface PipelineErrorEventPayload {
  runId: string;
  message: string;
  timestamp: string;
  nodeId?: string;
}

export interface PipelineLiveEventMap {
  "pipeline:started": PipelineStartedEventPayload;
  "pipeline:stage": PipelineStageEventPayload;
  "pipeline:log": PipelineLogEventPayload;
  "pipeline:artifact": PipelineArtifactEventPayload;
  "pipeline:question": PipelineQuestionEventPayload;
  "pipeline:completed": PipelineCompletedEventPayload;
  "pipeline:error": PipelineErrorEventPayload;
}

export type RunStatus = "running" | "completed" | "failed" | "cancelled" | "paused";

export interface StageEndStatus {
  nodeId: string;
  nodeLabel: string;
  status: string;
  output?: string;
  durationMs?: number;
}

export type RunEvent =
  | { type: "run_started"; runId: string; timestamp: string }
  | { type: "stage_started"; nodeId: string; nodeLabel: string; timestamp: string }
  | { type: "stage_log"; nodeId: string; text: string; timestamp: string }
  | { type: "stage_ended"; nodeId: string; nodeLabel: string; status: StageEndStatus; timestamp: string }
  | { type: "artifact"; nodeId: string; name: string; content: string; artifactType: string; timestamp: string }
  | { type: "session_ref"; sessionGroup: string; providerSessionRef: string; timestamp: string }
  | { type: "question"; questionId: string; questionText: string; timestamp: string }
  | { type: "answer"; questionId: string; answerText: string; timestamp: string }
  | { type: "iteration_completed"; iteration: number; verdict: string; timestamp: string }
  | { type: "run_ended"; status: RunStatus; reason?: string; timestamp: string };
