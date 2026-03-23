# Phase 4 Review Report

**Date:** 2026-03-23
**Reviewer:** Claude Opus 4.6
**Scope:** Graph Orchestrator and Visual Pipeline Builder (Phase4.md)
**Status:** Completed and closed

## Review history

| Round | Date | Outcome |
|-------|------|---------|
| Initial review | 2026-03-23 | 8 findings (2 P1, 4 P2, 2 P3) |
| First recheck | 2026-03-23 | All 8 original findings fixed; 1 new P2 found (handler registry mismatch) |
| Second recheck | 2026-03-23 | All findings resolved |

## Build and test status

| Check | Result |
|-------|--------|
| `cargo check` | Clean (0 errors, 0 warnings) |
| `cargo test` | 107 passed, 0 failed |
| `tsc --noEmit` | Clean |
| `vitest run` | 8 passed, 0 failed |
| 300-line file limit | All files within limit (max: scheduler.rs at 292) |

## What changed across the three review rounds

### Round 1 findings and how they were resolved

#### [P1] Handler registry not implemented (Work Item 5)

**Problem:** All nodes routed through `stream_chat_node` regardless of `node.handler` value. No dispatch table existed.

**Resolution:** New `stage_runner.rs` module with:
- `HandlerKind` enum (`Chat`, `Judge`, `Summary`, `SkillSelect`, `SkillRun`)
- `handler_registry()` function mapping handler strings to enum variants
- `dispatch_handler()` performing the lookup and routing
- Dedicated `run_judge_handler()` that extends chat with verdict extraction and logging
- `run_node` now calls `dispatch_handler` as its sole execution path

#### [P1] Plan gate not wired into graph mode (Work Item 12)

**Problem:** No `config.requires_plan_approval` check existed anywhere in the orchestrator.

**Resolution:** Two new modules:
- `plan_gate.rs` — reads `requires_plan_approval` and `plan_auto_approve_timeout_sec` from `node.config`, calls `request_user_approval`
- `user_questions.rs` — emits `pipeline:question` event, registers a oneshot channel, waits with timeout, auto-approves on timeout
- `enforce_plan_gate` is called at the top of `run_node` before any execution begins
- `answer_pipeline_question` Tauri command registered in `lib.rs` to deliver answers from the frontend
- `question_answers` channel map propagated through `PipelineRuntimeContext` → `StageRunnerContext`

#### [P2] `thread::sleep` blocks Tokio runtime

**Problem:** `wait_if_paused` used `std::thread::sleep` inside an async context, risking thread pool starvation under concurrent fan-out.

**Resolution:** `helpers.rs` — `wait_if_paused` is now `async fn` using `tokio::time::sleep(Duration::from_millis(150)).await`. Callers `.await` it correctly.

#### [P2] No cross-model fallback warning emitted

**Problem:** `SessionManager::decide_mode` silently returned `New` on provider/model mismatch with no user-visible feedback.

**Resolution:** `SessionDecision::New` now carries `warning: Option<String>`. Two warning paths: inbound candidate mismatch and stored group mismatch. `stage_runner.rs` emits the warning as a `pipeline:log` event. Test verifies warning is `Some`.

#### [P2] Pipeline canvas missing SVG edges, duplicate, rename, validation display, save/run

**Problem:** Edges listed as text only. No duplicate/rename. No inline validation. No save/run buttons.

**Resolution:**
- SVG `<line>` elements drawn within canvas using node centre coordinates
- `duplicateNode()` and `renameNode()` added to `graph.ts` with proper validation
- `validateGraph()` returns per-node and per-edge error maps
- `PipelineBuilder.tsx` renders errors inline on nodes and edges
- Save and Run buttons wired to `updateTemplate`/`createTemplate` and `startPipelineRun`

#### [P3] Loop edge exception not implemented

**Problem:** All cycles rejected unconditionally. No support for bounded loop-control edges.

**Resolution:**
- `TemplateEdge` gained `loop_control: bool` field
- `graph_analysis.rs` implements Kosaraju's SCC algorithm
- `graph_validation.rs` uses SCC to detect cycles; permits cycles where all intra-component edges are `loop_control: true` and `max_iterations > 1`
- `graph_executor/scheduler.rs` re-arms loop nodes up to `max_iterations` via `NodePlan`
- `graph_executor/topology.rs` builds the SCC component map for the scheduler
- Tests verify bounded iteration (3 iterations) and deadlock detection for pure cycles

#### [P3] Long lines in pipeline.rs

