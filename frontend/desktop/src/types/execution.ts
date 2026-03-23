export type PipelineExecutionMode = "graph" | "direct_task";

export type PipelineExecutionStatus =
  | "idle"
  | "running"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

export type PipelineStageExecutionStatus =
  | "pending"
  | "running"
  | "completed"
  | "failed"
  | "cancelled"
  | "skipped";

export interface PipelineRunState {
  runId: string;
  mode: PipelineExecutionMode;
  templateId?: string;
  status: PipelineExecutionStatus;
  startedAt: string;
  updatedAt: string;
}

export interface PipelineStageState {
  runId: string;
  nodeId: string;
  nodeLabel: string;
  status: PipelineStageExecutionStatus;
  detail?: string;
  updatedAt: string;
}

export interface PipelineStageEventRecord {
  runId: string;
  nodeId: string;
  nodeLabel: string;
  status: PipelineStageExecutionStatus;
  detail?: string;
  timestamp: string;
}

export interface PipelineLogEntry {
  runId: string;
  nodeId: string;
  text: string;
  timestamp: string;
}

export interface PipelineArtifactEntry {
  runId: string;
  nodeId: string;
  name: string;
  artifactType: string;
  content: string;
  timestamp: string;
}

export interface PipelineQuestionState {
  runId: string;
  questionId: string;
  nodeId: string;
  questionText: string;
  timestamp: string;
}

export interface PipelineExecutionState {
  run: PipelineRunState | null;
  stages: PipelineStageState[];
  stageEvents: PipelineStageEventRecord[];
  logs: PipelineLogEntry[];
  artifacts: PipelineArtifactEntry[];
  sessionGroups: Record<string, string>;
  pendingQuestion: PipelineQuestionState | null;
  error: string | null;
}
