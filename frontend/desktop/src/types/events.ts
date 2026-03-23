export const PIPELINE_EVENTS = {
  STARTED: "pipeline:started",
  STAGE: "pipeline:stage",
  LOG: "pipeline:log",
  ARTIFACT: "pipeline:artifact",
  QUESTION: "pipeline:question",
  COMPLETED: "pipeline:completed",
  ERROR: "pipeline:error",
} as const;

export type RunStatus = "running" | "completed" | "failed" | "cancelled" | "paused";

export interface StageEndStatus {
  stageId: string;
  label: string;
  status: string;
  output?: string;
  durationMs?: number;
}

export type RunEvent =
  | { type: "run_started"; runId: string; timestamp: string }
  | { type: "stage_started"; stageId: string; label: string; timestamp: string }
  | { type: "stage_log"; stageId: string; text: string; timestamp: string }
  | { type: "stage_ended"; stageId: string; label: string; status: StageEndStatus; timestamp: string }
  | { type: "artifact"; stageId: string; name: string; content: string; artifactType: string; timestamp: string }
  | { type: "session_ref"; sessionGroup: string; providerSessionRef: string; timestamp: string }
  | { type: "question"; questionId: string; questionText: string; timestamp: string }
  | { type: "answer"; questionId: string; answerText: string; timestamp: string }
  | { type: "iteration_completed"; iteration: number; verdict: string; timestamp: string }
  | { type: "run_ended"; status: RunStatus; reason?: string; timestamp: string };
