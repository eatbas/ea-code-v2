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

export const PIPELINE_RUN_COMMANDS = {
  START: "start_pipeline_run",
  PAUSE: "pause_pipeline_run",
  RESUME: "resume_pipeline_run",
  CANCEL: "cancel_pipeline_run",
  ANSWER: "answer_pipeline_question",
} as const;

export type PipelineRunCommand =
  (typeof PIPELINE_RUN_COMMANDS)[keyof typeof PIPELINE_RUN_COMMANDS];

export interface StartPipelineRunPayload {
  prompt: string;
  workspacePath: string;
  templateId?: string;
  template?: import("./templates").PipelineTemplate;
  directTask?: boolean;
  provider?: string;
  model?: string;
  executionIntent?: "text" | "code";
  providerOptions?: Record<string, unknown>;
  extraVars?: Record<string, string>;
}

export interface PausePipelineRunPayload {
  runId?: string;
}

export interface ResumePipelineRunPayload {
  runId?: string;
}

export interface CancelPipelineRunPayload {
  runId?: string;
  reason?: string;
}

export interface AnswerPipelineQuestionPayload {
  questionId: string;
  answerText: string;
}

export interface PipelineRunStateSnapshot {
  runId: string;
  status: PipelineStatus;
  updatedAt: string;
}

export interface StartPipelineRunResult extends PipelineRunStateSnapshot {
  startedAt: string;
}

export interface PipelineRunControlResult extends PipelineRunStateSnapshot {
  reason?: string;
}
