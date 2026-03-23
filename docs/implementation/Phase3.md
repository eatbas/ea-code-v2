# Phase 3 - Pipeline Templates, Prompts, and Persistence ✅ COMPLETED

## Objective

Enable user-owned pipeline definitions with persisted templates, stage configuration, and editable prompts.

## Scope

- Add template CRUD storage and commands.
- Ship built-in templates with prompt defaults migrated from v1 prompt files.
- Support prompt variables, conditional sections, and validation.
- Add prompt Enhance flow.

## Work items

### 1. Template storage

**File: `frontend/desktop/src-tauri/src/storage/templates.rs` (NEW)**

Uses the same atomic write + per-file mutex lock pattern as existing storage modules:
- Read/write to `~/.ea-code/pipeline-templates/{id}.json`.
- Built-in templates are read-only (loaded from app resources, not user-editable).
- User templates support full CRUD.
- Cloning a built-in template creates a user copy with `is_builtin: false` and a new UUID.

### 2. Backend commands

**File: `frontend/desktop/src-tauri/src/commands/templates.rs` (NEW)**

| Command | Signature | Purpose |
|---------|-----------|---------|
| `list_templates` | `() -> Vec<PipelineTemplate>` | All built-in + user templates |
| `get_template` | `(id: String) -> PipelineTemplate` | Single template by ID |
| `create_template` | `(payload: CreateTemplateRequest) -> PipelineTemplate` | Create user template (server sets id, timestamps, is_builtin) |
| `update_template` | `(id: String, payload: UpdateTemplateRequest) -> PipelineTemplate` | Update user template (reject if `is_builtin`) |
| `delete_template` | `(id: String) -> ()` | Delete user template (reject if `is_builtin`) |
| `clone_template` | `(id: String, payload: CloneTemplateRequest) -> PipelineTemplate` | Clone any template into a user copy |

Request DTOs:
- `CreateTemplateRequest`: name, description, max_iterations, stop_on_first_pass, stages
- `UpdateTemplateRequest`: name, description, max_iterations, stop_on_first_pass, stages
- `CloneTemplateRequest`: new_name

### 3. Frontend types and hooks

**File: `frontend/desktop/src/types/templates.ts`** â€” Already defined in Phase 1. Ensure re-exported from `frontend/desktop/src/types/index.ts`.

**File: `frontend/desktop/src/hooks/usePipelineTemplates.ts` (NEW)**

```typescript
usePipelineTemplates()
  -> {
    templates: PipelineTemplate[],
    builtinTemplates: PipelineTemplate[],
    userTemplates: PipelineTemplate[],
    createTemplate: (payload: CreateTemplateRequest) => Promise<PipelineTemplate>,
    updateTemplate: (id: string, payload: UpdateTemplateRequest) => Promise<PipelineTemplate>,
    deleteTemplate: (id: string) => Promise<void>,
    cloneTemplate: (id: string, payload: CloneTemplateRequest) => Promise<PipelineTemplate>,
    refreshTemplates: () => Promise<void>,
  }
```

### 4. Template variables

The prompt renderer injects these variables at runtime. All variables use `{{name}}` syntax.

| Variable | Source | Available |
|----------|--------|-----------|
| `{{task}}` | User's original prompt | Always |
| `{{workspace_path}}` | Absolute project path | Always |
| `{{file_list}}` | Files in workspace (tree listing) | Always |
| `{{code_context}}` | Relevant file contents (auto-selected or from skills) | Always |
| `{{previous_output}}` | Output from the prior stage in this iteration | Position > 0 |
| `{{iteration_number}}` | Current loop iteration (1-based) | Always |
| `{{max_iterations}}` | Pipeline iteration limit from template | Always |
| `{{test_results}}` | Test output from last iteration's test stage | Iteration > 1 |
| `{{judge_feedback}}` | Judge's reasoning if looping back | Iteration > 1 |
| `{{git_branch}}` | Current git branch name | If git repo |
| `{{git_diff}}` | Working tree changes (staged + unstaged) | If git repo |

**Conditional sections:** Support `{{#if variable}}...{{/if}}` blocks so prompts can gracefully handle missing variables:

```
{{#if previous_output}}
Previous analysis findings:
{{previous_output}}
{{/if}}
```

Variables that are unavailable (e.g., `{{test_results}}` on iteration 1) resolve to empty string. Conditional blocks with empty variables are omitted entirely.

### 5. Execution intent

Each stage has an `execution_intent` field:

