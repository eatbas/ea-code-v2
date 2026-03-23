export interface PipelineTemplate {
  id: string;
  name: string;
  description: string;
  isBuiltin: boolean;
  maxIterations: number;
  stopOnFirstPass: boolean;
  nodes: StageNodeDefinition[];
  edges: StageEdgeDefinition[];
  createdAt: string;
  updatedAt: string;
}

export interface StageNodeUiPosition {
  x: number;
  y: number;
}

export type StageExecutionIntent = "text" | "code";

export interface StageNodeDefinition {
  id: string;
  label: string;
  stageType: string;
  handler: string;
  config?: Record<string, unknown> | null;
  provider: string;
  model: string;
  sessionGroup: string;
  promptTemplate: string;
  enabled: boolean;
  executionIntent: StageExecutionIntent;
  uiPosition: StageNodeUiPosition;
}

export enum StageEdgeCondition {
  Always = "always",
  OnSuccess = "on_success",
  OnFailure = "on_failure",
}

export interface StageEdgeDefinition {
  id: string;
  sourceNodeId: string;
  targetNodeId: string;
  condition: StageEdgeCondition;
  inputKey?: string | null;
  loopControl: boolean;
}

export interface CreateTemplateRequest {
  name: string;
  description: string;
  maxIterations: number;
  stopOnFirstPass: boolean;
  nodes: StageNodeDefinition[];
  edges: StageEdgeDefinition[];
}

export interface UpdateTemplateRequest {
  name: string;
  description: string;
  maxIterations: number;
  stopOnFirstPass: boolean;
  nodes: StageNodeDefinition[];
  edges: StageEdgeDefinition[];
}

export interface CloneTemplateRequest {
  newName: string;
}
