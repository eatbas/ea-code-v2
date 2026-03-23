# Phase 4 - Dynamic Orchestrator and Session Groups

## Objective

Switch from hardcoded stage flow to template-driven execution with session continuity across stage groups.

## Scope

- Rewrite orchestrator loop to execute enabled stages from template order.
- Implement session group state and resume mode selection.
- Preserve pause, resume, cancel, question, and DirectTask workflows.
- Support parallel stage execution within templates.
- Integrate skill selection, diff capture, judge, and context summary into the dynamic pipeline.

## Work items

### 1. Dynamic stage dispatch

**File: `frontend/desktop/src-tauri/src/orchestrator/pipeline.rs` (REWRITE)**

Replace fixed stage dispatch with template-driven execution:

```rust
pub async fn run_pipeline(
    template: &PipelineTemplate,
    prompt: &str,
    workspace_path: &str,
    app_handle: &AppHandle,
    cancel_flag: Arc<AtomicBool>,
    pause_flag: Arc<AtomicBool>,
    // ... answer channels
) -> Result<(), String> {
    let mut session_refs: HashMap<String, String> = HashMap::new();
    let enabled_stages: Vec<&StageDefinition> = template.stages
        .iter()
        .filter(|s| s.enabled)
        .collect();

    for iteration in 1..=template.max_iterations {
        let mut previous_output = String::new();

        for stage in &enabled_stages {
            // Check cancel/pause between every stage
            check_cancel_pause(&cancel_flag, &pause_flag).await?;

            // 1. Render prompt template with variables
            let rendered = render_prompt(&stage.prompt_template, &context);

            // 2. Determine resume mode
            let (mode, session_ref) = resolve_session_mode(stage, &session_refs);

            // 3. Call hive-api via hive_client
            let (output, new_ref) = run_hive_chat(...).await?;

            // 4. Store session ref
            if let Some(ref_id) = new_ref {
                session_refs.insert(stage.session_group.clone(), ref_id);
            }

            // 5. Capture diff if execution_intent == "code"
            if stage.execution_intent == "code" {
                capture_workspace_diff(workspace_path, app_handle, run_id).await;
            }

            previous_output = output;
        }

        // Iteration termination check (judge logic)
        if should_terminate(template, iteration, &previous_output)? {
            break;
        }
    }
}
```

### 2. Session group manager

**File: `frontend/desktop/src-tauri/src/orchestrator/session_manager.rs` (NEW)**

Tracks `provider_session_ref` values per session group per run.

**Session group decision rules:**

| Condition | Mode | Action |
|-----------|------|--------|
| Same `session_group` + same `provider` + same `model` as prior stage in group | `resume` | Pass stored `provider_session_ref` |
| Same `session_group` but different `provider` or `model` | `new` | Cannot resume across providers. Start new session, overwrite group ref. |
| Different `session_group` | `new` | Start fresh session. Previous stage output passed as `{{previous_output}}` in prompt text. |
| First stage in a group (no stored ref) | `new` | No ref exists yet. |

**Cross-model resume rejection:** If hive-api returns a `ShellSessionError` (model mismatch), fall back to `new` mode, log warning, and retry the stage. Do not fail the run.

**Persistence:** `session_refs` map is stored in `RunSummary` so it can be inspected in history.

### 3. DirectTask mode

**File: `frontend/desktop/src-tauri/src/orchestrator/pipeline/direct_task.rs` (KEEP, adapt)**

DirectTask is the single-agent bypass mode (user checks "Direct Task" on the prompt bar). In v2:
- DirectTask skips the template pipeline loop entirely.
- Sends a single `POST /v1/chat` with the user prompt directly.
- Uses the default provider/model from template's first stage (or app settings).
- Still creates a run entry, emits events, and stores results.
- No iteration, no session groups, no judge.

### 4. Parallel stage execution

**Stage types that support parallel execution:**

The v1 orchestrator runs up to 3 planners and 3 reviewers in parallel via `tokio::join!()`. In v2, this is driven by template configuration:

- Stages at the same `position` value execute in parallel.
- Or: adjacent stages with a shared `parallel_group` marker run concurrently.

**Recommended approach:** Add an optional `parallel_group: Option<String>` field to `StageDefinition`. Stages with the same `parallel_group` value execute concurrently via `tokio::join!()`. Output is concatenated (or merged by a subsequent merge stage).

Example in a template:
```
Position 0: Analyse (sequential)
Position 1: Review A (parallel_group: "reviewers")
Position 2: Review B (parallel_group: "reviewers")  â† runs in parallel with Review A
Position 3: Implement (sequential, receives merged review output)
```

