import {
  type StageEdgeDefinition,
  StageEdgeCondition,
  type StageNodeDefinition,
} from "../../types";
import type {
  ConnectNodesInput,
  CreateNodeInput,
  DuplicateNodeInput,
  DeleteNodeInput,
  DisconnectEdgeInput,
  DisconnectNodesInput,
  MoveNodeInput,
  PipelineBuilderGraphState,
  RenameNodeInput,
} from "./model";

const NODE_ID_PREFIX = "node";
const EDGE_ID_PREFIX = "edge";

function nextGraphId(
  existingIds: readonly string[],
  prefix: string,
): string {
  const prefixWithDash = `${prefix}-`;
  const nextIndex =
    existingIds.reduce((max, candidate) => {
      if (!candidate.startsWith(prefixWithDash)) {
        return max;
      }

      const indexText = candidate.slice(prefixWithDash.length);
      const parsed = Number.parseInt(indexText, 10);
      if (Number.isNaN(parsed)) {
        return max;
      }

      return Math.max(max, parsed);
    }, 0) + 1;

  return `${prefix}-${nextIndex}`;
}

function assertNodeExists(
  nodes: readonly StageNodeDefinition[],
  nodeId: string,
): void {
  if (!nodes.some((node) => node.id === nodeId)) {
    throw new Error(`Node does not exist: ${nodeId}`);
  }
}

function edgeExists(
  edges: readonly StageEdgeDefinition[],
  input: ConnectNodesInput,
): boolean {
  const condition = input.condition ?? StageEdgeCondition.Always;
  const inputKey = input.inputKey ?? null;
  const loopControl = input.loopControl ?? false;
  return edges.some(
    (edge) =>
      edge.sourceNodeId === input.sourceNodeId &&
      edge.targetNodeId === input.targetNodeId &&
      edge.condition === condition &&
      (edge.inputKey ?? null) === inputKey &&
      edge.loopControl === loopControl,
  );
}

export function createNode(
  state: PipelineBuilderGraphState,
  input: CreateNodeInput,
): PipelineBuilderGraphState {
  const nodeId = input.id ?? nextGraphId(state.nodes.map((node) => node.id), NODE_ID_PREFIX);
  if (state.nodes.some((node) => node.id === nodeId)) {
    throw new Error(`Node ID already exists: ${nodeId}`);
  }

  const nextNode: StageNodeDefinition = {
    id: nodeId,
    label: input.label,
    stageType: input.stageType,
    handler: input.handler,
    config: input.config ?? null,
    provider: input.provider,
    model: input.model,
    sessionGroup: input.sessionGroup,
    promptTemplate: input.promptTemplate,
    enabled: input.enabled ?? true,
    executionIntent: input.executionIntent,
    uiPosition: {
      x: input.uiPosition.x,
      y: input.uiPosition.y,
    },
  };

  return {
    nodes: [...state.nodes, nextNode],
    edges: [...state.edges],
  };
}

export function deleteNode(
  state: PipelineBuilderGraphState,
  input: DeleteNodeInput,
): PipelineBuilderGraphState {
  if (!state.nodes.some((node) => node.id === input.nodeId)) {
    return state;
  }

  return {
    nodes: state.nodes.filter((node) => node.id !== input.nodeId),
    edges: state.edges.filter(
      (edge) => edge.sourceNodeId !== input.nodeId && edge.targetNodeId !== input.nodeId,
    ),
  };
}

export function moveNode(
  state: PipelineBuilderGraphState,
  input: MoveNodeInput,
): PipelineBuilderGraphState {
  assertNodeExists(state.nodes, input.nodeId);

  return {
    nodes: state.nodes.map((node) =>
      node.id === input.nodeId
        ? {
            ...node,
            uiPosition: {
              x: input.uiPosition.x,
              y: input.uiPosition.y,
            },
          }
        : node,
    ),
    edges: [...state.edges],
  };
}

export function renameNode(
  state: PipelineBuilderGraphState,
  input: RenameNodeInput,
): PipelineBuilderGraphState {
  assertNodeExists(state.nodes, input.nodeId);
  const trimmed = input.label.trim();
  if (trimmed.length === 0) {
    throw new Error("Node label must not be empty");
  }

  return {
    nodes: state.nodes.map((node) =>
      node.id === input.nodeId ? { ...node, label: trimmed } : node,
    ),
    edges: [...state.edges],
  };
}