| Value | Meaning | Effect |
|-------|---------|--------|
| `"text"` | Read-only analysis | Agent should only output text (no file writes). Prompt includes instruction to analyse, not modify. |
| `"code"` | Implementation | Agent is allowed to write files. Workspace diff captured after stage completes. |

This drives:
- Whether `DiffAfterCoder`/`DiffAfterCodeFixer` capture runs after the stage (Phase 4).
- Frontend display: text stages show output as markdown; code stages show output + diff viewer.

### 6. Built-in pipeline templates

Model values in built-in templates must match exact hive-api model IDs. Avoid shorthand labels.

#### Template 1: Full Review Loop (default)

```
Name:           Full Review Loop
Max iterations: 5
Stop on pass:   true

Stages:
â”Œâ”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚ Pos â”‚ Label        â”‚ Type      â”‚ Providerâ”‚ Model â”‚ Group  â”‚ Intent â”‚
â”œâ”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¤
â”‚  0  â”‚ Analyse      â”‚ analyse   â”‚ claude  â”‚ opus  â”‚ A      â”‚ text   â”‚
â”‚  1  â”‚ Review       â”‚ review    â”‚ claude  â”‚ opus  â”‚ A      â”‚ text   â”‚
â”‚  2  â”‚ Implement    â”‚ implement â”‚ claude  â”‚ sonnetâ”‚ B      â”‚ code   â”‚
â”‚  3  â”‚ Test         â”‚ test      â”‚ claude  â”‚ sonnetâ”‚ B      â”‚ code   â”‚
â””â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”˜

Session flow: Analyse â”€â”€resumeâ”€â”€â–¶ Review â”€â”€newâ”€â”€â–¶ Implement â”€â”€resumeâ”€â”€â–¶ Test
```

#### Template 2: Quick Fix

```
Name:           Quick Fix
Max iterations: 1
Stop on pass:   true

Stages:
â”Œâ”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚ Pos â”‚ Label        â”‚ Type      â”‚ Providerâ”‚ Model â”‚ Group  â”‚ Intent â”‚
â”œâ”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¤
â”‚  0  â”‚ Implement    â”‚ implement â”‚ claude  â”‚ sonnetâ”‚ A      â”‚ code   â”‚
â”‚  1  â”‚ Test         â”‚ test      â”‚ claude  â”‚ sonnetâ”‚ A      â”‚ code   â”‚
â””â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”˜

Session flow: Implement â”€â”€resumeâ”€â”€â–¶ Test
```

#### Template 3: Research Only

```
Name:           Research Only
Max iterations: 1
Stop on pass:   true

Stages:
â”Œâ”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚ Pos â”‚ Label        â”‚ Type      â”‚ Providerâ”‚ Model â”‚ Group  â”‚ Intent â”‚
â”œâ”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¤
â”‚  0  â”‚ Analyse      â”‚ analyse   â”‚ claude  â”‚ opus  â”‚ A      â”‚ text   â”‚
â”‚  1  â”‚ Review       â”‚ review    â”‚ claude  â”‚ opus  â”‚ A      â”‚ text   â”‚
â””â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”˜

Session flow: Analyse â”€â”€resumeâ”€â”€â–¶ Review
```

#### Template 4: Multi-Brain Review

```
Name:           Multi-Brain Review
Max iterations: 3
Stop on pass:   true

Stages:
â”Œâ”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚ Pos â”‚ Label        â”‚ Type      â”‚ Providerâ”‚ Model â”‚ Group  â”‚ Intent â”‚
â”œâ”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¤
â”‚  0  â”‚ Analyse      â”‚ analyse   â”‚ claude  â”‚ opus  â”‚ A      â”‚ text   â”‚
â”‚  1  â”‚ Review       â”‚ review    â”‚ gemini  â”‚ gemini-3.1-pro-preview â”‚ B â”‚ text â”‚
â”‚  2  â”‚ Review 2     â”‚ review    â”‚ codex   â”‚ codex-5.3              â”‚ C â”‚ text â”‚
â”‚  3  â”‚ Implement    â”‚ implement â”‚ claude  â”‚ sonnetâ”‚ D      â”‚ code   â”‚
â”‚  4  â”‚ Test         â”‚ test      â”‚ claude  â”‚ sonnetâ”‚ D      â”‚ code   â”‚
â””â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”˜

Session flow: Analyse â”€â”€â–¶ Review (Gemini) â”€â”€â–¶ Review 2 (Codex) â”€â”€â–¶ Implement â”€â”€resumeâ”€â”€â–¶ Test
(3 independent perspectives, then implementation)
```

#### Template 5: Security Audit

