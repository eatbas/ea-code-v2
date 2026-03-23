import type {
  PipelineArtifactEventPayload,
  PipelineCompletedEventPayload,
  PipelineErrorEventPayload,
  PipelineExecutionState,
  PipelineExecutionStatus,
  PipelineLogEventPayload,
  PipelineQuestionEventPayload,
  PipelineRunState,
  PipelineStageEventPayload,
  PipelineStartedEventPayload,
  PipelineTerminalStatus,
} from "../types";

export type PipelineExecutionAction =
  | { type: "RUN_STARTED"; payload: PipelineStartedEventPayload }
  | { type: "STAGE_EVENT"; payload: PipelineStageEventPayload }
  | { type: "LOG_EVENT"; payload: PipelineLogEventPayload }
  | { type: "ARTIFACT_EVENT"; payload: PipelineArtifactEventPayload }
  | { type: "QUESTION_EVENT"; payload: PipelineQuestionEventPayload }
  | { type: "RUN_COMPLETED"; payload: PipelineCompletedEventPayload }
  | { type: "RUN_ERROR"; payload: PipelineErrorEventPayload }
  | { type: "SET_SESSION_REF"; sessionGroup: string; providerSessionRef: string }
  | { type: "ANSWER_QUESTION"; questionId: string }
  | { type: "CONTROL_STATUS"; status: Extract<PipelineExecutionStatus, "running" | "paused" | "cancelled">; runId?: string }
  | { type: "RESET" };

export const initialPipelineExecutionState: PipelineExecutionState = {
  run: null,
  stages: [],
  stageEvents: [],
  logs: [],
  artifacts: [],
  sessionGroups: {},
  pendingQuestion: null,
  error: null,
};

function mapTerminalStatus(status: PipelineTerminalStatus): PipelineExecutionStatus {
  if (status === "completed") return "completed";
  if (status === "cancelled") return "cancelled";
  return "failed";
}

function shouldIgnoreForRun(state: PipelineExecutionState, runId: string): boolean {
  if (!state.run) {
    return false;
  }
  return state.run.runId !== runId;
}

function upsertStage(
  state: PipelineExecutionState,
  payload: PipelineStageEventPayload,
): PipelineExecutionState["stages"] {
  const index = state.stages.findIndex((stage) => stage.nodeId === payload.nodeId);
  const nextStage = {
    runId: payload.runId,
    nodeId: payload.nodeId,
    nodeLabel: payload.nodeLabel,
    status: payload.status,
    detail: payload.detail,
    updatedAt: payload.timestamp,
  };

  if (index < 0) {
    return [...state.stages, nextStage];
  }

  return state.stages.map((stage, position) => {
    if (position !== index) return stage;
    return { ...stage, ...nextStage };
  });
}

function updateRun(
  state: PipelineExecutionState,
  patch: Partial<PipelineRunState>,
): PipelineRunState | null {
  if (!state.run) {
    return null;
  }
  return {
    ...state.run,
    ...patch,
  };
}

export function pipelineExecutionReducer(
  state: PipelineExecutionState,
  action: PipelineExecutionAction,
): PipelineExecutionState {
  switch (action.type) {
    case "RUN_STARTED": {
      return {
        ...initialPipelineExecutionState,
        run: {
          runId: action.payload.runId,
          mode: action.payload.mode,
          templateId: action.payload.templateId,
          status: "running",
          startedAt: action.payload.timestamp,
          updatedAt: action.payload.timestamp,
        },
      };
    }
    case "STAGE_EVENT": {
      if (shouldIgnoreForRun(state, action.payload.runId)) {
        return state;
      }
      return {
        ...state,
        run: updateRun(state, { updatedAt: action.payload.timestamp }),
        stages: upsertStage(state, action.payload),
        stageEvents: [
          ...state.stageEvents,
          {
            runId: action.payload.runId,
            nodeId: action.payload.nodeId,
            nodeLabel: action.payload.nodeLabel,
            status: action.payload.status,
            detail: action.payload.detail,
            timestamp: action.payload.timestamp,
          },
        ],
      };
    }
    case "LOG_EVENT": {
      if (shouldIgnoreForRun(state, action.payload.runId)) {
        return state;
      }
      return {
        ...state,
        run: updateRun(state, { updatedAt: action.payload.timestamp }),
        logs: [...state.logs, action.payload],
      };
    }
    case "ARTIFACT_EVENT": {
      if (shouldIgnoreForRun(state, action.payload.runId)) {
        return state;
      }
      return {
        ...state,
        run: updateRun(state, { updatedAt: action.payload.timestamp }),
        artifacts: [...state.artifacts, action.payload],
      };
    }
    case "QUESTION_EVENT": {
      if (shouldIgnoreForRun(state, action.payload.runId)) {
        return state;
      }
      return {
        ...state,
        run: updateRun(state, { updatedAt: action.payload.timestamp }),
        pendingQuestion: action.payload,
      };
    }
    case "RUN_COMPLETED": {
      if (shouldIgnoreForRun(state, action.payload.runId)) {
        return state;
      }
      return {
        ...state,
        run: updateRun(state, {
          status: mapTerminalStatus(action.payload.status),
          updatedAt: action.payload.timestamp,
        }),
      };
    }
    case "RUN_ERROR": {
      if (shouldIgnoreForRun(state, action.payload.runId)) {
        return state;
      }
      return {
        ...state,
        error: action.payload.message,
        run: updateRun(state, {
          status: "failed",
          updatedAt: action.payload.timestamp,
        }),
      };
    }
    case "SET_SESSION_REF": {
      return {
        ...state,
        sessionGroups: {
          ...state.sessionGroups,
          [action.sessionGroup]: action.providerSessionRef,
        },
      };
    }
    case "ANSWER_QUESTION": {
      if (!state.pendingQuestion) {
        return state;
      }
      if (state.pendingQuestion.questionId !== action.questionId) {
        return state;
      }
      return {
        ...state,
        pendingQuestion: null,
      };
    }
    case "CONTROL_STATUS": {
      if (!state.run) {
        return state;
      }
      if (action.runId && state.run.runId !== action.runId) {
        return state;
      }
      return {
        ...state,
        run: {
          ...state.run,
          status: action.status,
          updatedAt: new Date().toISOString(),
        },
      };
    }
    case "RESET":
      return initialPipelineExecutionState;
    default:
      return state;
  }
}
