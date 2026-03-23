export interface ProjectEntry {
  id: string;
  name: string;
  path: string;
  createdAt: string;
  updatedAt: string;
}

export interface SessionMeta {
  id: string;
  projectId: string;
  title: string;
  createdAt: string;
  updatedAt: string;
}

export interface SessionDetail {
  session: SessionMeta;
  runs: RunDetail[];
}

export interface RunDetail {
  id: string;
  sessionId: string;
  status: string;
  prompt: string;
  startedAt: string;
  endedAt?: string;
  iterationCount: number;
  totalTokens?: number;
  totalCost?: number;
  pipelineTemplateId?: string;
  pipelineTemplateName?: string;
  sessionRefs?: Record<string, string>;
}

export interface GitBaseline {
  branch: string;
  commit: string;
  status: string;
}
