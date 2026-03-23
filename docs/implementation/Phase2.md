# Phase 2 - hive-api Integration and Runtime Control ✅ COMPLETED

## Objective

Replace direct CLI process spawning with a stable hive-api client path and runtime management.

## Completion summary

All work items delivered. `cargo check` passes, `cargo test` passes (69 tests — 42 hive_client + 27 models), `npx tsc --noEmit` passes.

## Scope

- Implement backend `hive_client` module for HTTP and SSE streaming.
- Add hive-api lifecycle management (start, health polling, shutdown).
- Add commands for hive-api health visibility.
- Map hive-api SSE events to Tauri pipeline events.
- Preserve current pipeline log streaming behaviour in the UI.

## Work items

### 1. hive-api lifecycle management

**File: `frontend/desktop/src-tauri/src/commands/hive_api.rs` (NEW)**

Tauri manages hive-api as a child process:

1. **Start:** On app launch (if `auto_start_hive_api` is true), spawn `uvicorn hive_api.main:app --host 127.0.0.1 --port <port>`.
2. **Health wait:** Poll `GET /health` every 2 s until response includes `drones_booted: true` (timeout: 30 s per Phase 1 SLO).
3. **Readiness:** Emit `hive-api:ready` Tauri event once healthy. Frontend shows loading state until this fires.
4. **Shutdown:** On app close, send SIGTERM to child process. hive-api's lifespan handler cleans up drones gracefully.
5. **Crash recovery:** If health check fails during a run, emit `hive-api:disconnected` and pause the pipeline.

**Environment variables passed to hive-api child process:**

| Variable | Purpose | Default |
|----------|---------|---------|
| `HIVE_API_CONFIG` | Path to `config.toml` | `~/.ea-code/hive-api/config.toml` |
| `HIVE_API_HOST` | Bind address | `127.0.0.1` |
| `HIVE_API_PORT` | Port | `8000` |

### 2. hive_client Rust module

**File: `frontend/desktop/src-tauri/src/hive_client/mod.rs` (NEW)** â€” `HiveClient` struct with `reqwest::Client`, base URL, connection state.

**File: `frontend/desktop/src-tauri/src/hive_client/chat.rs` (NEW)** â€” Core agent invocation:

```rust
pub struct ChatRequest {
    pub provider: String,
    pub model: String,
    pub workspace_path: String,
    pub mode: String,              // "new" or "resume"
    pub prompt: String,
    pub stream: bool,
    pub provider_session_ref: Option<String>,
    pub provider_options: HashMap<String, Value>,
}

/// Sends a chat request to hive-api and streams SSE events back.
/// Emits pipeline:log events for each output_delta.
/// Returns (final_text, provider_session_ref).
pub async fn run_hive_chat(
    client: &reqwest::Client,
    base_url: &str,
    request: ChatRequest,
    app_handle: &AppHandle,
    run_id: &str,
    stage_label: &str,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(String, Option<String>), String>
```

**File: `frontend/desktop/src-tauri/src/hive_client/health.rs` (NEW)** â€” `GET /health`, readiness polling loop.

**File: `frontend/desktop/src-tauri/src/hive_client/providers.rs` (NEW)** â€” `GET /v1/providers`, `GET /v1/models`, `GET /v1/drones`.

**File: `frontend/desktop/src-tauri/src/hive_client/versions.rs` (NEW)** — CLI version management endpoint proxies (`GET /v1/cli/versions`, `GET /v1/cli/versions/{provider}`, `POST /v1/cli/update/{provider}`).

### 3. SSE event type mapping

hive-api emits 7 SSE event types. The `hive_client/chat.rs` module consumes these and translates them to Tauri `pipeline:*` events:

| hive-api SSE event | Payload | Tauri event | Action |
|--------------------|---------|-------------|--------|
| `run_started` | `{ provider, model, job_id }` | `pipeline:stage` (status: running) | Store `job_id` for cancellation |
| `provider_session` | `{ provider_session_ref }` | (internal) | Store ref in `session_refs` map |
| `output_delta` | `{ text }` | `pipeline:log` | Append to running output, emit to frontend |
| `completed` | `{ final_text, exit_code, session_ref, warnings }` | `pipeline:stage` (status: completed) | Capture final output, advance to next stage |
| `failed` | `{ error, exit_code, warnings }` | `pipeline:error` | Mark stage failed, propagate error |
| `stopped` | `{ provider, model, job_id }` | `pipeline:stage` (status: cancelled) | User-initiated cancel acknowledged |

**Note:** `provider_session` is consumed internally by the orchestrator (stored in session_refs map) and not forwarded to the frontend.

### 4. HTTP error mapping

