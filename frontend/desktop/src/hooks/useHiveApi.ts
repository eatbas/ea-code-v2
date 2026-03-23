import { useState, useCallback, useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  DroneInfo,
  HealthResponse,
  HiveApiStatus,
  ProviderInfo,
} from "../types";
import { invoke } from "../lib/invoke";
import { toErrorMessage } from "../utils/toErrorMessage";

interface StartHiveApiParams {
  entryPath: string;
  host: string;
  port: number;
}

export function useHiveApi() {
  const [status, setStatus] = useState<HiveApiStatus>("disconnected");
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [drones, setDrones] = useState<DroneInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Listen for backend health events
  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];
    let active = true;

    const register = async () => {
      try {
        const ready = await listen<HealthResponse>("hive-api:ready", () => {
          setStatus("ready");
          setError(null);
        });
        if (active) unlisteners.push(ready);

        const disconnected = await listen<string>(
          "hive-api:disconnected",
          (event) => {
            setStatus("disconnected");
            setError(event.payload ?? "hive-api disconnected");
          },
        );
        if (active) unlisteners.push(disconnected);

        const reconnected = await listen<string>("hive-api:reconnected", () => {
          setStatus("ready");
          setError(null);
        });
        if (active) unlisteners.push(reconnected);
      } catch (eventError) {
        setStatus("error");
        setError(toErrorMessage(eventError));
      }
    };

    void register();

    return () => {
      active = false;
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, []);

  const initClient = useCallback(async (host: string, port: number) => {
    try {
      setStatus("starting");
      setError(null);
      await invoke("init_hive_client", { host, port });
    } catch (e) {
      setStatus("error");
      setError(toErrorMessage(e));
    }
  }, []);

  const checkHealth = useCallback(async (): Promise<HealthResponse | null> => {
    try {
      setError(null);
      const health = await invoke<HealthResponse>("hive_api_status");
      if (health.dronesBooted) {
        setStatus("ready");
      } else {
        setStatus("starting");
      }
      return health;
    } catch (e) {
      setStatus("disconnected");
      setError(toErrorMessage(e));
      return null;
    }
  }, []);

  const waitReady = useCallback(async (): Promise<HealthResponse | null> => {
    try {
      setStatus("starting");
      setError(null);
      const health = await invoke<HealthResponse>("hive_api_wait_ready");
      setStatus("ready");
      return health;
    } catch (e) {
      setStatus("disconnected");
      setError(toErrorMessage(e));
      return null;
    }
  }, []);

  const refreshProviders = useCallback(async () => {
    try {
      const p = await invoke<ProviderInfo[]>("hive_api_providers");
      setProviders(p);
    } catch (e) {
      setError(toErrorMessage(e));
    }
  }, []);

  const refreshDrones = useCallback(async () => {
    try {
      const d = await invoke<DroneInfo[]>("hive_api_drones");
      setDrones(d);
    } catch (e) {
      setError(toErrorMessage(e));
    }
  }, []);

  const startApi = useCallback(
    async ({ entryPath, host, port }: StartHiveApiParams): Promise<boolean> => {
      try {
        setStatus("starting");
        setError(null);
        await invoke<void>("start_hive_api", {
          entryPath,
          host,
          port,
        });
        return true;
      } catch (e) {
        setStatus("error");
        setError(toErrorMessage(e));
        return false;
      }
    },
    [],
  );

  const stopApi = useCallback(async (): Promise<boolean> => {
    try {
      setError(null);
      await invoke<void>("stop_hive_api");
      setStatus("disconnected");
      return true;
    } catch (e) {
      setStatus("error");
      setError(toErrorMessage(e));
      return false;
    }
  }, []);

  const restartApi = useCallback(
    async (params: StartHiveApiParams): Promise<boolean> => {
      const stopResult = await stopApi();
      if (!stopResult) {
        return false;
      }
      return startApi(params);
    },
    [startApi, stopApi],
  );

  const isApiRunning = useCallback(async (): Promise<boolean> => {
    try {
      setError(null);
      return await invoke<boolean>("hive_api_process_running");
    } catch (e) {
      setStatus("error");
      setError(toErrorMessage(e));
      return false;
    }
  }, []);

  const startMonitor = useCallback(
    async (pollIntervalSecs?: number) => {
      try {
        setError(null);
        await invoke("start_hive_monitor", {
          pollIntervalSecs: pollIntervalSecs ?? 60,
        });
      } catch (e) {
        setError(toErrorMessage(e));
      }
    },
    [],
  );

  const stopMonitor = useCallback(async () => {
    try {
      setError(null);
      await invoke("stop_hive_monitor");
    } catch (e) {
      setError(toErrorMessage(e));
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
    startApi,
    stopApi,
    restartApi,
    isApiRunning,
    startMonitor,
    stopMonitor,
  };
}
