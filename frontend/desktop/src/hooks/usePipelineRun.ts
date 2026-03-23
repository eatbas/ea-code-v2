import { useCallback, useState } from "react";
import { invoke } from "../lib/invoke";
import {
  type AnswerPipelineQuestionPayload,
  PIPELINE_RUN_COMMANDS,
  type CancelPipelineRunPayload,
  type PausePipelineRunPayload,
  type PipelineRunCommand,
  type PipelineRunControlResult,
  type ResumePipelineRunPayload,
  type StartPipelineRunPayload,
  type StartPipelineRunResult,
} from "../types";

type PipelineRunResult = StartPipelineRunResult | PipelineRunControlResult;

export function usePipelineRun() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastResult, setLastResult] = useState<PipelineRunResult | null>(null);

  const runCommand = useCallback(
    async <TPayload, TResult extends PipelineRunResult>(
      command: PipelineRunCommand,
      payload: TPayload,
    ): Promise<TResult | null> => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<TResult>(command, { payload });
        setLastResult(result);
        return result;
      } catch (e) {
        setError(String(e));
        return null;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const startPipelineRun = useCallback(
    (payload: StartPipelineRunPayload): Promise<StartPipelineRunResult | null> =>
      runCommand<StartPipelineRunPayload, StartPipelineRunResult>(
        PIPELINE_RUN_COMMANDS.START,
        payload,
      ),
    [runCommand],
  );

  const pausePipelineRun = useCallback(
    (
      payload: PausePipelineRunPayload = {},
    ): Promise<PipelineRunControlResult | null> =>
      runCommand<PausePipelineRunPayload, PipelineRunControlResult>(
        PIPELINE_RUN_COMMANDS.PAUSE,
        payload,
      ),
    [runCommand],
  );

  const resumePipelineRun = useCallback(
    (
      payload: ResumePipelineRunPayload = {},
    ): Promise<PipelineRunControlResult | null> =>
      runCommand<ResumePipelineRunPayload, PipelineRunControlResult>(
        PIPELINE_RUN_COMMANDS.RESUME,
        payload,
      ),
    [runCommand],
  );

  const cancelPipelineRun = useCallback(
    (
      payload: CancelPipelineRunPayload = {},
    ): Promise<PipelineRunControlResult | null> =>
      runCommand<CancelPipelineRunPayload, PipelineRunControlResult>(
        PIPELINE_RUN_COMMANDS.CANCEL,
        payload,
      ),
    [runCommand],
  );

  const answerPipelineQuestion = useCallback(
    async (payload: AnswerPipelineQuestionPayload): Promise<boolean> => {
      setLoading(true);
      setError(null);
      try {
        await invoke<void>(PIPELINE_RUN_COMMANDS.ANSWER, { payload });
        return true;
      } catch (e) {
        setError(String(e));
        return false;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  return {
    loading,
    error,
    lastResult,
    startPipelineRun,
    pausePipelineRun,
    resumePipelineRun,
    cancelPipelineRun,
    answerPipelineQuestion,
  };
}
