import {
  type StageEdgeDefinition,
  StageEdgeCondition,
  type StageExecutionIntent,
  type StageNodeDefinition,
  type StageNodeUiPosition,
} from "../../types";

export interface PipelineBuilderGraphState {
  nodes: StageNodeDefinition[];
  edges: StageEdgeDefinition[];
}

export interface PipelineBuilderSelection {
  nodeId?: string;
  edgeId?: string;
}

export interface PipelineBuilderState {
  graph: PipelineBuilderGraphState;
  selection: PipelineBuilderSelection | null;
}

export interface CreateNodeInput {
  id?: string;
  label: string;
  stageType: string;
  handler: string;
  config?: Record<string, unknown> | null;
  provider: string;
  model: string;
  sessionGroup: string;
  promptTemplate: string;
  enabled?: boolean;
  executionIntent: StageExecutionIntent;
  uiPosition: StageNodeUiPosition;
}

export interface DeleteNodeInput {
  nodeId: string;
}

export interface MoveNodeInput {
  nodeId: string;
  uiPosition: StageNodeUiPosition;
}

export interface RenameNodeInput {
  nodeId: string;
  label: string;
}

export interface DuplicateNodeInput {
  nodeId: string;
  id?: string;
}

export interface ConnectNodesInput {
  id?: string;
  sourceNodeId: string;
  targetNodeId: string;
  condition?: StageEdgeCondition;
  inputKey?: string | null;
  loopControl?: boolean;
}

export interface DisconnectEdgeInput {
  edgeId: string;
}

export interface DisconnectNodesInput {
  sourceNodeId: string;
  targetNodeId: string;
}

export function createPipelineBuilderGraphState(
  nodes: StageNodeDefinition[] = [],
  edges: StageEdgeDefinition[] = [],
): PipelineBuilderGraphState {
  return {
    nodes: [...nodes],
    edges: [...edges],
  };
}

export function createPipelineBuilderState(
  graph: PipelineBuilderGraphState = createPipelineBuilderGraphState(),
): PipelineBuilderState {
  return {
    graph,
    selection: null,
  };
}
