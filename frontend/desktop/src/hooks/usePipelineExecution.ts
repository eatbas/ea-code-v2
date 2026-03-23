import { useCallback, useEffect, useMemo, useReducer, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  PipelineArtifactEventPayload,
  PipelineCompletedEventPayload,
  PipelineErrorEventPayload,
  PipelineLogEventPayload,
  PipelineQuestionEventPayload,
  PipelineStageEventPayload,
  PipelineStartedEventPayload,
  StartPipelineRunPayload,
} from "../types";
import { PIPELINE_EVENTS } from "../types";
import { usePipelineRun } from "./usePipelineRun";
import {
  initialPipelineExecutionState,
  pipelineExecutionReducer,
} from "./pipelineExecutionReducer";

export function usePipelineExecution() {
  const [state, dispatch] = useReducer(
    pipelineExecutionReducer,
    initialPipelineExecutionState,
  );
  const [listenerError, setListenerError] = useState<string | null>(null);
  const {
    loading,
    error: commandError,
    startPipelineRun,
    pausePipelineRun,
    resumePipelineRun,
    cancelPipelineRun,
    answerPipelineQuestion,
  } = usePipelineRun();

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];
    let active = true;

    const register = async () => {
      try {
        const started = await listen<PipelineStartedEventPayload>(
          PIPELINE_EVENTS.STARTED,
          (event) => {
            dispatch({ type: "RUN_STARTED", payload: event.payload });
          },
        );
        if (active) unlisteners.push(started);

        const stage = await listen<PipelineStageEventPayload>(
          PIPELINE_EVENTS.STAGE,
          (event) => {
            dispatch({ type: "STAGE_EVENT", payload: event.payload });
          },
        );
        if (active) unlisteners.push(stage);

        const log = await listen<PipelineLogEventPayload>(
          PIPELINE_EVENTS.LOG,
          (event) => {
            dispatch({ type: "LOG_EVENT", payload: event.payload });
          },
        );
        if (active) unlisteners.push(log);

        const artifact = await listen<PipelineArtifactEventPayload>(
          PIPELINE_EVENTS.ARTIFACT,
          (event) => {
            dispatch({ type: "ARTIFACT_EVENT", payload: event.payload });
          },
        );
        if (active) unlisteners.push(artifact);

        const question = await listen<PipelineQuestionEventPayload>(
          PIPELINE_EVENTS.QUESTION,
          (event) => {
            dispatch({ type: "QUESTION_EVENT", payload: event.payload });
          },
        );
        if (active) unlisteners.push(question);

        const completed = await listen<PipelineCompletedEventPayload>(
          PIPELINE_EVENTS.COMPLETED,
          (event) => {
            dispatch({ type: "RUN_COMPLETED", payload: event.payload });
          },
        );
        if (active) unlisteners.push(completed);

        const failed = await listen<PipelineErrorEventPayload>(
          PIPELINE_EVENTS.ERROR,
          (event) => {
            dispatch({ type: "RUN_ERROR", payload: event.payload });
          },
        );
        if (active) unlisteners.push(failed);
      } catch (error) {
        setListenerError(String(error));
      }
    };

    void register();

    return () => {
      active = false;
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, []);

  const startPipeline = useCallback(
    async (payload: StartPipelineRunPayload): Promise<boolean> => {
      const result = await startPipelineRun(payload);
      if (!result) return false;

      const mode = payload.directTask ? "direct_task" : "graph";
      dispatch({
        type: "RUN_STARTED",
        payload: {
          runId: result.runId,
          mode,
          templateId: payload.templateId,
          timestamp: result.startedAt,
        },
      });
      return true;
    },
    [startPipelineRun],
  );

  const pausePipeline = useCallback(async (): Promise<boolean> => {
    const result = await pausePipelineRun();
    if (!result) return false;
    dispatch({
      type: "CONTROL_STATUS",
      status: "paused",
      runId: result.runId,
    });
    return true;
  }, [pausePipelineRun]);

  const resumePipeline = useCallback(async (): Promise<boolean> => {
    const result = await resumePipelineRun();
    if (!result) return false;
    dispatch({
      type: "CONTROL_STATUS",
      status: "running",
      runId: result.runId,
    });
    return true;
  }, [resumePipelineRun]);

  const cancelPipeline = useCallback(async (): Promise<boolean> => {
    const result = await cancelPipelineRun();
    if (!result) return false;
    dispatch({
      type: "CONTROL_STATUS",
      status: "cancelled",
      runId: result.runId,
    });
    return true;
  }, [cancelPipelineRun]);

  const answerQuestion = useCallback(
    async (questionId: string, answer: string): Promise<boolean> => {
      const ok = await answerPipelineQuestion({
        questionId,
        answerText: answer,
      });
      if (ok) {
        dispatch({ type: "ANSWER_QUESTION", questionId });
      }
      return ok;
    },
    [answerPipelineQuestion],
  );

  const currentStage = useMemo(() => {
    const running = state.stages.find((stage) => stage.status === "running");
    if (running) return running;
    if (state.stages.length === 0) return null;
    return state.stages[state.stages.length - 1] ?? null;
  }, [state.stages]);

  const isRunning = state.run?.status === "running" || state.run?.status === "paused";
  const error = listenerError ?? commandError ?? state.error;

  const setSessionRef = useCallback((sessionGroup: string, providerSessionRef: string) => {
    dispatch({
      type: "SET_SESSION_REF",
      sessionGroup,
      providerSessionRef,
    });
  }, []);

  const resetExecution = useCallback(() => {
    dispatch({ type: "RESET" });
  }, []);

  return {
    run: state.run,
    isRunning,
    currentStage,
    stages: state.stages,
    stageEvents: state.stageEvents,
    logs: state.logs,
    artifacts: state.artifacts,
    pendingQuestion: state.pendingQuestion,
    sessionGroups: state.sessionGroups,
    loading,
    error,
    startPipeline,
    pausePipeline,
    resumePipeline,
    cancelPipeline,
    answerQuestion,
    setSessionRef,
    resetExecution,
  };
}
