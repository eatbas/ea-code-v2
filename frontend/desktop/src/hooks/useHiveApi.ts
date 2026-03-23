import { useState, useCallback } from "react";
import type {
  HealthResponse,
  ProviderInfo,
  DroneInfo,
  HiveApiStatus,
} from "../types";
import { invoke } from "../lib/invoke";

export function useHiveApi() {
  const [status, setStatus] = useState<HiveApiStatus>("disconnected");
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [drones, setDrones] = useState<DroneInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  const initClient = useCallback(async (host: string, port: number) => {
    try {
      setStatus("connecting");
      await invoke("init_hive_client", { host, port });
    } catch (e) {
      setStatus("error");
      setError(String(e));
    }
  }, []);

  const checkHealth = useCallback(async (): Promise<HealthResponse | null> => {
    try {
      const health = await invoke<HealthResponse>("hive_api_status");
      if (health.dronesBooted) {
        setStatus("ready");
      }
      return health;
    } catch (e) {
      setStatus("error");
      setError(String(e));
      return null;
    }
  }, []);

  const waitReady = useCallback(async (): Promise<HealthResponse | null> => {
    try {
      setStatus("connecting");
      const health = await invoke<HealthResponse>("hive_api_wait_ready");
      setStatus("ready");
      return health;
    } catch (e) {
      setStatus("error");
      setError(String(e));
      return null;
    }
  }, []);

  const refreshProviders = useCallback(async () => {
    try {
      const p = await invoke<ProviderInfo[]>("hive_api_providers");
      setProviders(p);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const refreshDrones = useCallback(async () => {
    try {
      const d = await invoke<DroneInfo[]>("hive_api_drones");
      setDrones(d);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  return {
    status,
    providers,
    drones,
    error,
    initClient,
    checkHealth,
    waitReady,
    refreshProviders,
    refreshDrones,
  };
}