| HTTP status | hive-api meaning | Frontend display |
|-------------|------------------|------------------|
| 400 | Provider CLI not installed (`"Provider 'X' is not available"`) | Toast: "Provider X is not available. Check hive-api status." |
| 404 | No drone for provider/model pair | Toast: "No drone available for X/model. Check provider configuration." |
| 500 | CLI crash or internal error | Toast: "Agent execution failed. Check logs." |
| Connection refused | hive-api not running | Toast: "hive-api is not running. Starting..." + auto-start attempt |

### 5. Job lifecycle and cancellation

**Job states:** `QUEUED â†’ RUNNING â†’ COMPLETED | FAILED | STOPPED`

Cancellation bridge:
1. Capture `job_id` from `run_started` SSE event.
2. On cancel command: `POST /v1/chat/{job_id}/stop`.
3. hive-api sends Ctrl-C (`\x03\n`) to the drone's bash stdin.
4. `stopped` SSE event closes the stream.
5. Orchestrator marks stage as cancelled and run as cancelled.

Queued jobs (waiting for a drone) are removed immediately on cancel.

### 6. MCP config passthrough

v1's `agents/mcp.rs` built temporary MCP config files for each CLI invocation. In v2:
- MCP server configurations remain stored in `~/.ea-code/mcp.json`.
- When building a `ChatRequest`, include MCP server configs in `provider_options` as a serialised config block.
- hive-api passes MCP config to drones via their initialisation environment.
- The `commands/mcp.rs` CRUD commands remain unchanged.

### 7. Replace legacy CLI hooks in frontend

- Replace `useCliHealth` hook with hive-api health status (consumed from `hive-api:ready` / `hive-api:disconnected` events).
- Replace `useCliVersions` hook with proxy calls through `hive_client/versions.rs`.
- `CliSetupView` becomes a hive-api status view (full replacement deferred to Phase 5).

### 8. Retry and backoff

- HTTP requests to hive-api use exponential backoff: 1 s, 2 s, 4 s (max 3 retries).
- SSE stream reconnection: if stream drops mid-stage, retry once. If second attempt fails, mark stage as failed.
- Health poll during idle: every 60 s background ping to detect hive-api crashes early.

## File paths summary

| Action | Path |
|--------|------|
| NEW | `frontend/desktop/src-tauri/src/hive_client/mod.rs` |
| NEW | `frontend/desktop/src-tauri/src/hive_client/chat.rs` |
| NEW | `frontend/desktop/src-tauri/src/hive_client/health.rs` |
| NEW | `frontend/desktop/src-tauri/src/hive_client/providers.rs` |
| NEW | `frontend/desktop/src-tauri/src/hive_client/versions.rs` |
| NEW | `frontend/desktop/src-tauri/src/commands/hive_api.rs` |
| MODIFY | `frontend/desktop/src-tauri/src/commands/mod.rs` (add hive-api commands to AppState) |
| MODIFY | `frontend/desktop/src-tauri/src/lib.rs` (register new commands, init HiveClient) |
| MODIFY | `frontend/desktop/src/hooks/useCliHealth.ts` (replace internals or delete) |
| MODIFY | `frontend/desktop/src/hooks/useCliVersions.ts` (replace internals or delete) |

## Testing

- **SSE parser tests:** Capture real hive-api stream output as fixtures (completed, failed, stopped scenarios). Parse and verify event sequence.
- **Health polling tests:** Mock `/health` endpoint returning various states (booting, ready, error). Verify readiness detection and timeout.
- **Cancellation test:** Verify `POST /v1/chat/{job_id}/stop` is called when cancel_flag is set, and `stopped` event is handled.
- **Error mapping test:** Verify HTTP 400/404/500 responses produce correct user-facing error strings.

## Deliverables

- End-to-end stage execution through hive-api with streamed output.
- hive-api lifecycle managed by Tauri (start, health wait, shutdown).
- Cancel action stops remote job and updates run status correctly.
- UI status indicator for hive-api health.

## Dependencies

- Phase 1 contracts approved.
- hive-api available in development environment (bundling decision from Phase 1).
- `reqwest` crate with SSE/streaming support in `Cargo.toml`.

## Risks and mitigations

- Risk: SSE parsing edge cases under long outputs.
  Mitigation: Add parser tests using captured stream fixtures for completed, failed, and stopped events.
- Risk: hive-api startup too slow on lower-spec machines.
  Mitigation: Lazy drone booting (only boot providers that are actually configured in templates). Show progress in UI.
- Risk: MCP config passthrough breaks provider-specific behaviour.
  Mitigation: Test each provider with MCP servers enabled/disabled. Fall back to no-MCP if provider rejects config.

## Exit criteria

- No remaining runtime usage of legacy `agents/*` execution path for new runs.
- Stage logs and final outputs match expected event order.
- Health status is visible and actionable in the desktop UI.
- Cancellation works for both queued and running jobs.
- hive-api starts and stops cleanly with the app.

## Estimated duration

1 to 1.5 weeks

---

## Implementation results

### Files created — Rust backend

