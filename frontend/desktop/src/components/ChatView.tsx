import type { ReactNode } from "react";
import { useMemo, useState } from "react";
import { usePipelineContext } from "../contexts/PipelineContext";
import { useTemplateContext } from "../contexts/TemplateContext";
import { sessionGroupClass } from "../utils/sessionGroupClass";
import { RunTimeline } from "./RunTimeline";

function normaliseLogLines(chunks: string[]): string {
  const joined = chunks.join("");
  return joined.trim().length > 0 ? joined : "No output yet.";
}

export function ChatView(): ReactNode {
  const {
    run,
    currentStage,
    stages,
    stageEvents,
    logs,
    artifacts,
    pendingQuestion,
    error,
    loading,
    pausePipeline,
    resumePipeline,
    cancelPipeline,
    answerQuestion,
  } = usePipelineContext();
  const { templates } = useTemplateContext();
  const [answerText, setAnswerText] = useState("");

  const templateForRun = useMemo(
    () =>
      run?.templateId
        ? templates.find((template) => template.id === run.templateId) ?? null
        : null,
    [run?.templateId, templates],
  );

  const enabledNodes = useMemo(
    () => templateForRun?.nodes.filter((node) => node.enabled) ?? [],
    [templateForRun],
  );

  const stageOrder = useMemo(
    () =>
      enabledNodes.length > 0
        ? enabledNodes.map((node) => node.id)
        : Array.from(new Set(stages.map((stage) => stage.nodeId))),
    [enabledNodes, stages],
  );

  const stageLabels = useMemo(() => {
    const labels: Record<string, string> = {};
    for (const node of enabledNodes) {
      labels[node.id] = node.label;
    }
    for (const stage of stages) {
      if (!labels[stage.nodeId]) {
        labels[stage.nodeId] = stage.nodeLabel;
      }
    }
    return labels;
  }, [enabledNodes, stages]);

  const stageSessionGroups = useMemo(() => {
    const groups: Record<string, string> = {};
    for (const node of enabledNodes) {
      groups[node.id] = node.sessionGroup;
    }
    return groups;
  }, [enabledNodes]);

  const stageExecutionIntent = useMemo(() => {
    const intent: Record<string, "text" | "code"> = {};
    for (const node of enabledNodes) {
      intent[node.id] = node.executionIntent;
    }
    return intent;
  }, [enabledNodes]);

  const groupedLogs = useMemo(() => {
    const grouped = new Map<string, string[]>();
    for (const entry of logs) {
      const existing = grouped.get(entry.nodeId) ?? [];
      existing.push(entry.text);
      grouped.set(entry.nodeId, existing);
    }
    return grouped;
  }, [logs]);

  const diffByNode = useMemo(() => {
    const grouped = new Map<string, string[]>();
    for (const artifact of artifacts) {
      if (artifact.artifactType !== "git_diff") continue;
      const existing = grouped.get(artifact.nodeId) ?? [];
      existing.push(artifact.content);
      grouped.set(artifact.nodeId, existing);
    }
    return grouped;
  }, [artifacts]);

  const totalStages = Math.max(1, stageOrder.length || stages.length);
  const completedStages = stages.filter((stage) => stage.status === "completed").length;
  const progressPercent = Math.round((completedStages / totalStages) * 100);

  const firstNodeId = stageOrder[0];
  const inferredIteration = useMemo(() => {
    if (!firstNodeId) return 1;
    const starts = stageEvents.filter(
      (event) => event.nodeId === firstNodeId && event.status === "running",
    ).length;
    return Math.max(1, starts);
  }, [firstNodeId, stageEvents]);
  const maxIterations = templateForRun?.maxIterations ?? 1;

  const runStatus = run?.status ?? "idle";

  return (
    <div className="flex h-full flex-col gap-4 bg-slate-50 p-6">
      <section className="rounded border border-slate-200 bg-white p-4">
        <div className="mb-2 flex items-center justify-between gap-3">
          <strong className="text-sm text-slate-900">Execution Overview</strong>
          <span className="rounded bg-slate-100 px-2 py-1 text-xs text-slate-700">
            {runStatus}
          </span>
        </div>
        <div className="grid gap-1 text-xs text-slate-600 md:grid-cols-2">
          <p>Run ID: {run?.runId ?? "n/a"}</p>
          <p>Mode: {run?.mode ?? "n/a"}</p>
          <p>
            Iteration: {inferredIteration}/{Math.max(1, maxIterations)}
          </p>
          <p>
            Stage Progress: {completedStages}/{totalStages}
          </p>
        </div>
        <div className="mt-3 h-2 w-full overflow-hidden rounded bg-slate-200">
          <div
            className="h-full bg-slate-800 transition-[width]"
            style={{ width: `${Math.min(100, Math.max(0, progressPercent))}%` }}
          />
        </div>
      </section>

      <div className="flex flex-wrap items-center gap-2">
        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          disabled={loading || runStatus !== "running"}
          onClick={() => {
            void pausePipeline();
          }}
          type="button"
        >
          Pause
        </button>
        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          disabled={loading || runStatus !== "paused"}
          onClick={() => {
            void resumePipeline();
          }}
          type="button"
        >
          Resume
        </button>
        <button
          className="rounded border border-red-300 px-3 py-2 text-sm text-red-700"
          disabled={loading || runStatus === "idle"}
          onClick={() => {
            void cancelPipeline();
          }}
          type="button"
        >
          Cancel
        </button>
      </div>

      {error ? (
        <div className="rounded border border-red-200 bg-red-50 p-3 text-sm text-red-800">
          {error}
        </div>
      ) : null}

      {pendingQuestion ? (
        <section className="rounded border border-amber-200 bg-amber-50 p-4">
          <p className="mb-2 text-sm font-medium text-amber-900">
            {pendingQuestion.questionText}
          </p>
          <div className="flex items-center gap-2">
            <input
              className="flex-1 rounded border border-amber-300 px-3 py-2 text-sm"
              onChange={(event) => setAnswerText(event.target.value)}
              placeholder="Type your answer"
              value={answerText}
            />
            <button
              className="rounded bg-amber-700 px-3 py-2 text-sm text-white"
              onClick={() => {
                void answerQuestion(pendingQuestion.questionId, answerText);
                setAnswerText("");
              }}
              type="button"
            >
              Submit
            </button>
          </div>
        </section>
      ) : null}

      <RunTimeline
        currentIteration={inferredIteration}
        currentNodeId={currentStage?.nodeId}
        maxIterations={maxIterations}
        stageEvents={stageEvents}
        stageLabels={stageLabels}
        stageOrder={stageOrder}
        stageSessionGroups={stageSessionGroups}
        stages={stages}
      />

      <section className="rounded border border-slate-200 bg-white p-4">
        <strong className="mb-3 block text-sm text-slate-900">Dynamic Stage Cards</strong>
        {stageOrder.length === 0 ? (
          <p className="text-xs text-slate-600">Waiting for stages...</p>
        ) : (
          <div className="space-y-3">
            {stageOrder.map((nodeId, index) => {
              const stage = stages.find((candidate) => candidate.nodeId === nodeId);
              const label = stageLabels[nodeId] ?? nodeId;
              const status = stage?.status ?? "pending";
              const sessionGroup = stageSessionGroups[nodeId];
              const logsForStage = groupedLogs.get(nodeId) ?? [];
              const intent =
                stageExecutionIntent[nodeId] ??
                (run?.mode === "direct_task" ? "code" : "text");
              const diff = diffByNode.get(nodeId)?.join("\n\n");

              return (
                <article
                  className="rounded border border-slate-200 bg-slate-50 p-3"
                  key={nodeId}
                >
                  <div className="mb-2 flex items-center justify-between gap-2">
                    <strong className="text-sm text-slate-900">
                      {index + 1}. {label}
                    </strong>
                    <div className="flex items-center gap-2">
                      <span
                        className={`rounded px-2 py-0.5 text-[11px] ${sessionGroupClass(
                          sessionGroup,
                        )}`}
                      >
                        Group {sessionGroup ?? "—"}
                      </span>
                      <span className="rounded bg-slate-100 px-2 py-0.5 text-[11px] text-slate-700">
                        {status}
                      </span>
                    </div>
                  </div>

                  <p className="mb-2 text-xs text-slate-600">Node: {nodeId}</p>

                  <pre className="max-h-40 overflow-auto whitespace-pre-wrap rounded bg-white p-2 text-xs text-slate-700">
                    {normaliseLogLines(logsForStage)}
                  </pre>

                  {intent === "code" ? (
                    <details className="mt-2 rounded border border-slate-200 bg-white p-2">
                      <summary className="cursor-pointer text-xs font-semibold text-slate-700">
                        Diff Viewer
                      </summary>
                      <pre className="mt-2 max-h-48 overflow-auto whitespace-pre-wrap text-xs text-slate-700">
                        {diff ?? "No code diff artefact for this stage yet."}
                      </pre>
                    </details>
                  ) : null}
                </article>
              );
            })}
          </div>
        )}
      </section>
    </div>
  );
}
