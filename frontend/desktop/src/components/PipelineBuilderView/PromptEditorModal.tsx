import { useMemo, useRef, useState, type ReactNode } from "react";
import type { ProviderInfo, StageExecutionIntent, StageNodeDefinition } from "../../types";
import { inferModels } from "../../utils/inferModels";
import { EXECUTION_INTENTS, SESSION_GROUP_OPTIONS, VARIABLE_CHIPS } from "./constants";

interface PromptEditorModalProps {
  stage: StageNodeDefinition;
  providers: ProviderInfo[];
  onCancel: () => void;
  onSave: (nextStage: StageNodeDefinition) => void;
  onEnhance: (
    draft: string,
    provider: string,
    model: string,
  ) => Promise<string | null>;
}

export function PromptEditorModal({
  stage,
  providers,
  onCancel,
  onSave,
  onEnhance,
}: PromptEditorModalProps): ReactNode {
  const [draft, setDraft] = useState(stage.promptTemplate);
  const [label, setLabel] = useState(stage.label);
  const [stageType, setStageType] = useState(stage.stageType);
  const [provider, setProvider] = useState(stage.provider);
  const [model, setModel] = useState(stage.model);
  const [sessionGroup, setSessionGroup] = useState(stage.sessionGroup);
  const [executionIntent, setExecutionIntent] = useState(stage.executionIntent);
  const [isEnhancing, setIsEnhancing] = useState(false);
  const [enhancedDraft, setEnhancedDraft] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const modelOptions = useMemo(
    () => inferModels({ provider, model }, providers),
    [model, provider, providers, stage],
  );

  const insertVariable = (variable: string) => {
    const element = textareaRef.current;
    if (!element) {
      setDraft((previous) => `${previous}${variable}`);
      return;
    }

    const start = element.selectionStart ?? draft.length;
    const end = element.selectionEnd ?? draft.length;
    const next = `${draft.slice(0, start)}${variable}${draft.slice(end)}`;
    setDraft(next);

    window.requestAnimationFrame(() => {
      element.focus();
      const cursor = start + variable.length;
      element.selectionStart = cursor;
      element.selectionEnd = cursor;
    });
  };

  const enhancePrompt = async () => {
    setIsEnhancing(true);
    const enhanced = await onEnhance(draft, provider, model);
    setIsEnhancing(false);
    if (enhanced) {
      setEnhancedDraft(enhanced);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-6">
      <div className="max-h-[92vh] w-full max-w-5xl overflow-auto rounded border border-slate-300 bg-white p-4">
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-slate-900">Prompt Editor</h2>
          <button
            className="rounded border border-slate-300 px-3 py-1 text-sm"
            onClick={onCancel}
            type="button"
          >
            Close
          </button>
        </div>

        <div className="grid gap-2 md:grid-cols-3">
          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Stage Label
            <input
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) => setLabel(event.target.value)}
              value={label}
            />
          </label>

          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Stage Type
            <input
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) => setStageType(event.target.value)}
              value={stageType}
            />
          </label>

          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Session Group
            <select
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) => setSessionGroup(event.target.value)}
              value={sessionGroup}
            >
              {SESSION_GROUP_OPTIONS.map((group) => (
                <option key={group} value={group}>
                  {group}
                </option>
              ))}
            </select>
          </label>

          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Provider
            <select
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) => {
                const nextProvider = event.target.value;
                const providerInfo = providers.find(
                  (entry) => entry.name === nextProvider,
                );
                setProvider(nextProvider);
                setModel(providerInfo?.models[0] ?? model);
              }}
              value={provider}
            >
              {[provider, ...providers.map((entry) => entry.name)]
                .filter((value, position, array) => array.indexOf(value) === position)
                .map((providerName) => (
                  <option key={providerName} value={providerName}>
                    {providerName}
                  </option>
                ))}
            </select>
          </label>

          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Model
            <select
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) => setModel(event.target.value)}
              value={model}
            >
              {modelOptions.map((option) => (
                <option key={option} value={option}>
                  {option}
                </option>
              ))}
            </select>
          </label>

          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Execution Intent
            <select
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) =>
                setExecutionIntent(event.target.value as StageExecutionIntent)
              }
              value={executionIntent}
            >
              {EXECUTION_INTENTS.map((intent) => (
                <option key={intent} value={intent}>
                  {intent}
                </option>
              ))}
            </select>
          </label>
        </div>

        <div className="mt-3 flex flex-wrap gap-2">
          {VARIABLE_CHIPS.map((variable) => (
            <button
              className="rounded border border-slate-300 px-2 py-1 text-xs"
              key={variable}
              onClick={() => insertVariable(variable)}
              type="button"
            >
              {variable}
            </button>
          ))}
        </div>

        <textarea
          className="mt-3 min-h-[280px] w-full rounded border border-slate-300 bg-slate-50 p-3 font-mono text-sm"
          onChange={(event) => setDraft(event.target.value)}
          ref={textareaRef}
          value={draft}
        />

        <div className="mt-3 flex items-center gap-2">
          <button
            className="rounded border border-slate-300 px-3 py-2 text-sm"
            disabled={isEnhancing}
            onClick={() => {
              void enhancePrompt();
            }}
            type="button"
          >
            {isEnhancing ? "Enhancing..." : "Enhance"}
          </button>
          <button
            className="rounded border border-slate-300 px-3 py-2 text-sm"
            onClick={onCancel}
            type="button"
          >
            Cancel
          </button>
          <button
            className="rounded bg-slate-900 px-3 py-2 text-sm text-white"
            onClick={() =>
              onSave({
                ...stage,
                label,
                stageType,
                handler: stageType,
                provider,
                model,
                sessionGroup,
                executionIntent,
                promptTemplate: draft,
              })}
            type="button"
          >
            Save Stage
          </button>
        </div>

        {enhancedDraft ? (
          <div className="mt-4 rounded border border-slate-200 bg-slate-50 p-3">
            <p className="mb-2 text-sm font-semibold text-slate-800">
              Enhanced Prompt Diff
            </p>
            <div className="grid gap-3 md:grid-cols-2">
              <div>
                <p className="mb-1 text-xs font-semibold text-slate-700">Before</p>
                <pre className="max-h-56 overflow-auto whitespace-pre-wrap rounded bg-white p-2 text-xs text-slate-700">
                  {draft}
                </pre>
              </div>
              <div>
                <p className="mb-1 text-xs font-semibold text-slate-700">After</p>
                <pre className="max-h-56 overflow-auto whitespace-pre-wrap rounded bg-white p-2 text-xs text-slate-700">
                  {enhancedDraft}
                </pre>
              </div>
            </div>
            <div className="mt-2 flex gap-2">
              <button
                className="rounded border border-slate-300 px-2 py-1 text-xs"
                onClick={() => {
                  setDraft(enhancedDraft);
                  setEnhancedDraft(null);
                }}
                type="button"
              >
                Accept
              </button>
              <button
                className="rounded border border-slate-300 px-2 py-1 text-xs"
                onClick={() => setEnhancedDraft(null)}
                type="button"
              >
                Reject
              </button>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}
