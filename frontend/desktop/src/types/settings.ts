export interface AppSettings {
  maxIterations: number;
  requireGit: boolean;
  requirePlanApproval: boolean;
  planAutoApproveTimeoutSec: number;
  retentionDays: number;
  agentRetryCount: number;
  agentTimeoutMs: number;
  agentMaxTurns: number;
  hiveApiHost: string;
  hiveApiPort: number;
  defaultPipelineId: string;
  autoStartHiveApi: boolean;
  settingsVersion: number;
}

export const DEFAULT_SETTINGS: AppSettings = {
  maxIterations: 5,
  requireGit: true,
  requirePlanApproval: true,
  planAutoApproveTimeoutSec: 30,
  retentionDays: 30,
  agentRetryCount: 2,
  agentTimeoutMs: 300000,
  agentMaxTurns: 25,
  hiveApiHost: "127.0.0.1",
  hiveApiPort: 8000,
  defaultPipelineId: "full-review-loop",
  autoStartHiveApi: true,
  settingsVersion: 2,
};
