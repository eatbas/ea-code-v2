# EA Code v2 — Non-Functional SLOs

These service-level objectives define the performance and reliability targets for EA Code v2. They are internal targets, not contractual guarantees.

## Stage Execution

| Metric | Target |
|--------|--------|
| Stage start latency | < 500 ms from orchestrator dispatch to first SSE event received by the frontend |
| Event delivery | Zero dropped `output_delta` events under normal operation |
| Stage timeout | Configurable per stage; default 5 minutes. Orchestrator cancels and marks `failed` on timeout. |

## hive-api

| Metric | Target |
|--------|--------|
| Boot time | < 30 s from process spawn to `/health` returning `drones_booted: true` |
| Health poll interval | 60 s during idle, 2 s during startup sequence |
| Request latency overhead | < 100 ms added by hive-api proxy layer above raw provider latency |

## Storage

| Metric | Target |
|--------|--------|
| Atomic write safety | No data loss on app crash during a write operation. Achieved via `.tmp` -> `.bak` -> final rename pattern. |
| History view load time | < 2 s for sessions with 1000+ events |
| Index rebuild time | < 5 s for a full `index.json` rebuild from 500 projects |

## Recovery

| Metric | Target |
|--------|--------|
| Interrupted run cleanup | Runs interrupted by crash are marked `failed` with a synthetic `RunEnd` event on next startup |
| Settings migration | Idempotent. Backup written before any migration. Re-running migration on already-migrated data is a no-op. |
| Corrupt file recovery | Atomic write `.bak` files are used as fallback. If both primary and `.bak` are corrupt, the file is reset to defaults and logged. |

## UI Responsiveness

| Metric | Target |
|--------|--------|
| View switch | < 100 ms to render the new view after user navigation |
| Pipeline event processing | Frontend processes incoming events within one animation frame (16 ms) |
| Settings save round-trip | < 200 ms from user action to confirmation |