**File: `frontend/desktop/src-tauri/src/orchestrator/parallel.rs` (ADAPT from `parallel_stage.rs`)**

Reuse the existing parallel execution runner, adapted for dynamic `StageDefinition` lists.

### 5. Diff capture stages

v1 has `DiffAfterCoder` and `DiffAfterCodeFixer` as explicit pipeline stages. In v2, diff capture is **automatic** based on `execution_intent`:

- After any stage with `execution_intent: "code"` completes, the orchestrator runs `git diff` on the workspace.
- The diff is emitted as a `pipeline:artifact` event (type: `diff`).
- The diff is stored in the run events log.
- No explicit "DiffAfterCoder" stage needed in the template â€” it happens automatically.

**File: `frontend/desktop/src-tauri/src/orchestrator/helpers.rs` (MODIFY)** â€” Add `capture_workspace_diff()` helper using existing `git.rs` functions.

### 6. Judge integration and iteration termination

v1 has a dedicated Judge agent stage. In v2, iteration termination is configurable per template:

**Option A (recommended): Built-in judge logic**
- After the last stage of each iteration, the orchestrator checks if the pipeline should loop.
- For templates with `stop_on_first_pass: true`: if the last stage's output contains no failure indicators, stop.
- For templates with `stop_on_first_pass: false`: always run `max_iterations`.
- The judge prompt is a hardcoded system prompt that evaluates the final stage output.

**Option B: Judge as an optional template stage**
- Users can add a "Judge" stage to their template with `stage_type: "judge"`.
- The orchestrator recognises this stage type and uses its output to decide COMPLETE vs NOT_COMPLETE.
- If no judge stage exists, use Option A behaviour.

**Recommended: Combine both.** The orchestrator always runs a lightweight built-in judge check after the last stage. If a user adds an explicit judge stage, its verdict takes precedence.

**Judge verdict parsing:** Reuse `parsing/plan.rs` verdict extraction. Judge output must contain `COMPLETE` or `NOT_COMPLETE`.

**`{{judge_feedback}}` variable:** On iteration > 1, this contains the judge's reasoning from the previous iteration, injected into stage prompts via the template variable system.

### 7. Skill selection and skill stage

v1 runs `SkillSelect` after prompt enhancement to identify applicable user-defined skills. In v2:

- Skill selection remains a built-in orchestrator step (not a template stage).
- Before executing the first stage of each iteration, the orchestrator runs skill selection.
- Selected skills' content is injected as `{{code_context}}` or appended to the first stage's prompt.
- Skill execution (if a skill requires running an agent) uses hive-api like any other stage.

**Files preserved and adapted:**
- `orchestrator/skill_selection.rs` â€” Adapted to use hive_client instead of CLI dispatch.
- `orchestrator/skill_stage.rs` â€” Adapted to use hive_client for skill agent execution.

### 8. Context summary and executive summary

- **Context summary** (`context_summary.rs`): Generates a brief summary of what happened in the current iteration. Adapted to use hive_client. Called after the last stage of each iteration. Summary stored in run events.
- **Executive summary**: An optional final stage the user can add to any template with `stage_type: "summary"`. The orchestrator recognises this type and runs it once after the final iteration completes (not on every iteration).

### 9. Session memory

**File: `frontend/desktop/src-tauri/src/orchestrator/session_memory.rs` (ADAPT)**

v1's session memory builds context from prior runs in the same session. In v2:
- Same concept: on new runs within an existing session, previous run summaries are injected as context.
- With session groups and resume, less context injection is needed (the resumed session already has memory).
- Session memory is injected into the first stage's rendered prompt as additional context.

### 10. Plan gate and user questions

**File: `frontend/desktop/src-tauri/src/orchestrator/plan_gate.rs` (ADAPT)**

Plan approval gate remains functional:
- If `require_plan_approval` is true in settings, the orchestrator pauses after any stage with `stage_type: "analyse"` or `stage_type: "review"` that produces a plan-like output.
- User approves or rejects via the existing question flow.
- If `plan_auto_approve_timeout_sec` elapses, auto-approve.

**File: `frontend/desktop/src-tauri/src/orchestrator/user_questions.rs` (KEEP)**

Mid-run question handling is unchanged. Works with dynamic stages identically to v1.

### 11. SSE â†’ pipeline event translation

The orchestrator's stage runner translates hive-api SSE events to Tauri pipeline events:

