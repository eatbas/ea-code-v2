import { useMemo, useState, type MouseEvent } from "react";
import { usePipelineRun } from "../../hooks/usePipelineRun";
import { usePipelineTemplates } from "../../hooks/usePipelineTemplates";
import { StageEdgeCondition, type PipelineTemplate, type StageExecutionIntent } from "../../types";
import {
  connectNodes,
  createNode,
  deleteNode,
  disconnectEdge,
  duplicateNode,
  moveNode,
  renameNode,
  validateGraph,
} from "./graph";
import { createPipelineBuilderGraphState, type PipelineBuilderGraphState } from "./model";

interface PipelineBuilderProps {
  value: PipelineBuilderGraphState;
  onChange: (next: PipelineBuilderGraphState) => void;
  templateId?: string;
  templateName?: string;
  templateDescription?: string;
  maxIterations?: number;
  stopOnFirstPass?: boolean;
  workspacePath?: string;
}

interface DragState {
  nodeId: string;
  offsetX: number;
  offsetY: number;
}

function makeNodeLabel(kind: string): string {
  if (kind === "analyse") return "Analyse";
  if (kind === "review") return "Review";
  if (kind === "implement") return "Implement";
  return "Node";
}

function makeNodePrompt(kind: string): string {
  if (kind === "analyse") return "Analyse the task carefully.\n\nTask: {{task}}";
  if (kind === "review") return "Review upstream outputs.\n\n{{upstream_outputs}}";
  if (kind === "implement") return "Implement safely.\n\n{{previous_output}}";
  return "{{task}}";
}

function makeExecutionIntent(kind: string): StageExecutionIntent {
  return kind === "implement" ? "code" : "text";
}

