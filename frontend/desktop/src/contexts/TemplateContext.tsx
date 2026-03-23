import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useReducer,
  type Dispatch,
  type PropsWithChildren,
  type ReactNode,
} from "react";
import { usePipelineTemplates } from "../hooks/usePipelineTemplates";
import { useAppContext } from "./AppContext";
import type {
  CloneTemplateRequest,
  CreateTemplateRequest,
  PipelineTemplate,
  UpdateTemplateRequest,
} from "../types";

interface TemplateState {
  activeTemplateId: string | null;
}

export type TemplateAction =
  | { type: "SET_ACTIVE"; templateId: string | null }
  | { type: "RESET" };

interface TemplateContextType {
  templates: PipelineTemplate[];
  builtinTemplates: PipelineTemplate[];
  userTemplates: PipelineTemplate[];
  activeTemplate: PipelineTemplate | null;
  loading: boolean;
  error: string | null;
  dispatch: Dispatch<TemplateAction>;
  refreshTemplates: () => Promise<void>;
  createTemplate: (payload: CreateTemplateRequest) => Promise<PipelineTemplate | null>;
  updateTemplate: (
    id: string,
    payload: UpdateTemplateRequest,
  ) => Promise<PipelineTemplate | null>;
  deleteTemplate: (id: string) => Promise<boolean>;
  cloneTemplate: (
    id: string,
    payload: CloneTemplateRequest,
  ) => Promise<PipelineTemplate | null>;
  enhancePrompt: (
    draft: string,
    provider: string,
    model: string,
  ) => Promise<string | null>;
}

const TemplateContext = createContext<TemplateContextType | null>(null);

const initialTemplateState: TemplateState = {
  activeTemplateId: null,
};

function templateReducer(
  state: TemplateState,
  action: TemplateAction,
): TemplateState {
  switch (action.type) {
    case "SET_ACTIVE":
      return {
        activeTemplateId: action.templateId,
      };
    case "RESET":
      return initialTemplateState;
    default:
      return state;
  }
}

export function TemplateProvider({ children }: PropsWithChildren): ReactNode {
  const { settings } = useAppContext();
  const {
    templates,
    builtinTemplates,
    userTemplates,
    loading,
    error,
    refreshTemplates,
    createTemplate,
    updateTemplate,
    deleteTemplate,
    cloneTemplate,
    enhancePrompt,
  } = usePipelineTemplates();
  const [state, dispatch] = useReducer(templateReducer, initialTemplateState);

  useEffect(() => {
    void refreshTemplates();
  }, [refreshTemplates]);

  useEffect(() => {
    if (templates.length === 0) return;

    const activeStillExists =
      state.activeTemplateId &&
      templates.some((template) => template.id === state.activeTemplateId);

    if (activeStillExists) return;

    const preferred = templates.find(
      (template) => template.id === settings.defaultPipelineId,
    );
    const fallback = templates[0];
    dispatch({
      type: "SET_ACTIVE",
      templateId: preferred?.id ?? fallback?.id ?? null,
    });
  }, [settings.defaultPipelineId, templates, state.activeTemplateId]);

  const activeTemplate = useMemo(() => {
    if (!state.activeTemplateId) return null;
    return (
      templates.find((template) => template.id === state.activeTemplateId) ?? null
    );
  }, [state.activeTemplateId, templates]);

  const refreshAndSetActive = useCallback(async () => {
    await refreshTemplates();
  }, [refreshTemplates]);

  const value = useMemo<TemplateContextType>(
    () => ({
      templates,
      builtinTemplates,
      userTemplates,
      activeTemplate,
      loading,
      error,
      dispatch,
      refreshTemplates: refreshAndSetActive,
      createTemplate,
      updateTemplate,
      deleteTemplate,
      cloneTemplate,
      enhancePrompt,
    }),
    [
      templates,
      builtinTemplates,
      userTemplates,
      activeTemplate,
      loading,
      error,
      refreshAndSetActive,
      createTemplate,
      updateTemplate,
      deleteTemplate,
      cloneTemplate,
      enhancePrompt,
    ],
  );

  return (
    <TemplateContext.Provider value={value}>{children}</TemplateContext.Provider>
  );
}

export function useTemplateContext(): TemplateContextType {
  const context = useContext(TemplateContext);
  if (!context) {
    throw new Error("useTemplateContext must be used within a TemplateProvider");
  }
  return context;
}
