import type { ReactNode } from "react";
import { useEffect, useMemo, useState } from "react";
import { useAppContext } from "../../contexts/AppContext";
import { useHiveApi } from "../../hooks/useHiveApi";
import { useHiveVersions } from "../../hooks/useHiveVersions";
import { formatTime } from "../../utils/formatTime";
import { CliVersionPanel } from "./CliVersionPanel";
import { DroneInventory } from "./DroneInventory";

interface ErrorLogEntry {
  timestamp: string;
  message: string;
}

function statusBadgeClass(status: string): string {
  if (status === "ready") return "bg-emerald-100 text-emerald-800";
  if (status === "starting") return "bg-blue-100 text-blue-800";
  if (status === "disconnected") return "bg-amber-100 text-amber-800";
  return "bg-red-100 text-red-800";
}

export function HiveApiStatusView(): ReactNode {
  const { settings } = useAppContext();
  const {
    status,
    providers,
    drones,
    error,
    checkHealth,
    waitReady,
    refreshProviders,
    refreshDrones,
    startApi,
    stopApi,
    restartApi,
    isApiRunning,
  } = useHiveApi();
  const {
    versions,
    loading: versionsLoading,
    error: versionError,
    fetchVersions,
    checkVersion,
    updateCli,
  } = useHiveVersions();

  const [entryPath, setEntryPath] = useState(settings.hiveApiEntryPath);
  const [running, setRunning] = useState(false);
  const [lastCheckedAt, setLastCheckedAt] = useState<string | null>(null);
  const [errorLog, setErrorLog] = useState<ErrorLogEntry[]>([]);

  const appendErrorLog = (message: string) => {
    setErrorLog((previous) => [
      {
        timestamp: new Date().toISOString(),
        message,
      },
      ...previous,
    ].slice(0, 20));
  };

  useEffect(() => {
    const refresh = async () => {
      const [health, runningNow] = await Promise.all([
        checkHealth(),
        isApiRunning(),
      ]);
      setRunning(runningNow);
      setLastCheckedAt(new Date().toISOString());
      if (health) {
        await Promise.all([
          refreshProviders(),
          refreshDrones(),
          fetchVersions(),
        ]);
      }
    };

    void refresh();
    const interval = setInterval(() => {
      void refresh();
    }, 15000);

    return () => clearInterval(interval);
  }, [
    checkHealth,
    fetchVersions,
    isApiRunning,
    refreshDrones,
    refreshProviders,
  ]);

  useEffect(() => {
    if (error) {
      appendErrorLog(error);
    }
  }, [error]);

  useEffect(() => {
    if (versionError) {
      appendErrorLog(versionError);
    }
  }, [versionError]);

  const apiParams = useMemo(
    () => ({
      entryPath,
      host: settings.hiveApiHost,
      port: settings.hiveApiPort,
    }),
    [entryPath, settings.hiveApiHost, settings.hiveApiPort],
  );

  return (
    <div className="flex h-full flex-col gap-4 bg-white p-6">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold text-slate-900">hive-api Status</h1>
          <p className="text-sm text-slate-600">
            Health check: {formatTime(lastCheckedAt)}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <span
            className={`rounded px-2 py-1 text-xs font-semibold ${statusBadgeClass(
              status,
            )}`}
          >
            {status}
          </span>
          <span className="rounded bg-slate-100 px-2 py-1 text-xs text-slate-700">
            process {running ? "running" : "stopped"}
          </span>
        </div>
      </header>

      <label className="flex flex-col gap-1 text-sm text-slate-800">
        hive-api entry path
        <input
          className="rounded border border-slate-300 px-3 py-2"
          onChange={(event) => setEntryPath(event.target.value)}
          placeholder="/path/to/hive-api/main.py"
          value={entryPath}
        />
      </label>

      <div className="flex flex-wrap gap-2">
        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          onClick={() => {
            void startApi(apiParams).then((started) => {
              if (started) {
                void waitReady();
              }
            });
          }}
          type="button"
        >
          Start
        </button>
        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          onClick={() => {
            void stopApi();
          }}
          type="button"
        >
          Stop
        </button>
        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          onClick={() => {
            void restartApi(apiParams).then((started) => {
              if (started) {
                void waitReady();
              }
            });
          }}
          type="button"
        >
          Restart
        </button>
        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          onClick={() => {
            void Promise.all([
              checkHealth(),
              refreshProviders(),
              refreshDrones(),
              fetchVersions(),
            ]).then(async () => {
              const runningNow = await isApiRunning();
              setRunning(runningNow);
              setLastCheckedAt(new Date().toISOString());
            });
          }}
          type="button"
        >
          Refresh
        </button>
      </div>

      <section className="rounded border border-slate-200 p-4">
        <h2 className="mb-2 text-sm font-semibold text-slate-800">
          Providers ({providers.length})
        </h2>
        <div className="grid gap-3 md:grid-cols-2">
          {providers.map((provider) => (
            <article className="rounded border border-slate-200 bg-slate-50 p-3" key={provider.name}>
              <div className="mb-1 flex items-center justify-between">
                <strong className="text-sm text-slate-900">{provider.displayName}</strong>
                <span
                  className={`rounded px-2 py-0.5 text-xs ${
                    provider.available
                      ? "bg-emerald-100 text-emerald-800"
                      : "bg-amber-100 text-amber-800"
                  }`}
                >
                  {provider.available ? "available" : "unavailable"}
                </span>
              </div>
              <p className="mb-2 text-xs text-slate-600">{provider.name}</p>
              <div className="flex flex-wrap gap-1">
                {provider.models.map((model) => (
                  <span
                    className="rounded bg-white px-2 py-0.5 text-[11px] text-slate-700"
                    key={model}
                  >
                    {model}
                  </span>
                ))}
              </div>
            </article>
          ))}
        </div>
      </section>

      <DroneInventory drones={drones} />

      <CliVersionPanel
        loading={versionsLoading}
        onCheckVersion={(provider) => {
          void checkVersion(provider);
        }}
        onUpdateCli={(provider) => {
          void updateCli(provider);
        }}
        versions={versions}
      />

      <section className="rounded border border-slate-200 p-4">
        <h2 className="mb-2 text-sm font-semibold text-slate-800">Error Log</h2>
        {errorLog.length === 0 ? (
          <p className="text-xs text-slate-600">No startup or health errors recorded.</p>
        ) : (
          <ul className="space-y-1">
            {errorLog.map((entry, index) => (
              <li className="text-xs text-slate-700" key={`${entry.timestamp}-${index}`}>
                {formatTime(entry.timestamp)} · {entry.message}
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
