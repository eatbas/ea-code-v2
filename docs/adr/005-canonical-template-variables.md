# ADR 005 — Canonical Template Variables

| Field   | Value                          |
|---------|--------------------------------|
| Status  | Accepted                       |
| Date    | 2026-03-23                     |
| Scope   | Orchestrator, Templates, Docs  |

## Context

Pipeline templates contain prompt text with placeholder variables (e.g., `{{task}}`). The orchestrator substitutes these at runtime before sending the prompt to a provider.

Without a locked variable set, drift is inevitable: one template uses `{{task}}`, another uses `{{user_prompt}}`, a third uses `{{prompt}}`. The orchestrator must then support all variants or silently leave placeholders unresolved.

## Decision

The following 11 template variables are canonical. The orchestrator recognises exactly these names. No aliases are permitted.

| Variable                | Type    | Description |
|-------------------------|---------|-------------|
| `{{task}}`              | String  | The user's original prompt, unmodified |
| `{{workspace_path}}`    | String  | Absolute path to the project workspace |
| `{{file_list}}`         | String  | Newline-separated list of files in the workspace |
| `{{code_context}}`      | String  | Relevant file contents (selected by the orchestrator or prior stage) |
| `{{previous_output}}`   | String  | Output from the preceding stage (empty for the first stage) |
| `{{iteration_number}}`  | Integer | Current iteration, 1-based |
| `{{max_iterations}}`    | Integer | Maximum iterations defined in the template |
| `{{test_results}}`      | String  | Test runner output from the most recent iteration (empty if no tests ran) |
| `{{judge_feedback}}`    | String  | Judge reasoning from the prior iteration (empty on first iteration) |
| `{{git_branch}}`        | String  | Current git branch name |
| `{{git_diff}}`          | String  | Working tree diff (`git diff`) at the time of stage execution |

### Substitution rules

1. **Exact match only.** `{{task}}` is valid. `{{ task }}` (with spaces), `{{Task}}`, and `{{user_prompt}}` are not.
2. **Unresolved variables are errors.** If a prompt contains `{{unknown_var}}`, the orchestrator logs an error and halts the stage. Silent pass-through of unresolved placeholders is not permitted.
3. **Empty values are valid.** A variable that resolves to an empty string (e.g., `{{test_results}}` when no tests ran) is substituted as-is. The prompt receives an empty string, not an error.
4. **No nested substitution.** If a variable's value contains `{{another_var}}`, the inner placeholder is treated as literal text, not expanded.

### Extension process

Adding a new canonical variable requires:

1. An update to this ADR with the new variable name, type, and description.
2. Orchestrator code changes to populate the variable.
3. An update to template validation to recognise the new name.

Removing a variable follows the same process in reverse, with a deprecation period of at least one minor release.

## Consequences

### Positive

- Template authors have a single, documented reference for available variables.
- The orchestrator's substitution logic is a simple map lookup, not a fuzzy matcher.
- Errors from typos (`{{taks}}`) are caught immediately rather than producing mysterious prompt gaps.

### Negative

- Rigidity. Adding a variable requires an ADR update, not just a code change. This is intentional friction.
- Users cannot define custom variables without extending the orchestrator.

### Neutral

- The 11-variable set covers the inputs needed by all five built-in templates. Custom templates that need additional context must request it via prompt text, not new variables.
