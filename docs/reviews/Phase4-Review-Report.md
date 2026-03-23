# Phase 4 Review Report (Recheck)

**Date:** 2026-03-23
**Reviewer:** Claude Opus 4.6
**Scope:** Re-validation after fixes for all Phase 4 findings

## Build and test status

| Check | Result |
|-------|--------|
| `cargo check` | Clean (0 errors, 0 warnings) |
| `cargo test` | 107 passed, 0 failed |
| `tsc --noEmit` | Clean |
| `vitest run` | 8 passed, 0 failed |
| 300-line file limit | All files within limit (max: scheduler.rs at 292) |

## Previous findings — recheck

### [P1] Handler registry not implemented

Status: **Fixed**

Evidence:
- `stage_runner.rs:41-49` defines `handler_registry()` mapping `chat`, `judge`, `summary`, `skill_select`, `skill_run` to `HandlerKind` variants
- `dispatch_handler` (line 174) looks up `node.handler` in the registry and routes accordingly
- Unknown handlers return a structured failure: `"Unknown node handler '...'."`
- Judge handler (`run_judge_handler`, line 201) extends chat with verdict extraction and logging

Verification: The handler dispatch is now the sole execution path — `run_node` calls `dispatch_handler` (line 169), not `stream_chat_node` directly.

### [P1] Plan gate not wired into graph mode

Status: **Fixed**

Evidence:
- `plan_gate.rs:13-21` reads `config.requires_plan_approval` from `node.config`
- `plan_gate.rs:23-30` reads optional `plan_auto_approve_timeout_sec` for configurable timeout
- `enforce_plan_gate` is called at the top of `run_node` (stage_runner.rs:65) before any execution
- `user_questions.rs:17-55` implements the full question/answer flow via oneshot channels with timeout
- Auto-approve on timeout (line 53) — sensible default to prevent indefinite hangs
- `answer_pipeline_question` command registered in `lib.rs:29`
- `question_answers` propagated through `PipelineRuntimeContext` → `StageRunnerContext`

### [P2] `thread::sleep` in async context

Status: **Fixed**

Evidence:
- `helpers.rs:125-129` now uses `tokio::time::sleep(Duration::from_millis(150)).await`
- `wait_if_paused` is now `async fn` (line 125)
- Caller in `stage_runner.rs:60` properly `.await`s the call

### [P2] No cross-model fallback warning

Status: **Fixed**

Evidence:
- `SessionDecision::New` now carries `warning: Option<String>` (session_manager.rs:16)
- Two warning paths: inbound candidate mismatch (line 50-58) and stored group mismatch (line 68-73)
- `stage_runner.rs:128-138` emits the warning as a `pipeline:log` event when present
- Test `falls_back_to_new_on_provider_model_mismatch` (line 148) verifies warning is `Some`

### [P2] Pipeline canvas missing features

Status: **Fixed**

Evidence:
- **SVG edge rendering:** `PipelineBuilder.tsx:179-188` renders `<line>` elements within an SVG overlay using node centre coordinates
- **Duplicate:** `graph.ts:159-187` implements `duplicateNode` with offset positioning and generated ID
- **Rename:** `graph.ts:141-157` implements `renameNode` with trim and empty-label validation
- **Inline validation:** `PipelineBuilder.tsx:206-208` renders per-node errors; lines 232-234 render per-edge errors
- **Save/Run:** Lines 166-167 add Save and Run buttons; `saveTemplate` (line 121) and `runTemplate` (line 139) are wired to commands
- **Frontend tests:** `graph.test.ts` (259 lines, 8 tests) covers create, delete, move, rename, duplicate, connect, disconnect, and validation

### [P3] Loop edge exception not implemented

Status: **Fixed**

Evidence:
- `TemplateEdge` now has `loop_control: bool` field (templates.rs:119)
- `graph_validation.rs:81-109` uses Kosaraju's SCC algorithm (via `graph_analysis.rs`) to find cycles
- Cycles where all intra-component edges are `loop_control: true` are permitted; otherwise rejected
- `max_iterations <= 1` with cycles is explicitly rejected (line 91-95)
- Graph executor `scheduler.rs:117-119` re-arms loop nodes up to `max_iterations` via `NodePlan.loop_node`
- `topology.rs` builds the component map consumed by the scheduler
- Test `loop_control_cycle_executes_bounded_iterations` (tests.rs:93) verifies nodes `a` and `b` execute exactly 3 times with `max_iterations: 3`
- Test `unresolvable_graph_still_errors` (tests.rs:123) verifies a pure cycle with no entry node still deadlocks

### [P3] Long lines in pipeline.rs

Status: **Fixed**

