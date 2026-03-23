import { describe, expect, it } from "vitest";
import type {
  PipelineArtifactEventPayload,
  PipelineCompletedEventPayload,
  PipelineErrorEventPayload,
  PipelineLogEventPayload,
  PipelineQuestionEventPayload,
  PipelineStageEventPayload,
  PipelineStartedEventPayload,
} from "../types";
import {
  initialPipelineExecutionState,
  pipelineExecutionReducer,
} from "./pipelineExecutionReducer";

function runStartedPayload(overrides: Partial<PipelineStartedEventPayload> = {}): PipelineStartedEventPayload {
  return {
    runId: "run-1",
    mode: "graph",
    templateId: "full-review-loop",
    timestamp: "2026-03-23T12:00:00Z",
    ...overrides,
  };
}

function stagePayload(overrides: Partial<PipelineStageEventPayload> = {}): PipelineStageEventPayload {
  return {
    runId: "run-1",
    nodeId: "node-1",
    nodeLabel: "Analyse",
    status: "running",
    timestamp: "2026-03-23T12:00:01Z",
    ...overrides,
  };
}

function logPayload(overrides: Partial<PipelineLogEventPayload> = {}): PipelineLogEventPayload {
  return {
    runId: "run-1",
    nodeId: "node-1",
    text: "log line",
    timestamp: "2026-03-23T12:00:02Z",
    ...overrides,
  };
}

function artifactPayload(
  overrides: Partial<PipelineArtifactEventPayload> = {},
): PipelineArtifactEventPayload {
  return {
    runId: "run-1",
    nodeId: "node-2",
    name: "git-diff.patch",
    artifactType: "git_diff",
    content: "diff --git a b",
    timestamp: "2026-03-23T12:00:03Z",
    ...overrides,
  };
}

function questionPayload(
  overrides: Partial<PipelineQuestionEventPayload> = {},
): PipelineQuestionEventPayload {
  return {
    runId: "run-1",
    questionId: "q-1",
    nodeId: "node-3",
    questionText: "Apply this change?",
    timestamp: "2026-03-23T12:00:04Z",
    ...overrides,
  };
}

function completedPayload(
  overrides: Partial<PipelineCompletedEventPayload> = {},
): PipelineCompletedEventPayload {
  return {
    runId: "run-1",
    status: "completed",
    timestamp: "2026-03-23T12:00:05Z",
    ...overrides,
  };
}

function errorPayload(overrides: Partial<PipelineErrorEventPayload> = {}): PipelineErrorEventPayload {
  return {
    runId: "run-1",
    message: "Stage failed",
    timestamp: "2026-03-23T12:00:06Z",
    nodeId: "node-2",
    ...overrides,
  };
}

describe("pipelineExecutionReducer", () => {
  it("starts a run and clears prior transient state", () => {
    const dirtyState = {
      ...initialPipelineExecutionState,
      logs: [logPayload()],
      error: "old",
    };

    const next = pipelineExecutionReducer(dirtyState, {
      type: "RUN_STARTED",
      payload: runStartedPayload(),
    });

    expect(next.run).toEqual({
      runId: "run-1",
      mode: "graph",
      templateId: "full-review-loop",
      status: "running",
      startedAt: "2026-03-23T12:00:00Z",
      updatedAt: "2026-03-23T12:00:00Z",
    });
    expect(next.logs).toEqual([]);
    expect(next.error).toBeNull();
  });

  it("tracks stage, log, artifact, and question payloads", () => {
    const started = pipelineExecutionReducer(initialPipelineExecutionState, {
      type: "RUN_STARTED",
      payload: runStartedPayload(),
    });
    const withStage = pipelineExecutionReducer(started, {
      type: "STAGE_EVENT",
      payload: stagePayload(),
    });
    const withLog = pipelineExecutionReducer(withStage, {
      type: "LOG_EVENT",
      payload: logPayload(),
    });
    const withArtifact = pipelineExecutionReducer(withLog, {
      type: "ARTIFACT_EVENT",
      payload: artifactPayload(),
    });
    const withQuestion = pipelineExecutionReducer(withArtifact, {
      type: "QUESTION_EVENT",
      payload: questionPayload(),
    });

    expect(withQuestion.stages).toHaveLength(1);
    expect(withQuestion.stages[0]?.status).toBe("running");
    expect(withQuestion.logs).toEqual([logPayload()]);
    expect(withQuestion.artifacts).toEqual([artifactPayload()]);
    expect(withQuestion.pendingQuestion?.questionId).toBe("q-1");
  });

  it("ignores stale events from a different run", () => {
    const started = pipelineExecutionReducer(initialPipelineExecutionState, {
      type: "RUN_STARTED",
      payload: runStartedPayload(),
    });

    const next = pipelineExecutionReducer(started, {
      type: "LOG_EVENT",
      payload: logPayload({ runId: "run-stale" }),
    });

    expect(next).toBe(started);
  });

  it("handles completion, explicit control status, and errors", () => {
    const started = pipelineExecutionReducer(initialPipelineExecutionState, {
      type: "RUN_STARTED",
      payload: runStartedPayload(),
    });

    const paused = pipelineExecutionReducer(started, {
      type: "CONTROL_STATUS",
      status: "paused",
      runId: "run-1",
    });
    expect(paused.run?.status).toBe("paused");

    const completed = pipelineExecutionReducer(paused, {
      type: "RUN_COMPLETED",
      payload: completedPayload(),
    });
    expect(completed.run?.status).toBe("completed");

    const failed = pipelineExecutionReducer(completed, {
      type: "RUN_ERROR",
      payload: errorPayload(),
    });
    expect(failed.run?.status).toBe("failed");
    expect(failed.error).toBe("Stage failed");
  });

  it("clears pending question only for matching answer id", () => {
    const started = pipelineExecutionReducer(initialPipelineExecutionState, {
      type: "RUN_STARTED",
      payload: runStartedPayload(),
    });
    const withQuestion = pipelineExecutionReducer(started, {
      type: "QUESTION_EVENT",
      payload: questionPayload(),
    });
    const unchanged = pipelineExecutionReducer(withQuestion, {
      type: "ANSWER_QUESTION",
      questionId: "q-other",
    });
    const cleared = pipelineExecutionReducer(unchanged, {
      type: "ANSWER_QUESTION",
      questionId: "q-1",
    });

    expect(unchanged.pendingQuestion?.questionId).toBe("q-1");
    expect(cleared.pendingQuestion).toBeNull();
  });

  it("stores session references and supports full reset", () => {
    const started = pipelineExecutionReducer(initialPipelineExecutionState, {
      type: "RUN_STARTED",
      payload: runStartedPayload(),
    });
    const withSessionRef = pipelineExecutionReducer(started, {
      type: "SET_SESSION_REF",
      sessionGroup: "A",
      providerSessionRef: "sess-123",
    });
    expect(withSessionRef.sessionGroups.A).toBe("sess-123");

    const reset = pipelineExecutionReducer(withSessionRef, { type: "RESET" });
    expect(reset).toEqual(initialPipelineExecutionState);
  });
});
