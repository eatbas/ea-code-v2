# Phase 4 - Graph Orchestrator and Visual Pipeline Builder

## Objective

Switch from an ordered stage list to a graph-driven orchestrator where users create stage boxes in the UI via drag-and-drop and wire execution paths manually.

## Scope

- Replace `stages + position + parallel_group` execution with `nodes + edges` execution.
- Add a visual pipeline canvas for node creation, movement, connection, and deletion.
- Execute enabled nodes from graph topology (fan-out and fan-in), not fixed stage order.
- Preserve pause, resume, cancel, user-question, and DirectTask workflows.
- Keep session-group continuity across compatible node transitions.
- Emit dynamic node-based events for timeline and logs.

## Implementation status (2026-03-23)

Build and checks:
- `cargo check` passes.
- `cargo test` passes (`107` passed, `0` failed).
- `npm run typecheck` passes.
- `npm run test` passes (`8` tests).

Work item coverage:

| Work item | Status | Notes |
|-----------|--------|-------|
| 1. Graph template schema | Complete | `nodes`/`edges` + `handler`/`config`/`ui_position` in place |
| 2. Graph validation | Complete | SCC-based validation allows only explicit `loop_control` cycles when `max_iterations > 1` |
| 3. Legacy migration | Complete | Legacy `stages` payloads auto-migrate on load |
| 4. Graph executor | Complete | Fan-out/fan-in wave execution, conditional routing, and bounded loop-control iteration execution implemented |
| 5. Node handler registry | Complete | Handler dispatch registry implemented (`chat`, `judge`, `summary`, `skill_select`, `skill_run`) |
| 6. Prompt rendering | Complete | Graph-aware prompt variables implemented |
| 7. Session-group continuity | Complete | Resume/new logic with mismatch warning emission implemented |
| 8. DirectTask mode | Complete | Direct single-node bypass implemented |
| 9. Diff and artefact capture | Complete | Diff emitted for `execution_intent: "code"` |
| 10. Node-based event model | Complete | Node-centric payloads in Rust and TypeScript |
| 11. Pipeline canvas UI | Complete | Drag/drop nodes, SVG edge rendering, rename/duplicate, inline validation, save/run controls implemented |
| 12. Plan gate in graph mode | Complete | `config.requires_plan_approval` and question/answer flow wired |

Phase 4 closeout:
- All planned work items are implemented.
- Loop-control cycles are now both validated and executed with bounded re-arming tied to `max_iterations`.
- Frontend graph-operation unit tests are in place under `src/features/pipeline-builder/graph.test.ts`.

## Contradictions resolved from previous draft

- Previous draft still used ordered stage semantics (`template.stages`, `position`, `parallel_group`) instead of manual node wiring.
- Previous draft still depended on several built-in stage assumptions (for example, fixed plan-gate and judge placement).
- Previous draft did not define UI behaviour for drag-and-drop stage boxes or connection wiring.

## Work items

### 1. Graph template schema

**Files:**
- `frontend/desktop/src-tauri/src/models/templates.rs` (MODIFY)
- `frontend/desktop/src/types/templates.ts` (MODIFY)

Move template format to graph primitives:

- `nodes: StageNodeDefinition[]`
- `edges: StageEdgeDefinition[]`

`StageNodeDefinition`:
- Keep: `id`, `label`, `prompt_template`, `provider`, `model`, `session_group`, `enabled`, `execution_intent`
- Add: `handler`, `config`, `ui_position`
- Remove: `position`, `parallel_group`

`StageEdgeDefinition`:
- `id`
- `source_node_id`
- `target_node_id`
- `condition` (`always` | `on_success` | `on_failure`)
- `input_key` (optional)

### 2. Graph validation

**File:** `frontend/desktop/src-tauri/src/models/validation.rs` (REWRITE)

Validation rules:
- Unique node IDs and edge IDs.
- Every edge endpoint exists.
- At least one enabled entry node and one enabled terminal node.
- No orphan enabled nodes.
- No cycles unless the loop edge is explicitly marked as loop-control and bounded by `max_iterations`.
- Disabled nodes cannot be required by enabled paths.

