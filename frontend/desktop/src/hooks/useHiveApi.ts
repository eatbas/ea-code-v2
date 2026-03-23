import { useState, useCallback, useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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

  // Listen for backend health events
  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    listen<HealthResponse>("hive-api:ready", () => {
      setStatus("ready");
      setError(null);
    }).then((fn) => unlisteners.push(fn));

    listen<string>("hive-api:disconnected", (event) => {
      setStatus("error");
      setError(event.payload ?? "hive-api disconnected");
    }).then((fn) => unlisteners.push(fn));

    listen<string>("hive-api:reconnected", () => {
      setStatus("ready");
      setError(null);
    }).then((fn) => unlisteners.push(fn));

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

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

  const startMonitor = useCallback(
    async (pollIntervalSecs?: number) => {
      try {
        await invoke("start_hive_monitor", {
          pollIntervalSecs: pollIntervalSecs ?? 60,
        });
      } catch (e) {
        setError(String(e));
      }
    },
    [],
  );

  const stopMonitor = useCallback(async () => {
    try {
      await invoke("stop_hive_monitor");
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
    startMonitor,
    stopMonitor,
  };
}