**Problem:** Multiple lines exceeded 150 characters; `pipeline.rs` was 294 lines with mixed concerns.

**Resolution:** Node execution logic extracted to `stage_runner.rs` with dedicated `handle_node_completion` helper. `pipeline.rs` reduced from 294 to 210 lines.

#### [P3] No frontend tests for pipeline builder

**Problem:** `graph.ts` operations were untested.

**Resolution:** `graph.test.ts` (259 lines, 8 tests) covers: createNode, deleteNode, moveNode, renameNode, duplicateNode, connectNodes, disconnectEdge/disconnectNodes, and validateGraph.

#### [P3] User-question flow not wired into run_node

**Problem:** `question_answers` oneshot channels existed in `AppState` but were never used during node execution.

**Resolution:** Wired through `plan_gate.rs` → `user_questions.rs`. `answer_pipeline_question` command delivers answers. Channel map flows `AppState` → `PipelineRuntimeContext` → `StageRunnerContext`.

### Round 2 finding and resolution

#### [P2] Built-in template handler values not in registry

**Problem:** Built-in templates set `handler: stage_type.into()`, producing values like `"analyse"`, `"review"`, `"implement"`, `"test"`. The handler registry only contained `"chat"`, `"judge"`, `"summary"`, `"skill_select"`, `"skill_run"`. This would cause all built-in templates to fail at runtime.

**Resolution:** Added backward-compatible aliases to the handler registry in `stage_runner.rs:44-49`: `"analyse"`, `"review"`, `"implement"`, `"test"`, `"custom"` all map to `HandlerKind::Chat`.

## Work item coverage summary

| Work item | Status | Key files |
|-----------|--------|-----------|
| 1. Graph template schema | Complete | `models/templates.rs`, `types/templates.ts` |
| 2. Graph validation | Complete | `models/graph_validation.rs`, `models/graph_analysis.rs` |
| 3. Legacy migration | Complete | `models/templates.rs` (custom `Deserialize`) |
| 4. Graph executor | Complete | `orchestrator/graph_executor/{mod,scheduler,topology,tests}.rs` |
| 5. Node handler registry | Complete | `orchestrator/stage_runner.rs` |
| 6. Prompt rendering | Complete | `orchestrator/prompt_renderer.rs` |
| 7. Session-group continuity | Complete | `orchestrator/session_manager.rs` |
| 8. DirectTask mode | Complete | `orchestrator/pipeline.rs` |
| 9. Diff and artefact capture | Complete | `orchestrator/stage_runner.rs`, `orchestrator/helpers.rs` |
| 10. Node-based event model | Complete | `models/events.rs`, `types/events.ts` |
| 11. Pipeline canvas UI | Complete | `features/pipeline-builder/{PipelineBuilder,graph,model}.ts(x)` |
| 12. Plan gate in graph mode | Complete | `orchestrator/plan_gate.rs`, `orchestrator/user_questions.rs` |

## Architecture summary

```
commands/pipeline.rs          Tauri command entry points (start/pause/resume/cancel/answer)
    |
orchestrator/pipeline.rs      Run orchestration (direct task vs graph template)
    |
orchestrator/stage_runner.rs  Per-node execution: plan gate → handler dispatch → completion
    |                          handling (events, diffs, session memory)
    |
    +-- orchestrator/plan_gate.rs        Config-driven approval gate
    +-- orchestrator/user_questions.rs   Question/answer channel with timeout
    +-- orchestrator/session_manager.rs  Session resume/fallback decisions with warnings
    +-- orchestrator/prompt_renderer.rs  Graph-aware template variable rendering
    +-- orchestrator/helpers.rs          Event emission, git diff capture, chat streaming
    |
orchestrator/graph_executor/
    +-- mod.rs                 Types (NodeOutcome, NodeExecutionResult, etc.)
    +-- scheduler.rs           Wave-based topological execution with loop re-arming
    +-- topology.rs            SCC component map for loop detection
    +-- tests.rs               DAG, bounded loop, and deadlock tests
```

## Remaining cosmetic observations (not blocking)

- `stage_runner.rs:260,263,273,275` — struct literal lines exceed 130 characters
- `handler_registry()` allocates a new `HashMap` per call — negligible cost, could use `LazyLock` if profiled

## Verdict

Phase 4 is complete. All 12 work items from Phase4.md are implemented. All 9 findings across three review rounds are resolved. 107 Rust tests and 8 frontend tests pass. Build is clean with no errors or warnings. All files are under the 300-line limit.