| File | Purpose | Tests |
|------|---------|-------|
| `src-tauri/src/hive_client/mod.rs` | `HiveClient` struct (reqwest client + base URL) | — |
| `src-tauri/src/hive_client/error.rs` | HTTP error → user-facing message mapping | 9 |
| `src-tauri/src/hive_client/health.rs` | `check_health()`, `wait_until_ready()`, camelCase serde | 3 |
| `src-tauri/src/hive_client/chat.rs` | `HiveSseEvent` enum (6 variants) | — |
| `src-tauri/src/hive_client/sse.rs` | SSE line parser + event parsing + full sequence tests | 16 |
| `src-tauri/src/hive_client/streaming.rs` | `ChatRequest`, `ChatResult`, `Utf8Buffer`, `chat_stream()`, `cancel_job()` | 7 |
| `src-tauri/src/hive_client/providers.rs` | `ProviderInfo`, `DroneInfo`, `/v1/providers`, `/v1/drones` | 2 |
| `src-tauri/src/hive_client/versions.rs` | `CliVersionInfo`, `/v1/cli/versions`, `/v1/cli/update` | 2 |
| `src-tauri/src/hive_client/lifecycle.rs` | `HiveProcess` — spawn/stop/is_running for hive-api child process | 3 |
| `src-tauri/src/commands/mod.rs` | `AppState` (cancel/pause flags, question channels, hive client, process, monitor) | — |
| `src-tauri/src/commands/hive_api.rs` | 11 Tauri commands for hive-api lifecycle + query | — |
| `src-tauri/src/commands/hive_monitor.rs` | Background health monitor, emits `hive-api:disconnected` / `hive-api:reconnected` | — |

### Files created — TypeScript frontend

| File | Purpose |
|------|---------|
| `src/lib/invoke.ts` | Typed Tauri invoke wrapper (wired to `@tauri-apps/api/core`) |
| `src/types/hive.ts` | `HealthResponse`, `ProviderInfo`, `DroneInfo`, `CliVersionInfo`, `HiveApiStatus` |
| `src/hooks/useHiveApi.ts` | Connection management hook (init, health, providers, drones) |
| `src/hooks/useHiveVersions.ts` | CLI version management hook (fetch, check, update) |

### Files modified

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | Added `pub mod hive_client;`, `pub mod commands;`, wired AppState + 13 commands |
| `src-tauri/Cargo.toml` | Added `futures-util = "0.3"` |
| `src/types/index.ts` | Added hive type re-exports |
| `package.json` | Added `react` ^19.0.0, `@types/react` ^19.0.0, `@tauri-apps/api` |

### Tauri commands registered

| Command | Purpose |
|---------|---------|
| `init_hive_client` | Initialise HTTP client with host/port |
| `hive_api_status` | Check health and drone readiness |
| `hive_api_wait_ready` | Poll until ready, emit `hive-api:ready` event |
| `hive_api_providers` | List available providers |
| `hive_api_drones` | List active drones |
| `hive_api_cli_versions` | Get all provider CLI versions |
| `hive_api_check_cli_version` | Check single provider version |
| `hive_api_update_cli` | Trigger CLI update for provider |
| `start_hive_api` | Spawn hive-api child process + init HTTP client |
| `stop_hive_api` | Kill hive-api child process |
| `hive_api_process_running` | Check if child process is alive |
| `start_hive_monitor` | Start background health poll (emits `hive-api:disconnected` / `hive-api:reconnected`) |
| `stop_hive_monitor` | Stop background health poll |

### Post-review fixes applied

| Finding | Fix |
|---------|-----|
| `invoke.ts` always throws | Installed `@tauri-apps/api`, wired real `tauriInvoke` |
| DTO mismatches (HealthResponse, ProviderInfo, DroneInfo, CliVersionInfo) | Added `#[serde(rename_all = "camelCase")]` to HealthResponse, aligned all TS types to match Rust serialisation |
| Endpoint paths inconsistent | All endpoints use `/v1/` prefix except `/health` |
| UTF-8 split across SSE chunks | Added `Utf8Buffer` with incomplete-sequence buffering + 3 tests |
| No process lifecycle commands | Added `lifecycle.rs` + 3 Tauri commands (`start_hive_api`, `stop_hive_api`, `hive_api_process_running`) |
| No `hive-api:disconnected` emission | Added `hive_monitor.rs` with background health poll that emits `hive-api:disconnected` / `hive-api:reconnected` |
| CLI version endpoint doc mismatch | Updated Phase2.md to match code paths (`/v1/cli/versions`, `/v1/cli/update/{provider}`) |

### Verification

| Check | Result |
|-------|--------|
| `cargo check` | ✅ Passes |
| `cargo test` | ✅ 69 passed, 0 failed |
| `npx tsc --noEmit` | ✅ Zero errors |
| All files < 300 lines | ✅ |