Return structured validation errors so UI can highlight invalid boxes/edges.

### 3. Legacy template migration

**Files:**
- `frontend/desktop/src-tauri/src/storage/templates.rs` (MODIFY)
- `frontend/desktop/src-tauri/src/commands/templates.rs` (MODIFY)

Migration strategy:
- Accept legacy `stages` payloads on load.
- Convert each stage to a node and create linear edges based on legacy order.
- Persist only graph format on next save.

### 4. Graph executor

**Files:**
- `frontend/desktop/src-tauri/src/orchestrator/pipeline.rs` (REWRITE)
- `frontend/desktop/src-tauri/src/orchestrator/graph_executor/*` (NEW)

Execution model:
- Build adjacency and indegree maps from enabled nodes/edges.
- Schedule all ready nodes concurrently.
- Start a node only when all required inbound dependencies are complete.
- Evaluate conditional edges (`on_success`, `on_failure`) after node completion.
- For loop-control SCCs, re-arm loop nodes until `max_iterations` is reached.
- Apply cancellation and pause checks before each scheduling cycle and during stream handling.

No hardcoded stage order in the executor path.

### 5. Node handler registry (no hardcoded flow)

**File:** `frontend/desktop/src-tauri/src/orchestrator/stage_runner.rs` (NEW/MODIFY)

- Replace fixed stage-dispatch logic with a handler registry keyed by `node.handler`.
- Built-in handlers can include `chat`, `judge`, `summary`, `skill_select`, `skill_run`.
- Executor treats all nodes uniformly and only calls the selected handler.

This removes hardcoded sequence assumptions while still supporting built-in capabilities.

### 6. Prompt rendering for wired inputs

**File:** `frontend/desktop/src-tauri/src/orchestrator/prompt_renderer.rs` (ADAPT)

Support graph-aware variables:
- `{{upstream_outputs}}`
- `{{upstream_output.<node_id>}}`
- `{{edge_input.<input_key>}}`
- `{{previous_output}}` only when there is exactly one inbound edge

For fan-in nodes, merge inputs deterministically by edge order.

### 7. Session-group continuity on graph transitions

**File:** `frontend/desktop/src-tauri/src/orchestrator/session_manager.rs` (NEW/ADAPT)

Resume policy:
- Resume only when predecessor context matches `session_group + provider + model`.
- On provider/model mismatch (`ShellSessionError`), fall back to `new` mode and continue.
- For multiple candidate parents, use deterministic priority.

### 8. DirectTask mode

**File:** `frontend/desktop/src-tauri/src/orchestrator/pipeline/direct_task.rs` (KEEP/ADAPT)

DirectTask remains a graph bypass:
- Single `POST /v1/chat`
- Standard run persistence and events
- No node traversal

### 9. Diff and artefact capture

**File:** `frontend/desktop/src-tauri/src/orchestrator/helpers.rs` (MODIFY)

After any node with `execution_intent: "code"`:
- Capture workspace diff
- Emit `pipeline:artifact` with `node_id`
- Persist artefact in run events

### 10. Node-based event model

**Files:**
- `frontend/desktop/src-tauri/src/models/events.rs` (MODIFY)
- `frontend/desktop/src/types/events.ts` (MODIFY)

Replace stage-centric event payloads with node-centric payloads:
- `node_id`
- `node_label`
- `status`
- `output`
- optional `edge_id` for transition diagnostics

### 11. Pipeline canvas UI (drag-and-drop + wiring)

**Files:**
- `frontend/desktop/src/types/pipeline.ts` (MODIFY)
- `frontend/desktop/src/lib/invoke.ts` (MODIFY)
- `frontend/desktop/src/hooks/usePipelineTemplates.ts` (MODIFY)
- `frontend/desktop/src/features/pipeline-builder/*` (NEW)

UI requirements:
- Add, duplicate, delete, and rename node boxes.
- Drag nodes and persist `ui_position`.
- Draw, reconnect, and delete edges.
- Show validation errors inline on nodes/edges.
- Save and run directly from the graph editor.

