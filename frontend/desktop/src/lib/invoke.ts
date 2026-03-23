import { invoke as tauriInvoke } from "@tauri-apps/api/core";

/**
 * Typed wrapper around Tauri's invoke.
 * All IPC calls go through this single entry point.
 */
export async function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  return tauriInvoke<T>(cmd, args);
}