```
hive-api SSE event        â†’  Tauri event
â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€        â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
run_started               â†’  pipeline:stage { stage_id, label, status: "running" }
output_delta              â†’  pipeline:log { text, stage_id }
completed                 â†’  pipeline:stage { stage_id, label, status: "completed", output }
failed                    â†’  pipeline:error { stage_id, error }
stopped                   â†’  pipeline:stage { stage_id, label, status: "cancelled" }
provider_session          â†’  (internal: stored in session_refs, not emitted)
```

Stage labels in events come from `StageDefinition.label` (dynamic, not hardcoded enum values). This means:
- `RunEvent` variants use `String` stage identifiers instead of `PipelineStage` enum.
- Frontend timeline renders stage names from events, not from a static list.
- Backwards compatibility: v1 events still use the old enum names. Frontend detects format version.

## File paths summary

| Action | Path |
|--------|------|
| REWRITE | `frontend/desktop/src-tauri/src/orchestrator/pipeline.rs` (or `pipeline/mod.rs`) |
| NEW | `frontend/desktop/src-tauri/src/orchestrator/session_manager.rs` |
| NEW | `frontend/desktop/src-tauri/src/orchestrator/stage_runner.rs` |
| NEW | `frontend/desktop/src-tauri/src/orchestrator/prompt_renderer.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/pipeline/direct_task.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/parallel.rs` (from `parallel_stage.rs`) |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/helpers.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/skill_selection.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/skill_stage.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/context_summary.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/session_memory.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/plan_gate.rs` |
| KEEP | `frontend/desktop/src-tauri/src/orchestrator/user_questions.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/run_setup/mod.rs` |
| ADAPT | `frontend/desktop/src-tauri/src/orchestrator/run_setup/persistence.rs` |
| MODIFY | `frontend/desktop/src-tauri/src/models/events.rs` (String stage IDs) |
| MODIFY | `frontend/desktop/src-tauri/src/commands/pipeline.rs` (pass template to orchestrator) |

## Testing

- **Multi-template execution:** Run at least 3 different templates (Full Review Loop, Quick Fix, Research Only) end-to-end. Verify stage order, event sequence, and output correctness.
- **Session resume verification:** Run Full Review Loop. Verify that stages in group "A" (Analyse â†’ Review) use `mode: "resume"` on the second stage. Verify group "B" (Implement â†’ Test) starts a new session.
- **Cross-model rejection:** Configure a template where two stages in the same group use different models. Verify graceful fallback to `new` mode with a warning.
- **DirectTask:** Verify single-agent mode bypasses template loop, produces correct events.
- **Cancellation between dynamic stages:** Cancel mid-pipeline. Verify the current hive-api job is stopped and the run is marked cancelled.
- **Pause/resume:** Pause between two stages. Resume and verify the pipeline continues from the correct stage.
- **Parallel stages:** Configure two stages with the same `parallel_group`. Verify they execute concurrently.
- **Iteration loop:** Run a 3-iteration template. Verify `{{iteration_number}}` and `{{judge_feedback}}` are injected correctly.
- **Diff capture:** Run a stage with `execution_intent: "code"`. Verify `pipeline:artifact` event with diff is emitted.

## Deliverables

- Fully dynamic backend orchestration path driven by `PipelineTemplate`.
- Session groups demonstrably resume context across stage boundaries.
- DirectTask mode functional via hive-api.
- Parallel stage execution supported.
- Stable event stream compatible with updated frontend timeline.

## Dependencies

- Phase 3 template storage, prompt renderer, and built-in templates.
- Phase 2 hive_client module for all agent invocations.

## Risks and mitigations

- Risk: Complex templates break cancellation semantics.
  Mitigation: Add cancellation checks between all dynamic stage transitions and during SSE handling. Test with adversarial templates (many stages, nested groups).
- Risk: Session resume fails silently for some providers.
  Mitigation: Log all resume attempts. On `ShellSessionError`, fall back to `new` mode and emit a user-visible warning.
- Risk: Parallel stages produce conflicting outputs.
  Mitigation: Concatenate parallel outputs with clear separators. Let subsequent stages or users interpret.

## Exit criteria

- At least 3 different templates execute successfully without backend code changes.
- Pause/resume/cancel works in multi-iteration runs.
- Session resume verified with at least 2 providers (Claude + one other).
- Run summaries include template id, stage sequence, and session group metadata.
- DirectTask mode works end-to-end.
- No regressions in existing v1 history display.

## Estimated duration

1.5 weeks



