import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  type PropsWithChildren,
  type ReactNode,
} from "react";
import { usePipelineExecution } from "../hooks/usePipelineExecution";
import type { StartPipelineRunPayload } from "../types";

export type PipelineAction =
  | { type: "START_RUN"; payload: StartPipelineRunPayload }
  | { type: "PAUSE_RUN" }
  | { type: "RESUME_RUN" }
  | { type: "CANCEL_RUN" }
  | { type: "ANSWER_QUESTION"; questionId: string; answer: string }
  | { type: "SET_SESSION_REF"; sessionGroup: string; providerSessionRef: string }
  | { type: "RESET" };

interface PipelineContextType {
  run: ReturnType<typeof usePipelineExecution>["run"];
  isRunning: boolean;
  currentStage: ReturnType<typeof usePipelineExecution>["currentStage"];
  stages: ReturnType<typeof usePipelineExecution>["stages"];
  stageEvents: ReturnType<typeof usePipelineExecution>["stageEvents"];
  logs: ReturnType<typeof usePipelineExecution>["logs"];
  artifacts: ReturnType<typeof usePipelineExecution>["artifacts"];
  pendingQuestion: ReturnType<typeof usePipelineExecution>["pendingQuestion"];
  sessionGroups: ReturnType<typeof usePipelineExecution>["sessionGroups"];
  loading: boolean;
  error: string | null;
  dispatch: (action: PipelineAction) => Promise<boolean>;
  startPipeline: (payload: StartPipelineRunPayload) => Promise<boolean>;
  pausePipeline: () => Promise<boolean>;
  resumePipeline: () => Promise<boolean>;
  cancelPipeline: () => Promise<boolean>;
  answerQuestion: (questionId: string, answer: string) => Promise<boolean>;
  resetExecution: () => void;
}

const PipelineContext = createContext<PipelineContextType | null>(null);

export function PipelineProvider({ children }: PropsWithChildren): ReactNode {
  const execution = usePipelineExecution();

  const dispatch = useCallback(
    async (action: PipelineAction): Promise<boolean> => {
      switch (action.type) {
        case "START_RUN":
          return execution.startPipeline(action.payload);
        case "PAUSE_RUN":
          return execution.pausePipeline();
        case "RESUME_RUN":
          return execution.resumePipeline();
        case "CANCEL_RUN":
          return execution.cancelPipeline();
        case "ANSWER_QUESTION":
          return execution.answerQuestion(action.questionId, action.answer);
        case "SET_SESSION_REF":
          execution.setSessionRef(action.sessionGroup, action.providerSessionRef);
          return true;
        case "RESET":
          execution.resetExecution();
          return true;
        default:
          return true;
      }
    },
    [execution],
  );

  const value = useMemo<PipelineContextType>(
    () => ({
      run: execution.run,
      isRunning: execution.isRunning,
      currentStage: execution.currentStage,
      stages: execution.stages,
      stageEvents: execution.stageEvents,
      logs: execution.logs,
      artifacts: execution.artifacts,
      pendingQuestion: execution.pendingQuestion,
      sessionGroups: execution.sessionGroups,
      loading: execution.loading,
      error: execution.error,
      dispatch,
      startPipeline: execution.startPipeline,
      pausePipeline: execution.pausePipeline,
      resumePipeline: execution.resumePipeline,
      cancelPipeline: execution.cancelPipeline,
      answerQuestion: execution.answerQuestion,
      resetExecution: execution.resetExecution,
    }),
    [execution, dispatch],
  );

  return (
    <PipelineContext.Provider value={value}>{children}</PipelineContext.Provider>
  );
}

export function usePipelineContext(): PipelineContextType {
  const context = useContext(PipelineContext);
  if (!context) {
    throw new Error("usePipelineContext must be used within a PipelineProvider");
  }
  return context;
}