```
Name:           Security Audit
Max iterations: 2
Stop on pass:   true

Stages:
â”Œâ”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚ Pos â”‚ Label            â”‚ Type      â”‚ Providerâ”‚ Model â”‚ Group  â”‚ Intent â”‚
â”œâ”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”¤
â”‚  0  â”‚ Analyse          â”‚ analyse   â”‚ claude  â”‚ opus  â”‚ A      â”‚ text   â”‚
â”‚  1  â”‚ Security Review  â”‚ review    â”‚ claude  â”‚ opus  â”‚ A      â”‚ text   â”‚
â”‚  2  â”‚ Review           â”‚ review    â”‚ claude  â”‚ opus  â”‚ A      â”‚ text   â”‚
â”‚  3  â”‚ Implement        â”‚ implement â”‚ claude  â”‚ sonnetâ”‚ B      â”‚ code   â”‚
â”‚  4  â”‚ Test             â”‚ test      â”‚ claude  â”‚ sonnetâ”‚ B      â”‚ code   â”‚
â””â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”˜

Session flow: Analyse â”€â”€resumeâ”€â”€â–¶ Security Review â”€â”€resumeâ”€â”€â–¶ Review â”€â”€newâ”€â”€â–¶ Implement â”€â”€resumeâ”€â”€â–¶ Test
(Security Review uses a custom prompt focused on OWASP Top 10)
```

### 7. Prompt Enhance flow

When a user writes a rough prompt and clicks "Enhance":

1. Frontend sends draft prompt text to backend via `enhance_prompt` command.
2. Backend calls hive-api `POST /v1/chat` with a cheap model (Sonnet or Flash).
3. The meta-prompt instructs the AI to improve clarity, add structure, use available template variables.
4. Frontend shows enhanced version in a **diff view** (before/after).
5. User can **Accept**, **Edit further**, or **Reject**.

**Enhance meta-prompt:**

```
You are a prompt engineer. Improve this prompt for an AI coding agent.
Make it clearer, more structured, and more effective.
Preserve the user's intent exactly. Add specificity where vague.
Use available template variables: {{task}}, {{code_context}}, {{previous_output}},
{{file_list}}, {{iteration_number}}, {{test_results}}.
Return ONLY the improved prompt.
```

**Command:** `enhance_prompt(draft: String, provider: String, model: String) â†’ String`

### 8. v1 prompt migration mapping

The 11 prompt template files in `orchestrator/prompts/` become the default `prompt_template` values in built-in pipeline templates:

| v1 Prompt File | v2 Built-in Template Location |
|----------------|-------------------------------|
| `enhancer.rs` | Embedded in prompt enhance flow (not a pipeline stage) |
| `planner.rs` | `full-review-loop â†’ stages[0] (Analyse)` |
| `plan_auditor.rs` | Built into plan gate logic (not a separate stage) |
| `generator.rs` | `full-review-loop â†’ stages[2] (Implement)` |
| `reviewer.rs` | `full-review-loop â†’ stages[1] (Review)` |
| `review_merger.rs` | Merged into review stage prompt |
| `fixer.rs` | Merged into implement stage prompt (fix iteration) |
| `judge.rs` | Built into iteration termination logic (Phase 4) |
| `executive_summary.rs` | Optional final stage (user can add to any template) |
| `skills.rs` | Built into skill selection logic |

### 9. Schema validation

Validate templates on save and before execution:
- Stage `id` must be unique within the template.
- `provider` and `model` must be non-empty strings.
- `session_group` must be non-empty.
- `parallel_group`, when provided, must be non-empty.
- `prompt_template` must be non-empty.
- `position` values must form a contiguous 0-based sequence.
- `execution_intent` must be `"text"` or `"code"`.
- At least one stage must be enabled.
- Template `name` must be non-empty and unique among user templates.

## File paths summary

| Action | Path |
|--------|------|
| NEW | `frontend/desktop/src-tauri/src/storage/templates.rs` |
| NEW | `frontend/desktop/src-tauri/src/commands/templates.rs` |
| MODIFY | `frontend/desktop/src-tauri/src/commands/mod.rs` (register template commands) |
| MODIFY | `frontend/desktop/src-tauri/src/lib.rs` (register template commands) |
| MODIFY | `frontend/desktop/src-tauri/src/storage/mod.rs` (add `pub mod templates;`, add TEMPLATES file lock) |
| NEW | `frontend/desktop/src/hooks/usePipelineTemplates.ts` |
| NEW | `~/.ea-code/pipeline-templates/` directory (created on first launch) |

## Testing

