import type { ReactNode } from "react";
import { useMemo, useState } from "react";
import { useAppContext } from "../contexts/AppContext";
import { usePipelineContext } from "../contexts/PipelineContext";
import { useTemplateContext } from "../contexts/TemplateContext";

export function IdleView(): ReactNode {
  const { workspace, dispatch: appDispatch } = useAppContext();
  const { activeTemplate, templates, dispatch: templateDispatch } = useTemplateContext();
  const { startPipeline, loading } = usePipelineContext();

  const [prompt, setPrompt] = useState("Implement the requested change safely.");
  const [workspacePathInput, setWorkspacePathInput] = useState(
    workspace?.path ?? ".",
  );
  const [directTask, setDirectTask] = useState(false);
  const [noPlan, setNoPlan] = useState(false);

  const selectedTemplateId = activeTemplate?.id ?? "";
  const quickTemplates = [
    { id: "full-review-loop", label: "Full Review Loop" },
    { id: "quick-fix", label: "Quick Fix" },
    { id: "research-only", label: "Research Only" },
  ];

  const templateDescription = useMemo(() => {
    if (!activeTemplate) return "No template selected.";
    return activeTemplate.description;
  }, [activeTemplate]);

  const run = async () => {
    const workspacePath = workspacePathInput.trim() || ".";

    await startPipeline({
      prompt,
      workspacePath,
      templateId: directTask ? undefined : activeTemplate?.id,
      directTask,
      provider: directTask ? "claude" : undefined,
      model: directTask ? "sonnet" : undefined,
      executionIntent: "code",
      extraVars: noPlan ? { no_plan: "true" } : undefined,
    });

    appDispatch({ type: "SET_VIEW", view: "chat" });
  };

  return (
    <div className="flex h-full flex-col gap-4 bg-white p-6">
      <div className="space-y-1">
        <h1 className="text-2xl font-semibold text-slate-900">EA Code v2</h1>
        <p className="text-sm text-slate-600">
          Select a pipeline template and start a run.
        </p>
      </div>

      <label className="flex flex-col gap-1 text-sm text-slate-800">
        Workspace Path
        <input
          className="rounded border border-slate-300 px-3 py-2"
          onChange={(event) => {
            const nextPath = event.target.value;
            setWorkspacePathInput(nextPath);
            appDispatch({
              type: "SET_WORKSPACE",
              workspace: {
                path: nextPath,
                name:
                  nextPath.split("/").filter(Boolean).slice(-1)[0] ?? nextPath,
                isGitRepo: false,
              },
            });
          }}
          value={workspacePathInput}
        />
      </label>

      <label className="flex flex-col gap-1 text-sm text-slate-800">
        Pipeline Template
        <select
          className="rounded border border-slate-300 px-3 py-2"
          onChange={(event) =>
            templateDispatch({ type: "SET_ACTIVE", templateId: event.target.value })
          }
          value={selectedTemplateId}
        >
          {templates.map((template) => (
            <option key={template.id} value={template.id}>
              {template.name}
            </option>
          ))}
        </select>
      </label>

      <div className="flex flex-wrap gap-2">
        {quickTemplates.map((template) => (
          <button
            className={`rounded border px-2 py-1 text-xs ${
              selectedTemplateId === template.id
                ? "border-slate-900 bg-slate-900 text-white"
                : "border-slate-300"
            }`}
            key={template.id}
            onClick={() =>
              templateDispatch({
                type: "SET_ACTIVE",
                templateId: template.id,
              })
            }
            type="button"
          >
            {template.label}
          </button>
        ))}
      </div>

      <p className="rounded bg-slate-100 p-3 text-xs text-slate-700">
        {templateDescription}
      </p>

      <label className="flex flex-col gap-1 text-sm text-slate-800">
        Prompt
        <textarea
          className="min-h-36 rounded border border-slate-300 px-3 py-2"
          onChange={(event) => setPrompt(event.target.value)}
          value={prompt}
        />
      </label>

      <div className="flex flex-wrap items-center gap-4 text-sm text-slate-700">
        <label className="flex items-center gap-2">
          <input
            checked={directTask}
            onChange={(event) => setDirectTask(event.target.checked)}
            type="checkbox"
          />
          Direct Task
        </label>
        <label className="flex items-center gap-2">
          <input
            checked={noPlan}
            onChange={(event) => setNoPlan(event.target.checked)}
            type="checkbox"
          />
          No Plan
        </label>
      </div>

      <div className="flex items-center gap-3">
        <button
          className="rounded bg-slate-900 px-4 py-2 text-sm font-medium text-white disabled:opacity-60"
          disabled={loading || (!directTask && !activeTemplate)}
          onClick={() => {
            void run();
          }}
          type="button"
        >
          {loading ? "Starting..." : "Start Run"}
        </button>

        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          onClick={() => appDispatch({ type: "SET_VIEW", view: "pipeline-builder" })}
          type="button"
        >
          Edit Pipeline
        </button>

        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          onClick={() => appDispatch({ type: "SET_VIEW", view: "pipeline-gallery" })}
          type="button"
        >
          Browse All
        </button>
      </div>
    </div>
  );
}
