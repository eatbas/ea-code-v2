import type { DragEvent, ReactNode } from "react";
import type { ProviderInfo, StageExecutionIntent, StageNodeDefinition } from "../../types";
import { inferModels } from "../../utils/inferModels";
import { EXECUTION_INTENTS, SESSION_GROUP_OPTIONS } from "./constants";
import { SessionGroupIndicator } from "./SessionGroupIndicator";

interface StageCardProps {
  node: StageNodeDefinition;
  index: number;
  total: number;
  providers: ProviderInfo[];
  advancedSessionMode: boolean;
  resumeFromPrevious: boolean;
  showSessionBreakBefore: boolean;
  onToggleResumeFromPrevious: (index: number, resume: boolean) => void;
  onMove: (index: number, direction: "up" | "down") => void;
  onDelete: (nodeId: string) => void;
  onOpenPromptEditor: (nodeId: string) => void;
  onUpdate: (node: StageNodeDefinition) => void;
  onDragStart: (nodeId: string) => void;
  onDrop: (targetNodeId: string) => void;
}

export function StageCard({
  node,
  index,
  total,
  providers,
  advancedSessionMode,
  resumeFromPrevious,
  showSessionBreakBefore,
  onToggleResumeFromPrevious,
  onMove,
  onDelete,
  onOpenPromptEditor,
  onUpdate,
  onDragStart,
  onDrop,
}: StageCardProps): ReactNode {
  const models = inferModels(node, providers);

  const onDragOver = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
  };

  return (
    <>
      {showSessionBreakBefore ? (
        <div className="mx-1 border-t border-dashed border-slate-300 pt-2">
          <span className="text-[11px] text-slate-500">Session break</span>
        </div>
      ) : null}

      <div
        className="rounded border border-slate-200 bg-white p-3"
        draggable
        onDragOver={onDragOver}
        onDragStart={() => onDragStart(node.id)}
        onDrop={() => onDrop(node.id)}
      >
        <div className="mb-2 flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <span className="cursor-grab text-sm text-slate-400" title="Drag to reorder">
              ::
            </span>
            <strong className="text-sm text-slate-900">{node.label}</strong>
            <SessionGroupIndicator group={node.sessionGroup} />
          </div>

          <div className="flex items-center gap-2">
            <button
              className="rounded border border-slate-300 px-2 py-1 text-xs"
              disabled={index === 0}
              onClick={() => onMove(index, "up")}
              type="button"
            >
              ↑
            </button>
            <button
              className="rounded border border-slate-300 px-2 py-1 text-xs"
              disabled={index === total - 1}
              onClick={() => onMove(index, "down")}
              type="button"
            >
              ↓
            </button>
            <button
              className="rounded border border-slate-300 px-2 py-1 text-xs"
              onClick={() => onOpenPromptEditor(node.id)}
              type="button"
            >
              Edit Prompt
            </button>
            <button
              className="rounded border border-red-300 px-2 py-1 text-xs text-red-700"
              onClick={() => onDelete(node.id)}
              type="button"
            >
              Delete
            </button>
          </div>
        </div>

        <div className="grid gap-2 md:grid-cols-2">
          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Label
            <input
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) =>
                onUpdate({
                  ...node,
                  label: event.target.value,
                })}
              value={node.label}
            />
          </label>

          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Stage Type
            <input
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) =>
                onUpdate({
                  ...node,
                  stageType: event.target.value,
                  handler: event.target.value,
                })}
              value={node.stageType}
            />
          </label>

          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Provider
            <select
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) => {
                const providerName = event.target.value;
                const provider = providers.find((entry) => entry.name === providerName);
                onUpdate({
                  ...node,
                  provider: providerName,
                  model: provider?.models[0] ?? node.model,
                });
              }}
              value={node.provider}
            >
              {[node.provider, ...providers.map((provider) => provider.name)]
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
              onChange={(event) =>
                onUpdate({
                  ...node,
                  model: event.target.value,
                })}
              value={node.model}
            >
              {models.map((model) => (
                <option key={model} value={model}>
                  {model}
                </option>
              ))}
            </select>
          </label>

          <label className="flex flex-col gap-1 text-xs text-slate-700">
            Execution Intent
            <select
              className="rounded border border-slate-300 px-2 py-1"
              onChange={(event) =>
                onUpdate({
                  ...node,
                  executionIntent: event.target.value as StageExecutionIntent,
                })}
              value={node.executionIntent}
            >
              {EXECUTION_INTENTS.map((intent) => (
                <option key={intent} value={intent}>
                  {intent}
                </option>
              ))}
            </select>
          </label>

          {advancedSessionMode ? (
            <label className="flex flex-col gap-1 text-xs text-slate-700">
              Session Group
              <select
                className="rounded border border-slate-300 px-2 py-1"
                onChange={(event) =>
                  onUpdate({
                    ...node,
                    sessionGroup: event.target.value,
                  })}
                value={node.sessionGroup}
              >
                {SESSION_GROUP_OPTIONS.map((group) => (
                  <option key={group} value={group}>
                    {group}
                  </option>
                ))}
              </select>
            </label>
          ) : index > 0 ? (
            <label className="mt-5 flex items-center gap-2 text-xs text-slate-700">
              <input
                checked={resumeFromPrevious}
                onChange={(event) =>
                  onToggleResumeFromPrevious(index, event.target.checked)
                }
                type="checkbox"
              />
              Resume from previous
            </label>
          ) : (
            <div className="mt-5 text-xs text-slate-500">First stage starts new session</div>
          )}
        </div>

        <label className="mt-2 flex items-center gap-2 text-xs text-slate-700">
          <input
            checked={node.enabled}
            onChange={(event) =>
              onUpdate({
                ...node,
                enabled: event.target.checked,
              })}
            type="checkbox"
          />
          Enabled
        </label>
      </div>
    </>
  );
}