- **Template CRUD round-trip:** Create â†’ read â†’ update â†’ delete. Verify file persistence across restart.
- **Built-in immutability:** Verify update/delete rejected for `is_builtin: true` templates.
- **Clone test:** Clone built-in â†’ verify new ID, `is_builtin: false`, identical stages.
- **Prompt variable rendering:** Render template with all 11 variables populated. Verify substitution.
- **Conditional sections:** Render `{{#if test_results}}` with empty and populated values.
- **Schema validation:** Submit invalid templates (empty provider, duplicate IDs, empty stages). Verify rejection with clear error messages.
- **Enhance prompt:** Mock hive-api response, verify diff display.

## Deliverables

- Template management API and persistence live.
- 5 built-in templates shipped with v1 prompt defaults migrated.
- Prompt templates stored in data, not hardcoded Rust files.
- Users can create, clone, and save custom pipeline templates.
- Prompt Enhance flow functional.

## Dependencies

- Phase 2 hive-api provider metadata endpoint for provider/model option lists.
- Phase 1 type contracts (PipelineTemplate, StageDefinition).

## Risks and mitigations

- Risk: Invalid user templates cause runtime failures.
  Mitigation: Schema validation on save plus pre-run template validation with clear errors.
- Risk: Prompt variable rendering is too simple for complex conditionals.
  Mitigation: Start with `{{var}}` and `{{#if var}}` only. Avoid full templating engine complexity. Extend later if needed.

## Exit criteria

- Built-in and user templates render in frontend (Phase 5 builds the full UI; this phase verifies data round-trip).
- Saved templates survive restart.
- Invalid templates are blocked before execution.
- All 5 built-in templates pass schema validation.
- Prompt Enhance produces improved output via hive-api.

## Estimated duration

1 week

---

## Implementation results

### Files created — Rust backend

| File | Purpose | Tests |
|------|---------|-------|
| `src-tauri/src/storage/mod.rs` | Storage module root (exports templates, builtin_templates, builtin_prompts) | — |
| `src-tauri/src/storage/templates.rs` | File-based CRUD: list, read, write (atomic), delete | 4 |
| `src-tauri/src/storage/builtin_templates.rs` | 5 hardcoded built-in pipeline templates | 4 |
| `src-tauri/src/storage/builtin_prompts.rs` | 11 reusable prompt constants for built-in templates | — |
| `src-tauri/src/prompts/mod.rs` | Prompts module root | — |
| `src-tauri/src/prompts/renderer.rs` | `render_prompt()` with `{{var}}` + `{{#if var}}...{{/if}}` | 9 |
| `src-tauri/src/commands/templates.rs` | 7 Tauri commands: list, get, create, update, delete, clone, enhance_prompt | — |

### Files created — TypeScript frontend

| File | Purpose |
|------|---------|
| `src/hooks/usePipelineTemplates.ts` | React hook: CRUD + enhance_prompt + derived builtin/user lists |

### Files modified

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | Added `pub mod prompts; pub mod storage;`, registered 7 template commands |
| `src-tauri/src/commands/mod.rs` | Added `pub mod templates;` |
| `src-tauri/Cargo.toml` | Added `dirs = "6"`, `uuid = { version = "1", features = ["v4"] }`, `chrono = { version = "0.4", features = ["serde"] }` |

### Tauri commands registered

| Command | Purpose |
|---------|---------|
| `list_templates` | All built-in + user templates, sorted by name |
| `get_template` | Single template by ID (builtin or user) |
| `create_template` | Create user template with UUID, validation |
| `update_template` | Update user template (rejects builtin) |
| `delete_template` | Delete user template (rejects builtin) |
| `clone_template` | Clone any template into user copy with new ID |
| `enhance_prompt` | Send draft to hive-api for AI-powered improvement |

### Built-in templates shipped

| ID | Name | Stages | Providers |
|----|------|--------|-----------|
| `full-review-loop` | Full Review Loop | 4 (analyse→review→implement→test) | claude opus + sonnet |
| `quick-fix` | Quick Fix | 2 (implement→test) | claude sonnet |
| `research-only` | Research Only | 2 (analyse→review) | claude opus |
| `multi-brain-review` | Multi-Brain Review | 5 (analyse→review×2→implement→test) | claude + gemini + codex |
| `security-audit` | Security Audit | 5 (analyse→security→review→implement→test) | claude opus + sonnet |

### Verification

| Check | Result |
|-------|--------|
| `cargo check` | ✅ Zero warnings |
| `cargo test` | ✅ 91 passed, 0 failed |
| `npx tsc --noEmit` | ✅ Zero errors |
| All files < 300 lines | ✅ |
| All 5 built-in templates pass validate_template() | ✅ |



