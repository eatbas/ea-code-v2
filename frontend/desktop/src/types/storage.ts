export interface ChatMessage {
  role: string;
  content: string;
  timestamp: string;
  stageId?: string;
}

export interface RunSummaryFile {
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