### 12. Plan gate and user questions in graph mode

**Files:**
- `frontend/desktop/src-tauri/src/orchestrator/plan_gate.rs` (ADAPT)
- `frontend/desktop/src-tauri/src/orchestrator/user_questions.rs` (KEEP)

- Plan gate becomes node-config driven (`config.requires_plan_approval`) rather than stage-name driven.
- Mid-run user-question flow remains unchanged.

## File paths summary

| Action | Path |
|--------|------|
| REWRITE | `frontend/desktop/src-tauri/src/orchestrator/pipeline.rs` |
| NEW | `frontend/desktop/src-tauri/src/orchestrator/graph_executor/*` |
| NEW/ADAPT | `frontend/desktop/src-tauri/src/orchestrator/session_manager.rs` |
| NEW/MODIFY | `frontend/desktop/src-tauri/src/orchestrator/stage_runner.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/prompt_renderer.rs` |
| MODIFY | `frontend/desktop/src-tauri/src/models/templates.rs` |
| NEW | `frontend/desktop/src-tauri/src/models/graph_analysis.rs` |
| MODIFY | `frontend/desktop/src/types/templates.ts` |
| REWRITE | `frontend/desktop/src-tauri/src/models/validation.rs` |
| MODIFY | `frontend/desktop/src-tauri/src/storage/templates.rs` |
| MODIFY | `frontend/desktop/src-tauri/src/commands/templates.rs` |
| MODIFY | `frontend/desktop/src-tauri/src/commands/pipeline.rs` |
| MODIFY | `frontend/desktop/src-tauri/src/models/events.rs` |
| MODIFY | `frontend/desktop/src/types/events.ts` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/helpers.rs` |
| KEEP/ADAPT | `frontend/desktop/src-tauri/src/orchestrator/pipeline/direct_task.rs` |
| NEW | `frontend/desktop/src/features/pipeline-builder/*` |

## Testing

- **Graph round-trip:** Create 5 nodes and custom edges in UI, save/reload, verify structure and `ui_position` persist.
- **Cycle rejection:** Build `A -> B -> A`; verify save is blocked with clear error.
- **Fan-out:** One node wired to two child nodes; verify concurrent run.
- **Fan-in:** Two parent nodes wired into one node; verify child starts after both complete.
- **Conditional edges:** Judge/output node routes correctly on success/failure conditions.
- **Session resume:** Verify resume only on compatible `session_group + provider + model`.
- **Cross-model fallback:** Same session group with different model/provider falls back to new session with warning.
- **Diff artefacts:** Code-intent node emits diff artefact tied to `node_id`.
- **Pause/resume/cancel:** Verify correct behaviour during concurrent branches.
- **Legacy migration:** Load a legacy stage template, auto-migrate to graph, run successfully.

## Deliverables

- Graph-native orchestration driven by user-defined nodes and edges.
- Visual pipeline builder with drag-and-drop nodes and manual wiring.
- No hardcoded stage-order assumptions in execution path.
- Node-based event stream for timeline/log rendering.
- Backwards-compatible migration path from legacy templates.

## Dependencies

- Phase 3 template storage and prompt rendering baseline.
- Phase 2 `hive_client` for all agent calls.
- Frontend pipeline editor implementation for node/edge creation.

## Risks and mitigations

- Risk: Invalid wiring causes runtime dead-ends.
  Mitigation: strict pre-save and pre-run graph validation with actionable errors.
- Risk: Fan-in prompts become ambiguous.
  Mitigation: deterministic merge order plus optional `input_key` routing.
- Risk: Concurrent graph branches complicate cancellation semantics.
  Mitigation: cancellation checks before each scheduling cycle and during stream processing.

## Exit criteria

- User-created node/edge templates execute without backend code edits.
- No static stage order or enum-only flow assumptions remain in orchestrator logic.
- Fan-out, fan-in, and conditional routing pass end-to-end tests.
- Pause/resume/cancel remains stable under concurrent execution.
- Run summaries include node/edge metadata and transition history.
- DirectTask remains functional end-to-end.

## Estimated duration

2 weeks
