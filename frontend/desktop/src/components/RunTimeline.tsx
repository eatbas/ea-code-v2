import type { ReactNode } from "react";
import type {
  PipelineStageEventRecord,
  PipelineStageExecutionStatus,
  PipelineStageState,
} from "../types";
import { formatTime } from "../utils/formatTime";
import { sessionGroupClass } from "../utils/sessionGroupClass";

interface RunTimelineProps {
  stages: PipelineStageState[];
  stageEvents: PipelineStageEventRecord[];
  stageOrder: string[];
  stageLabels: Record<string, string>;
  stageSessionGroups: Record<string, string>;
  currentNodeId?: string;
  maxIterations: number;
  currentIteration: number;
}

function statusClass(status: PipelineStageExecutionStatus): string {
  if (status === "running") return "bg-blue-100 text-blue-800";
  if (status === "completed") return "bg-emerald-100 text-emerald-800";
  if (status === "failed") return "bg-red-100 text-red-800";
  if (status === "cancelled") return "bg-amber-100 text-amber-800";
  return "bg-slate-100 text-slate-800";
}

export function RunTimeline({
  stages,
  stageEvents,
  stageOrder,
  stageLabels,
  stageSessionGroups,
  currentNodeId,
  maxIterations,
  currentIteration,
}: RunTimelineProps): ReactNode {
  if (stages.length === 0 && stageEvents.length === 0) {
    return (
      <div className="rounded border border-dashed border-slate-300 p-4 text-sm text-slate-500">
        No stage events yet.
      </div>
    );
  }

  const stageStatusMap = new Map<string, PipelineStageExecutionStatus>();
  for (const stage of stages) {
    stageStatusMap.set(stage.nodeId, stage.status);
  }

  const derivedOrder = stageOrder.length > 0
    ? stageOrder
    : Array.from(new Set(stages.map((stage) => stage.nodeId)));
  const compactMode = derivedOrder.length <= 2;
  const expandedMode = derivedOrder.length >= 5;

  const recentEvents = stageEvents.slice(-10).reverse();

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <strong className="text-sm text-slate-900">Run Timeline</strong>
        <span className="text-xs text-slate-600">
          Iteration {currentIteration}/{Math.max(1, maxIterations)}
        </span>
      </div>

      <ol
        className={
          compactMode
            ? "grid grid-cols-2 gap-2"
            : expandedMode
              ? "grid grid-cols-1 gap-2 md:grid-cols-2"
              : "space-y-2"
        }
      >
        {derivedOrder.map((nodeId, index) => {
          const stage = stages.find((candidate) => candidate.nodeId === nodeId);
          const status = stageStatusMap.get(nodeId) ?? "pending";
          const label = stageLabels[nodeId] ?? stage?.nodeLabel ?? nodeId;
          const sessionGroup = stageSessionGroups[nodeId];
          const isCurrent = currentNodeId === nodeId && status === "running";

          return (
            <li
              className={`rounded border p-3 ${
                isCurrent
                  ? "border-blue-300 bg-blue-50"
                  : "border-slate-200 bg-white"
              }`}
              key={nodeId}
            >
              <div className="mb-1 flex items-center justify-between gap-2">
                <strong className="text-sm text-slate-900">
                  {index + 1}. {label}
                </strong>
                <span className={`rounded px-2 py-0.5 text-xs ${statusClass(status)}`}>
                  {status}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs text-slate-500">Node: {nodeId}</span>
                <span
                  className={`rounded px-2 py-0.5 text-[11px] ${sessionGroupClass(
                    sessionGroup,
                  )}`}
                >
                  Group {sessionGroup ?? "—"}
                </span>
              </div>
              {stage?.detail ? (
                <p className="mt-1 text-xs text-slate-600">{stage.detail}</p>
              ) : null}
            </li>
          );
        })}
      </ol>

      {recentEvents.length > 0 ? (
        <div className="rounded border border-slate-200 bg-white p-3">
          <p className="mb-2 text-xs font-semibold text-slate-700">
            Recent Stage Activity
          </p>
          <ul className="space-y-1">
            {recentEvents.map((event, index) => (
              <li className="text-xs text-slate-600" key={`${event.nodeId}-${event.timestamp}-${index}`}>
                {formatTime(event.timestamp, "time")} · {event.nodeLabel} · {event.status}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
