# ADR 001 — Dynamic Pipeline Execution

| Field   | Value                     |
|---------|---------------------------|
| Status  | Accepted                  |
| Date    | 2026-03-23                |
| Scope   | Orchestrator, Storage, UI |

## Context

EA Code v1 compiles a fixed 13-stage pipeline into Rust. The stage order, provider assignments, model choices, and prompt text are all hard-coded. Changing any aspect of the pipeline requires a code change, a recompile, and a new release.

This creates three problems:

1. **One-size-fits-all execution.** A trivial typo fix runs the same heavyweight pipeline as a cross-module refactor. Users cannot skip stages they do not need.
2. **Prompt rigidity.** Prompt engineering is locked behind the release cycle. Users who want to tune reviewer or planner behaviour must fork the repo.
3. **Backend coupling.** Adding a new agent backend or model requires touching orchestrator internals, not just configuration.

## Decision

Pipeline stages are defined in JSON templates. The orchestrator reads stage order, provider/model assignments, and prompt templates at runtime rather than compiling them in.

### Data model

A `PipelineTemplate` contains ordered `StageDefinition` entries:

```
PipelineTemplate {
  id: String,
  name: String,
  description: String,
  max_iterations: u32,
  stop_on_first_pass: bool,
  built_in: bool,
  stages: Vec<StageDefinition>,
}

StageDefinition {
  position: u32,
  label: String,
  stage_type: "text" | "code",
  provider: String,
  model: String,
  session_group: String,
  execution_intent: String,
  prompt_template: String,
}
```

### Orchestrator changes

- The orchestrator receives a `template_id` (or the full template object) when a pipeline run starts.
- It iterates `stages` in `position` order, dispatching each to the appropriate agent backend.
- Prompt templates use the canonical variables defined in ADR 005.
- Iteration control respects `max_iterations` and `stop_on_first_pass` from the template.

### User-facing operations

- **Create**: Users define a new template from scratch or clone an existing one.
- **Edit**: Users modify stage order, provider/model, prompts, and iteration settings.
- **Delete**: Users can delete custom templates. Built-in templates cannot be deleted but can be cloned and modified.
- **Select**: Users pick a template before starting a pipeline run.

### Built-in defaults

Five built-in templates ship with the application (documented in `docs/built-in-templates.md`). These cover the most common workflows and serve as starting points for customisation.

## Consequences

### Positive

- Users own their workflows. A security team can build an audit-focused pipeline; a frontend team can skip code review for styling tasks.
- Different templates for different task types eliminates wasted computation on simple tasks.
- Prompt text is editable without rebuilding the application.
- New agent backends only need a provider adapter, not orchestrator surgery.

### Negative

- The orchestrator becomes more complex. It must validate templates at runtime (missing fields, unknown providers, circular dependencies).
- Runtime validation adds a failure mode that did not exist with compile-time guarantees.
- Migration from v1 is required. Existing users need their settings mapped to the new template format.

### Neutral

- Five built-in templates ship as defaults, preserving the v1 experience for users who do not customise.
- Template storage uses the existing file-based persistence under `~/.ea-code/`.
