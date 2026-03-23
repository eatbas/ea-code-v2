export interface PipelineTemplate {
  id: string;
  name: string;
  description: string;
  isBuiltin: boolean;
  maxIterations: number;
  stopOnFirstPass: boolean;
  stages: StageDefinition[];
  createdAt: string;
  updatedAt: string;
}

export interface StageDefinition {
  id: string;
  label: string;
  stageType: string;
  position: number;
  provider: string;
  model: string;
  sessionGroup: string;
  parallelGroup?: string | null;
  promptTemplate: string;
  enabled: boolean;
  executionIntent: "text" | "code";
}

export interface CreateTemplateRequest {
  name: string;
  description: string;
  maxIterations: number;
  stopOnFirstPass: boolean;
  stages: StageDefinition[];
}

export interface UpdateTemplateRequest {
  name: string;
  description: string;
  maxIterations: number;
  stopOnFirstPass: boolean;
  stages: StageDefinition[];
}

export interface CloneTemplateRequest {
  newName: string;
}