Evidence:
- `pipeline.rs` reduced from 294 to 210 lines
- Node execution logic extracted to `stage_runner.rs` with dedicated `handle_node_completion` helper (line 249)
- `run_graph_template` (pipeline.rs:140) is now a clean orchestration function

### [P3] No frontend tests for pipeline builder

Status: **Fixed**

Evidence:
- `graph.test.ts` has 8 tests covering: createNode (explicit/generated IDs, duplicate rejection), deleteNode (cascade edges, missing node), moveNode (position update, missing node), renameNode (trim, empty rejection), duplicateNode (offset, config copy), connectNodes (generated IDs, duplicate detection, self-connection rejection), disconnectEdge/disconnectNodes, and validateGraph (blank labels, broken refs, self-connections)

### [P3] User-question flow not wired

Status: **Fixed**

Evidence:
- `user_questions.rs` implements `request_user_approval` with question emission, oneshot channel, and timeout
- `plan_gate.rs` calls `request_user_approval` for nodes requiring plan approval
- `commands/pipeline.rs` now includes `answer_pipeline_question` command (line 166+)
- `lib.rs:29` registers the command in the Tauri handler
- `question_answers` channel map flows from `AppState` → `PipelineRuntimeContext` → `StageRunnerContext`

## New observations

### [P3] `handle_node_completion` still has long single-line event emissions

Status: **Open (cosmetic)**

Evidence:
- `stage_runner.rs:260`, `:263`, `:273`, `:275` contain struct literals exceeding 130 characters

Impact: Readability only. No functional concern.

### [P3] `handler_registry()` allocates a new HashMap on every call

Status: **Open (minor)**

Evidence:
- `stage_runner.rs:41-49` creates a fresh `HashMap` each time `dispatch_handler` is called (once per node execution)

Impact: Negligible for typical graph sizes. Could use `LazyLock` or `OnceLock` if profiling shows it matters.

### [P2] `analyse`/`review`/`implement` stage types map to `chat` handler but were not in the registry

Status: **Fixed**

Evidence:
- `stage_runner.rs:44-49` now includes `analyse`, `review`, `implement`, `test`, and `custom` as backward-compatible aliases mapping to `HandlerKind::Chat`
- Built-in templates continue to use `handler: stage_type.into()` (builtin_templates.rs:189), which is now correctly resolved
- 107 Rust tests pass including `all_builtin_templates_pass_validation`

## Work item coverage summary

| Work item | Status | Notes |
|-----------|--------|-------|
| 1. Graph template schema | Complete | Graph-native schema with `loop_control` on edges |
| 2. Graph validation | Complete | SCC-based cycle handling with bounded loop-control exception |
| 3. Legacy migration | Complete | Legacy `stages` payloads migrate transparently |
| 4. Graph executor | Complete | Concurrent execution + conditional routing + bounded loop re-arming |
| 5. Node handler registry | Complete | Registry-based dispatch in `stage_runner` |
| 6. Prompt rendering | Complete | Graph-aware upstream/edge variable support |
| 7. Session-group continuity | Complete | Compatible resume with mismatch fallback warnings |
| 8. DirectTask mode | Complete | Single-node bypass preserved |
| 9. Diff and artefact capture | Complete | Code-intent nodes emit git diff artefacts |
| 10. Node-based event model | Complete | Node-centric Rust/TypeScript event payloads with `pipeline:question` |
| 11. Pipeline canvas UI | Complete | SVG edges, drag/drop, rename, duplicate, inline validation, save/run |
| 12. Plan gate in graph mode | Complete | Node-config driven approval with timeout and answer channel |

## Architecture quality

**Improvements since initial review:**
- Clean separation: `pipeline.rs` (orchestration) → `stage_runner.rs` (node execution) → `graph_executor/` (topology scheduling)
- `graph_executor` split into `mod.rs` (types), `scheduler.rs` (execution), `topology.rs` (SCC/loop detection), `tests.rs`
- Plan gate and user questions are self-contained modules with single-responsibility
- Frontend `graph.ts` now has 8 thorough tests covering all operations
- `loopControl` is a first-class concept from validation through execution

**Remaining risk:**
- The handler registry mismatch with built-in template handler values (`"analyse"` etc.) will cause runtime failures. This is a **P2 blocker** that should be fixed before any manual testing.

## Verdict

All 8 previous findings are resolved. The implementation now covers all 12 work items from Phase4.md. Build is clean, 107 Rust tests and 8 frontend tests pass, all files are under 300 lines.

All findings are now resolved. Phase 4 is complete against the implementation spec. Two minor cosmetic observations remain open (long lines in struct literals, per-call HashMap allocation) — neither affects correctness or runtime behaviour.
