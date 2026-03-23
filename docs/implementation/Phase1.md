# Phase 1 - Foundation and Contracts ✅ COMPLETED

## Objective

Establish the v2 baseline architecture, contracts, and development scaffolding before feature implementation.

## Scope

- Finalise shared domain model for pipelines, stages, templates, and session groups.
- Define API contract boundaries between Tauri backend, frontend, and hive-api.
- Decide hive-api bundling and distribution strategy.
- Prepare repository structure and feature flags for progressive rollout.

## Completion summary

All work items delivered. `cargo check`, `cargo test` (15 tests), and `npx tsc --noEmit` pass cleanly.

## Work items

### 1. Architecture decision records ✅

Created 5 ADRs in `docs/adr/`:

| ADR | File | Status |
|-----|------|--------|
| Dynamic pipeline execution | `docs/adr/001-dynamic-pipeline-execution.md` | Approved |
| Session group semantics | `docs/adr/002-session-group-semantics.md` | Approved |
| hive-api bundling strategy | `docs/adr/003-hive-api-bundling.md` | **Pending** — evaluates 3 options, recommends sidecar |
| Feature flags | `docs/adr/004-feature-flags.md` | Approved |
| Canonical template variables | `docs/adr/005-canonical-template-variables.md` | Approved — locks 11 variable names |

### 2. Rust contract types ✅

Cargo project scaffolded at `frontend/desktop/src-tauri/`. All model types created:

| File | Types | Tests |
|------|-------|-------|
| `src/models/templates.rs` | `PipelineTemplate`, `StageDefinition` | 2 (round-trip, camelCase keys) |
| `src/models/settings.rs` | `AppSettings` + `Default` impl | 3 (defaults, round-trip, camelCase) |
| `src/models/storage.rs` | `ProjectEntry`, `SessionMeta`, `RunSummary`, `GitBaseline`, `ChatMessage` | 3 (v1 compat, v2 fields, camelCase) |
| `src/models/events.rs` | `StageEndStatus`, `RunStatus`, `RunEvent` (10 variants) | 4 (tagged enum, all variants, snake_case status, nested struct) |
| `src/models/pipeline.rs` | `PipelineStatus`, `StageStatus`, `JudgeVerdict` | 3 (snake_case, round-trip, verdict) |

**Bug found and fixed during testing:** serde's `rename_all` on internally-tagged enums does not rename variant fields. `RunEvent` was emitting `stage_id` instead of `stageId`. Fixed with explicit per-field `#[serde(rename)]` annotations to ensure camelCase field names while keeping snake_case type tags. This would have broken frontend integration.

**Serde conventions enforced:**
- Structs: `#[serde(rename_all = "camelCase")]` — field names are camelCase for frontend
- Simple enums (`RunStatus`, `PipelineStatus`, etc.): `#[serde(rename_all = "snake_case")]` — values are snake_case
- Tagged enum (`RunEvent`): `#[serde(tag = "type")]` with explicit variant renames for snake_case tags + explicit field renames for camelCase fields

### 3. TypeScript contract types ✅

TypeScript project scaffolded at `frontend/desktop/`. All type files created:

| File | Exports |
|------|---------|
| `src/types/templates.ts` | `PipelineTemplate`, `StageDefinition`, `CreateTemplateRequest`, `UpdateTemplateRequest`, `CloneTemplateRequest` |
| `src/types/settings.ts` | `AppSettings`, `DEFAULT_SETTINGS` |
| `src/types/pipeline.ts` | `PipelineStatus`, `StageStatus`, `JudgeVerdict` |
| `src/types/events.ts` | `PIPELINE_EVENTS`, `RunStatus`, `StageEndStatus`, `RunEvent` (discriminated union) |
| `src/types/history.ts` | `ProjectEntry`, `SessionMeta`, `SessionDetail`, `RunDetail`, `GitBaseline` |
| `src/types/storage.ts` | `ChatMessage`, `RunSummaryFile` |
| `src/types/navigation.ts` | `ActiveView` union |
| `src/types/index.ts` | Barrel re-exports |

### 3.1 Canonical template variable names ✅

Locked in ADR-005. 11 variables, no aliases:
`{{task}}`, `{{workspace_path}}`, `{{file_list}}`, `{{code_context}}`, `{{previous_output}}`, `{{iteration_number}}`, `{{max_iterations}}`, `{{test_results}}`, `{{judge_feedback}}`, `{{git_branch}}`, `{{git_diff}}`

### 4. Compatibility layer ✅

