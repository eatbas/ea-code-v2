/** Health check response from hive-api. */
export interface HealthResponse {
  status: string;
  dronesBooted: boolean;
  providers: string[];
}

/** Provider info as returned by hive-api /v1/providers. */
export interface ProviderInfo {
  name: string;
  displayName: string;
  models: string[];
  available: boolean;
}

/** Drone info as returned by hive-api /v1/drones. */
export interface DroneInfo {
  id: string;
  provider: string;
  model: string;
  status: string;
}

/** CLI version info as returned by hive-api /v1/cli/versions. */
export interface CliVersionInfo {
  provider: string;
  installedVersion: string | null;
  latestVersion: string | null;
  updateAvailable: boolean;
  cliFound: boolean;
}

export type HiveApiStatus = "disconnected" | "connecting" | "ready" | "error";