export function duplicateNode(
  state: PipelineBuilderGraphState,
  input: DuplicateNodeInput,
): PipelineBuilderGraphState {
  const source = state.nodes.find((node) => node.id === input.nodeId);
  if (!source) {
    throw new Error(`Node does not exist: ${input.nodeId}`);
  }

  const nodeId = input.id ?? nextGraphId(state.nodes.map((node) => node.id), NODE_ID_PREFIX);
  if (state.nodes.some((node) => node.id === nodeId)) {
    throw new Error(`Node ID already exists: ${nodeId}`);
  }

  const duplicate: StageNodeDefinition = {
    ...source,
    id: nodeId,
    label: `${source.label} Copy`,
    uiPosition: {
      x: source.uiPosition.x + 30,
      y: source.uiPosition.y + 30,
    },
  };

  return {
    nodes: [...state.nodes, duplicate],
    edges: [...state.edges],
  };
}

export function connectNodes(
  state: PipelineBuilderGraphState,
  input: ConnectNodesInput,
): PipelineBuilderGraphState {
  assertNodeExists(state.nodes, input.sourceNodeId);
  assertNodeExists(state.nodes, input.targetNodeId);
  if (input.sourceNodeId === input.targetNodeId) {
    throw new Error("Self-connections are not allowed");
  }
  if (edgeExists(state.edges, input)) {
    return state;
  }

  const edgeId = input.id ?? nextGraphId(state.edges.map((edge) => edge.id), EDGE_ID_PREFIX);
  if (state.edges.some((edge) => edge.id === edgeId)) {
    throw new Error(`Edge ID already exists: ${edgeId}`);
  }

  const nextEdge: StageEdgeDefinition = {
    id: edgeId,
    sourceNodeId: input.sourceNodeId,
    targetNodeId: input.targetNodeId,
    condition: input.condition ?? StageEdgeCondition.Always,
    inputKey: input.inputKey ?? null,
    loopControl: input.loopControl ?? false,
  };

  return {
    nodes: [...state.nodes],
    edges: [...state.edges, nextEdge],
  };
}

export function disconnectEdge(
  state: PipelineBuilderGraphState,
  input: DisconnectEdgeInput,
): PipelineBuilderGraphState {
  return {
    nodes: [...state.nodes],
    edges: state.edges.filter((edge) => edge.id !== input.edgeId),
  };
}

export function disconnectNodes(
  state: PipelineBuilderGraphState,
  input: DisconnectNodesInput,
): PipelineBuilderGraphState {
  return {
    nodes: [...state.nodes],
    edges: state.edges.filter(
      (edge) =>
        !(
          edge.sourceNodeId === input.sourceNodeId &&
          edge.targetNodeId === input.targetNodeId
        ),
    ),
  };
}

export interface GraphValidationResult {
  nodeErrors: Record<string, string[]>;
  edgeErrors: Record<string, string[]>;
}

export function validateGraph(
  state: PipelineBuilderGraphState,
): GraphValidationResult {
  const nodeErrors: Record<string, string[]> = {};
  const edgeErrors: Record<string, string[]> = {};
  const nodeIds = new Set(state.nodes.map((node) => node.id));

  for (const node of state.nodes) {
    if (node.label.trim().length === 0) {
      nodeErrors[node.id] = [...(nodeErrors[node.id] ?? []), "Label is required"];
    }
  }

  for (const edge of state.edges) {
    if (!nodeIds.has(edge.sourceNodeId)) {
      edgeErrors[edge.id] = [
        ...(edgeErrors[edge.id] ?? []),
        `Missing source node: ${edge.sourceNodeId}`,
      ];
    }
    if (!nodeIds.has(edge.targetNodeId)) {
      edgeErrors[edge.id] = [
        ...(edgeErrors[edge.id] ?? []),
        `Missing target node: ${edge.targetNodeId}`,
      ];
    }
    if (edge.sourceNodeId === edge.targetNodeId) {
      edgeErrors[edge.id] = [
        ...(edgeErrors[edge.id] ?? []),
        "Self-connections are not allowed",
      ];
    }
  }

  return { nodeErrors, edgeErrors };
}
