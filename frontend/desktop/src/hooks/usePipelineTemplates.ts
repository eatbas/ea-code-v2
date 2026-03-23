import { useState, useCallback } from "react";
import type {
  PipelineTemplate,
  CreateTemplateRequest,
  UpdateTemplateRequest,
  CloneTemplateRequest,
} from "../types";
import { invoke } from "../lib/invoke";

export function usePipelineTemplates() {
  const [templates, setTemplates] = useState<PipelineTemplate[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const builtinTemplates = templates.filter((t) => t.isBuiltin);
  const userTemplates = templates.filter((t) => !t.isBuiltin);

  const refreshTemplates = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await invoke<PipelineTemplate[]>("list_templates");
      setTemplates(list);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const getTemplate = useCallback(
    async (id: string): Promise<PipelineTemplate | null> => {
      setLoading(true);
      setError(null);
      try {
        return await invoke<PipelineTemplate>("get_template", { id });
      } catch (e) {
        setError(String(e));
        return null;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const createTemplate = useCallback(
    async (
      payload: CreateTemplateRequest,
    ): Promise<PipelineTemplate | null> => {
      setLoading(true);
      setError(null);
      try {
        const created = await invoke<PipelineTemplate>("create_template", {
          payload,
        });
        const list = await invoke<PipelineTemplate[]>("list_templates");
        setTemplates(list);
        return created;
      } catch (e) {
        setError(String(e));
        return null;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const updateTemplate = useCallback(
    async (
      id: string,
      payload: UpdateTemplateRequest,
    ): Promise<PipelineTemplate | null> => {
      setLoading(true);
      setError(null);
      try {
        const updated = await invoke<PipelineTemplate>("update_template", {
          id,
          payload,
        });
        const list = await invoke<PipelineTemplate[]>("list_templates");
        setTemplates(list);
        return updated;
      } catch (e) {
        setError(String(e));
        return null;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const deleteTemplate = useCallback(async (id: string): Promise<boolean> => {
    setLoading(true);
    setError(null);
    try {
      await invoke<void>("delete_template", { id });
      const list = await invoke<PipelineTemplate[]>("list_templates");
      setTemplates(list);
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  const cloneTemplate = useCallback(
    async (
      id: string,
      payload: CloneTemplateRequest,
    ): Promise<PipelineTemplate | null> => {
      setLoading(true);
      setError(null);
      try {
        const cloned = await invoke<PipelineTemplate>("clone_template", {
          id,
          payload,
        });
        const list = await invoke<PipelineTemplate[]>("list_templates");
        setTemplates(list);
        return cloned;
      } catch (e) {
        setError(String(e));
        return null;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const enhancePrompt = useCallback(
    async (
      draft: string,
      provider: string,
      model: string,
    ): Promise<string | null> => {
      setLoading(true);
      setError(null);
      try {
        return await invoke<string>("enhance_prompt", {
          draft,
          provider,
          model,
        });
      } catch (e) {
        setError(String(e));
        return null;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  return {
    templates,
    builtinTemplates,
    userTemplates,
    loading,
    error,
    refreshTemplates,
    getTemplate,
    createTemplate,
    updateTemplate,
    deleteTemplate,
    cloneTemplate,
    enhancePrompt,
  };
}
