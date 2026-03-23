import { useState, useCallback } from "react";
import type { CliVersionInfo } from "../types";
import { invoke } from "../lib/invoke";

export function useHiveVersions() {
  const [versions, setVersions] = useState<CliVersionInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchVersions = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const v = await invoke<CliVersionInfo[]>("hive_api_cli_versions");
      setVersions(v);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const checkVersion = useCallback(async (provider: string) => {
    try {
      setError(null);
      const info = await invoke<CliVersionInfo>("hive_api_check_cli_version", {
        provider,
      });
      setVersions((prev) =>
        prev.map((v) => (v.provider === provider ? info : v)),
      );
      return info;
    } catch (e) {
      setError(String(e));
      return null;
    }
  }, []);

  const updateCli = useCallback(
    async (provider: string) => {
      try {
        setError(null);
        const result = await invoke<string>("hive_api_update_cli", {
          provider,
        });
        await checkVersion(provider);
        return result;
      } catch (e) {
        setError(String(e));
        return null;
      }
    },
    [checkVersion],
  );

  return {
    versions,
    loading,
    error,
    fetchVersions,
    checkVersion,
    updateCli,
  };
}
