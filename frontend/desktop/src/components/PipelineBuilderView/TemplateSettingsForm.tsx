import type { ReactNode } from "react";

type SessionMode = "simple" | "advanced";

interface TemplateSettingsFormProps {
  name: string;
  description: string;
  maxIterations: number;
  stopOnFirstPass: boolean;
  sessionMode: SessionMode;
  onNameChange: (value: string) => void;
  onDescriptionChange: (value: string) => void;
  onMaxIterationsChange: (value: number) => void;
  onStopOnFirstPassChange: (value: boolean) => void;
  onSessionModeChange: (mode: SessionMode) => void;
}

export function TemplateSettingsForm({
  name,
  description,
  maxIterations,
  stopOnFirstPass,
  sessionMode,
  onNameChange,
  onDescriptionChange,
  onMaxIterationsChange,
  onStopOnFirstPassChange,
  onSessionModeChange,
}: TemplateSettingsFormProps): ReactNode {
  return (
    <>
      <div className="grid gap-2 md:grid-cols-2">
        <label className="flex flex-col gap-1 text-xs text-slate-700">
          Name
          <input
            className="rounded border border-slate-300 px-2 py-1"
            onChange={(event) => onNameChange(event.target.value)}
            value={name}
          />
        </label>

        <label className="flex flex-col gap-1 text-xs text-slate-700">
          Description
          <input
            className="rounded border border-slate-300 px-2 py-1"
            onChange={(event) => onDescriptionChange(event.target.value)}
            value={description}
          />
        </label>

        <label className="flex flex-col gap-1 text-xs text-slate-700">
          Max Iterations
          <input
            className="rounded border border-slate-300 px-2 py-1"
            min={1}
            onChange={(event) => {
              const parsed = Number.parseInt(event.target.value, 10);
              onMaxIterationsChange(Number.isNaN(parsed) ? 1 : Math.max(1, parsed));
            }}
            type="number"
            value={maxIterations}
          />
        </label>

        <label className="mt-5 flex items-center gap-2 text-xs text-slate-700">
          <input
            checked={stopOnFirstPass}
            onChange={(event) => onStopOnFirstPassChange(event.target.checked)}
            type="checkbox"
          />
          Stop on first pass
        </label>
      </div>

      <div className="mt-3 flex flex-wrap items-center gap-2">
        <button
          className={`rounded border px-2 py-1 text-xs ${
            sessionMode === "simple"
              ? "border-slate-900 bg-slate-900 text-white"
              : "border-slate-300"
          }`}
          onClick={() => onSessionModeChange("simple")}
          type="button"
        >
          Simple session mode
        </button>
        <button
          className={`rounded border px-2 py-1 text-xs ${
            sessionMode === "advanced"
              ? "border-slate-900 bg-slate-900 text-white"
              : "border-slate-300"
          }`}
          onClick={() => onSessionModeChange("advanced")}
          type="button"
        >
          Advanced session mode
        </button>
      </div>
    </>
  );
}