export function PipelineBuilder({
  value,
  onChange,
  templateId,
  templateName = "Graph Template",
  templateDescription = "Visual pipeline",
  maxIterations = 1,
  stopOnFirstPass = true,
  workspacePath = ".",
}: PipelineBuilderProps) {
  const [drag, setDrag] = useState<DragState | null>(null);
  const [pendingConnectionSource, setPendingConnectionSource] = useState<string | null>(null);
  const [runPrompt, setRunPrompt] = useState("Implement requested change");

  const graph = useMemo(() => value ?? createPipelineBuilderGraphState(), [value]);
  const validation = useMemo(() => validateGraph(graph), [graph]);
  const { updateTemplate, createTemplate } = usePipelineTemplates();
  const { startPipelineRun, loading: runLoading } = usePipelineRun();

  const nodeMap = useMemo(() => {
    const map = new Map<string, { x: number; y: number }>();
    for (const node of graph.nodes) {
      map.set(node.id, {
        x: node.uiPosition.x + 110,
        y: node.uiPosition.y + 50,
      });
    }
    return map;
  }, [graph.nodes]);

  const addNode = (kind: string) => {
    const next = createNode(graph, {
      label: makeNodeLabel(kind),
      stageType: kind,
      handler: kind === "analyse" || kind === "review" || kind === "implement" ? "chat" : kind,
      provider: "claude",
      model: "sonnet",
      sessionGroup: "A",
      promptTemplate: makeNodePrompt(kind),
      executionIntent: makeExecutionIntent(kind),
      uiPosition: { x: 40 + graph.nodes.length * 24, y: 40 + graph.nodes.length * 18 },
    });
    onChange(next);
  };

  const beginDrag = (event: MouseEvent<HTMLDivElement>, nodeId: string) => {
    const node = graph.nodes.find((item) => item.id === nodeId);
    if (!node) return;
    setDrag({ nodeId, offsetX: event.clientX - node.uiPosition.x, offsetY: event.clientY - node.uiPosition.y });
  };

  const onCanvasMouseMove = (event: MouseEvent<HTMLDivElement>) => {
    if (!drag) return;
    onChange(
      moveNode(graph, {
        nodeId: drag.nodeId,
        uiPosition: { x: Math.max(0, event.clientX - drag.offsetX), y: Math.max(0, event.clientY - drag.offsetY) },
      }),
    );
  };

  const renameSelectedNode = (nodeId: string) => {
    const node = graph.nodes.find((item) => item.id === nodeId);
    if (!node) return;
    const nextLabel = window.prompt("Node label", node.label);
    if (!nextLabel) return;
    onChange(renameNode(graph, { nodeId, label: nextLabel }));
  };

  const saveTemplate = async () => {
    const payload = {
      name: templateName,
      description: templateDescription,
      maxIterations,
      stopOnFirstPass,
      nodes: graph.nodes,
      edges: graph.edges,
    };

    if (templateId) {
      await updateTemplate(templateId, payload);
      return;
    }

    await createTemplate(payload);
  };

  const runTemplate = async () => {
    const template: PipelineTemplate = {
      id: templateId ?? "unsaved-template",
      name: templateName,
      description: templateDescription,
      isBuiltin: false,
      maxIterations,
      stopOnFirstPass,
      nodes: graph.nodes,
      edges: graph.edges,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    await startPipelineRun({
      prompt: runPrompt,
      workspacePath,
      template,
    });
  };

  return (
    <div className="flex h-full w-full flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <button className="rounded border px-3 py-1 text-sm" onClick={() => addNode("analyse")} type="button">Add Analyse</button>
        <button className="rounded border px-3 py-1 text-sm" onClick={() => addNode("review")} type="button">Add Review</button>
        <button className="rounded border px-3 py-1 text-sm" onClick={() => addNode("implement")} type="button">Add Implement</button>
        <button className="rounded border px-3 py-1 text-sm" onClick={saveTemplate} type="button">Save</button>
        <button className="rounded border px-3 py-1 text-sm" disabled={runLoading} onClick={runTemplate} type="button">Run</button>
        {pendingConnectionSource ? <span className="text-sm">Wiring from: {pendingConnectionSource}</span> : null}
      </div>

      <input
        className="rounded border px-2 py-1 text-sm"
        onChange={(event) => setRunPrompt(event.target.value)}
        placeholder="Run prompt"
        value={runPrompt}
      />

      <div className="relative h-[520px] w-full overflow-hidden rounded border bg-slate-50" onMouseMove={onCanvasMouseMove} onMouseUp={() => setDrag(null)} role="application">
        <svg className="pointer-events-none absolute inset-0 h-full w-full">
          {graph.edges.map((edge) => {
            const source = nodeMap.get(edge.sourceNodeId);
            const target = nodeMap.get(edge.targetNodeId);
            if (!source || !target) return null;
            return (
              <line key={edge.id} stroke="#334155" strokeWidth={2} x1={source.x} x2={target.x} y1={source.y} y2={target.y} />
            );
          })}
        </svg>

        {graph.nodes.map((node) => (
          <div
            key={node.id}
            className="absolute w-[220px] cursor-move rounded border bg-white p-2 shadow"
            onMouseDown={(event) => beginDrag(event, node.id)}
            style={{ left: node.uiPosition.x, top: node.uiPosition.y }}
          >
            <div className="mb-2 flex items-center justify-between gap-2">
              <strong className="text-sm">{node.label}</strong>
              <div className="flex gap-1">
                <button className="rounded border px-2 py-0.5 text-xs" onClick={() => renameSelectedNode(node.id)} type="button">Rename</button>
                <button className="rounded border px-2 py-0.5 text-xs" onClick={() => onChange(duplicateNode(graph, { nodeId: node.id }))} type="button">Duplicate</button>
                <button className="rounded border px-2 py-0.5 text-xs" onClick={() => onChange(deleteNode(graph, { nodeId: node.id }))} type="button">Delete</button>
              </div>
            </div>
            <div className="mb-2 text-xs">{node.id}</div>
            {(validation.nodeErrors[node.id] ?? []).map((error) => (
              <div className="text-xs text-red-600" key={error}>{error}</div>
            ))}
            <div className="mt-2 flex gap-2">
              <button className="rounded border px-2 py-0.5 text-xs" onClick={() => setPendingConnectionSource(node.id)} type="button">Wire From</button>
              <button
                className="rounded border px-2 py-0.5 text-xs"
                onClick={() => {
                  if (!pendingConnectionSource || pendingConnectionSource === node.id) return;
                  onChange(connectNodes(graph, { sourceNodeId: pendingConnectionSource, targetNodeId: node.id, condition: StageEdgeCondition.Always }));
                  setPendingConnectionSource(null);
                }}
                type="button"
              >
                Wire To
              </button>
            </div>
          </div>
        ))}
      </div>

      <div className="grid gap-2">
        {graph.edges.map((edge) => (
          <div className="flex items-center justify-between rounded border bg-white p-2 text-sm" key={edge.id}>
            <span>{edge.sourceNodeId} → {edge.targetNodeId} ({edge.condition})</span>
            <div className="flex items-center gap-2">
              {(validation.edgeErrors[edge.id] ?? []).map((error) => (
                <span className="text-xs text-red-600" key={error}>{error}</span>
              ))}
              <button className="rounded border px-2 py-0.5 text-xs" onClick={() => onChange(disconnectEdge(graph, { edgeId: edge.id }))} type="button">Remove</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
