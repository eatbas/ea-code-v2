# Phase 2 Review Report (Updated)

**Date:** 2026-03-23  
**Reviewer:** Codex  
**Scope:** Re-validation of Phase 2 fixes (`hive_client`, `commands/hive_api`, frontend hive hooks/types)

## 0. Executive Status

| Item | Status | Notes |
|---|---|---|
| Runtime invoke wiring | Fixed | `invoke.ts` now uses `@tauri-apps/api/core` |
| Frontend/backend hive DTO alignment | Fixed | Health/provider/version shapes now match current Rust models |
| Lifecycle start/stop command gap | Fixed | `start_hive_api`, `stop_hive_api`, `hive_api_process_running` added |
| UTF-8 SSE split-chunk handling | Fixed | `Utf8Buffer` added with tests |
| Version endpoint contract consistency | **Open** | Rust paths differ from documented `/v1/cli-versions*` spec |
| `hive-api:disconnected` crash signal | **Open** | `hive-api:ready` exists, disconnect emit path not found |

## 1. Re-Verification Run

Commands executed:
- `cd frontend/desktop/src-tauri && cargo test`
- `cd frontend/desktop && npx tsc --noEmit`

Results:
- `cargo test`: **PASS** (69/69)
- `npx tsc --noEmit`: **PASS**

## 2. Previously Reported Findings - Recheck

### P1 (previous): Frontend invoke hard-fail stub

**Status:** Fixed  
**Evidence:** [invoke.ts](D:/Github/ea-code-v2/frontend/desktop/src/lib/invoke.ts)

### P1 (previous): Frontend-backend DTO mismatch

**Status:** Fixed for current code contract  
**Evidence:**
- [health.rs](D:/Github/ea-code-v2/frontend/desktop/src-tauri/src/hive_client/health.rs)
- [providers.rs](D:/Github/ea-code-v2/frontend/desktop/src-tauri/src/hive_client/providers.rs)
- [versions.rs](D:/Github/ea-code-v2/frontend/desktop/src-tauri/src/hive_client/versions.rs)
- [hive.ts](D:/Github/ea-code-v2/frontend/desktop/src/types/hive.ts)

### P1 (previous): Missing lifecycle commands

**Status:** Fixed  
**Evidence:**
- [hive_api.rs](D:/Github/ea-code-v2/frontend/desktop/src-tauri/src/commands/hive_api.rs)
- [lib.rs](D:/Github/ea-code-v2/frontend/desktop/src-tauri/src/lib.rs)

### P2 (previous): SSE UTF-8 split-chunk failure risk

**Status:** Fixed  
**Evidence:** [streaming.rs](D:/Github/ea-code-v2/frontend/desktop/src-tauri/src/hive_client/streaming.rs)

## 3. Remaining Findings

### [P2] CLI version endpoint shapes still diverge from repository Phase 2 spec/docs

- **Code currently uses:**
  - `GET /v1/cli/versions`
  - `GET /v1/cli/versions/{provider}`
  - `POST /v1/cli/update/{provider}`
  - See [versions.rs](D:/Github/ea-code-v2/frontend/desktop/src-tauri/src/hive_client/versions.rs)
- **Docs still describe:**
  - `/v1/cli-versions`
  - `/v1/cli-versions/{provider}/check`
  - `/v1/cli-versions/{provider}/update`
  - See [Phase2.md](D:/Github/ea-code-v2/docs/implementation/Phase2.md) and [ea-code-v2.md](D:/Github/ea-code-v2/docs/ea-code-v2.md)

Impact:
- If hive-api follows the documented contract, these client calls can fail with 404.

### [P2] No `hive-api:disconnected` emit path found for runtime crash detection

- `hive-api:ready` is emitted in [hive_api.rs](D:/Github/ea-code-v2/frontend/desktop/src-tauri/src/commands/hive_api.rs)
- No corresponding `hive-api:disconnected` emission path located.

Impact:
- Frontend cannot react to runtime disconnects in the way Phase 2 plan describes.

## 4. Coverage Check (Current)

| Area | Status | Notes |
|---|---|---|
| SSE parser and stream edge cases | Covered | Extensive unit tests, including split UTF-8 chunks |
| Error mapping tests | Covered | `hive_client/error.rs` tests |
| Lifecycle primitive tests | Covered | `hive_client/lifecycle.rs` tests |
| Command-level tests (`commands/hive_api.rs`) | Not covered | No direct tests for command-state behavior |
| Endpoint contract integration tests | Not covered | No mock-server verification of actual path/method contracts |
| Frontend hook tests (`useHiveApi`, `useHiveVersions`) | Not covered | No tests validating runtime command wiring/error transitions |

## 5. Conclusion

Phase 2 implementation quality has improved significantly and most critical blockers are fixed.  
Two contract/runtime-alignment items remain open (version endpoint contract + disconnect event flow). After those are resolved, Phase 2 can be considered fully complete.