- `RunSummary` v2 fields (`pipeline_template_id`, `pipeline_template_name`) are `Option<>`.
- `session_refs` uses `#[serde(default)]` — deserialises to empty `HashMap` when absent.
- Tested: v1 JSON without v2 fields deserialises cleanly (test: `run_summary_v1_compat_missing_v2_fields`).

### 5. Integration test harness ✅

15 inline `#[cfg(test)]` tests across all model modules. Coverage:
- Serde round-trip for all types
- camelCase serialisation verification
- snake_case enum serialisation
- Tagged enum type field correctness
- v1 backward compatibility
- Default settings values
- Nested struct serialisation in events

### 6. Non-functional SLOs ✅

Documented in `docs/slos.md`:
- Stage start latency: < 500 ms
- hive-api boot time: < 30 s
- Zero dropped events under normal operation
- History view load: < 2 s for 1000+ events
- Crash recovery: interrupted runs marked failed on next startup

### 7. Built-in template schemas ✅

Documented in `docs/built-in-templates.md`. All 5 templates with full stage tables:
1. Full Review Loop (4 stages, 5 iterations, groups A+B)
2. Quick Fix (2 stages, 1 iteration, group A)
3. Research Only (2 stages, 1 iteration, group A)
4. Multi-Brain Review (5 stages, 3 iterations, groups A-D)
5. Security Audit (5 stages, 2 iterations, groups A+B)

## Files created

### Rust backend
| File | Purpose |
|------|---------|
| `frontend/desktop/src-tauri/Cargo.toml` | Package config with serde, tauri, tokio, reqwest, uuid, chrono |
| `frontend/desktop/src-tauri/build.rs` | Tauri build script |
| `frontend/desktop/src-tauri/tauri.conf.json` | Tauri v2 app configuration |
| `frontend/desktop/src-tauri/src/main.rs` | Entry point |
| `frontend/desktop/src-tauri/src/lib.rs` | Library root (`pub mod models`) |
| `frontend/desktop/src-tauri/src/models/mod.rs` | Module index |
| `frontend/desktop/src-tauri/src/models/templates.rs` | PipelineTemplate, StageDefinition |
| `frontend/desktop/src-tauri/src/models/settings.rs` | AppSettings + Default |
| `frontend/desktop/src-tauri/src/models/storage.rs` | RunSummary, ProjectEntry, SessionMeta, etc. |
| `frontend/desktop/src-tauri/src/models/events.rs` | RunEvent, RunStatus, StageEndStatus |
| `frontend/desktop/src-tauri/src/models/pipeline.rs` | PipelineStatus, StageStatus, JudgeVerdict |

### TypeScript frontend
| File | Purpose |
|------|---------|
| `frontend/desktop/package.json` | Package setup (TypeScript 5.8) |
| `frontend/desktop/tsconfig.json` | Strict TS config |
| `frontend/desktop/src/types/templates.ts` | Template and stage types + CRUD request types |
| `frontend/desktop/src/types/settings.ts` | AppSettings + DEFAULT_SETTINGS |
| `frontend/desktop/src/types/pipeline.ts` | Status enums |
| `frontend/desktop/src/types/events.ts` | PIPELINE_EVENTS, RunEvent union |
| `frontend/desktop/src/types/history.ts` | Project, session, run types |
| `frontend/desktop/src/types/storage.ts` | ChatMessage, RunSummaryFile |
| `frontend/desktop/src/types/navigation.ts` | ActiveView union |
| `frontend/desktop/src/types/index.ts` | Barrel re-exports |

### Documentation
| File | Purpose |
|------|---------|
| `docs/adr/001-dynamic-pipeline-execution.md` | Pipeline template architecture |
| `docs/adr/002-session-group-semantics.md` | Session resume rules |
| `docs/adr/003-hive-api-bundling.md` | Bundling strategy (pending decision) |
| `docs/adr/004-feature-flags.md` | v1→v2 rollout via settings_version |
| `docs/adr/005-canonical-template-variables.md` | 11 locked variable names |
| `docs/slos.md` | Non-functional SLOs |
| `docs/built-in-templates.md` | 5 built-in template definitions |

## Verification results

| Check | Result |
|-------|--------|
| `cargo check` | ✅ Passes |
| `cargo test` | ✅ 15 passed, 0 failed |
| `npx tsc --noEmit` | ✅ Zero errors |
| All files < 300 lines | ✅ |

## Exit criteria — all met

- ✅ All core v2 domain types exist in Rust and TypeScript and compile cleanly.
- ✅ hive-api bundling strategy documented (decision pending, options evaluated in ADR-003).
- ✅ Built-in template schemas agreed and documented.
- ✅ Serde conventions verified by tests (camelCase fields, snake_case enums, tagged enum correctness).
- ✅ v1 backward compatibility verified by tests.
