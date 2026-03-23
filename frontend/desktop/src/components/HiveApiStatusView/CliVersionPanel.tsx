import type { ReactNode } from "react";
import type { CliVersionInfo } from "../../types";

interface CliVersionPanelProps {
  versions: CliVersionInfo[];
  loading: boolean;
  onCheckVersion: (provider: string) => void;
  onUpdateCli: (provider: string) => void;
}

export function CliVersionPanel({
  versions,
  loading,
  onCheckVersion,
  onUpdateCli,
}: CliVersionPanelProps): ReactNode {
  return (
    <section className="rounded border border-slate-200 p-4">
      <h2 className="mb-2 text-sm font-semibold text-slate-800">
        CLI Versions
      </h2>
      {loading ? (
        <p className="text-xs text-slate-600">Loading version information...</p>
      ) : (
        <div className="space-y-2">
          {versions.map((version) => (
            <div
              className="flex flex-wrap items-center justify-between gap-2 rounded bg-slate-50 p-2"
              key={version.provider}
            >
              <p className="text-sm text-slate-700">
                {version.provider}: {version.installedVersion ?? "n/a"}
                {version.updateAvailable ? " (update available)" : ""}
              </p>
              <div className="flex gap-2">
                <button
                  className="rounded border border-slate-300 px-2 py-1 text-xs"
                  onClick={() => onCheckVersion(version.provider)}
                  type="button"
                >
                  Check
                </button>
                <button
                  className="rounded border border-slate-300 px-2 py-1 text-xs"
                  disabled={!version.updateAvailable}
                  onClick={() => onUpdateCli(version.provider)}
                  type="button"
                >
                  Update
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
