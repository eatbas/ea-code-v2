# ADR 002 — Session Group Semantics

| Field   | Value                          |
|---------|--------------------------------|
| Status  | Accepted                       |
| Date    | 2026-03-23                     |
| Scope   | Orchestrator, hive-api, Agents |

## Context

EA Code v1 spawns a fresh CLI process for every stage. Each process starts with zero memory of previous stages. Context passes between stages only as text output injected into prompts via `{{previous_output}}`.

This has two costs:

1. **Lost context.** A reviewer cannot see the analyser's reasoning chain, only its final output. Nuance is lost in the text handoff.
2. **Redundant token spend.** Every stage re-reads the same workspace context from scratch because no provider session carries over.

## Decision

Stages in a pipeline template declare a `session_group` identifier (e.g., `"A"`, `"B"`). The orchestrator uses this group, combined with the stage's provider and model, to decide whether to resume an existing provider session or start a new one.

### Resolution rules

| Same group? | Same provider? | Same model? | Behaviour |
|:-----------:|:--------------:|:-----------:|-----------|
| Yes         | Yes            | Yes         | **Resume** — pass `provider_session_ref` to hive-api |
| Yes         | Yes            | No          | **New session** — cannot resume across models within a provider |
| Yes         | No             | —           | **New session** — cannot resume across providers |
| No          | —              | —           | **New session** — output from prior group passed as `{{previous_output}}` text |

### Resume mechanism

When resuming, the orchestrator passes a `provider_session_ref` (an opaque string returned by hive-api on session creation) to the next stage's API call. The provider maintains conversation history server-side, so the new stage sees the full context of prior stages in the same group.

### Cross-model rejection

If a template assigns the same session group to stages with different models on the same provider, the orchestrator falls back to `new` mode and logs a warning:

```
WARN: Stage "Review" (claude/opus) cannot resume session from "Implement" (claude/sonnet) —
      same group "A" but different models. Starting new session.
```

This is a configuration error, not a runtime failure. The pipeline continues.

### Cross-group handoff

When a stage belongs to a different group than the preceding stage, the orchestrator injects the prior stage's output into the `{{previous_output}}` template variable. This preserves the v1 text-handoff behaviour for intentional context boundaries.

## Consequences

### Positive

- **Compounding intelligence.** A reviewer in group A has the full reasoning chain from the analyser in group A, not just a text summary.
- **Cost optimisation.** Users can assign expensive models (Opus) to thinking stages in group A and cheaper models (Sonnet) to coding stages in group B, with each group maintaining its own session context.
- **Explicit boundaries.** Session groups make context boundaries visible in the template definition rather than implicit in code.

### Negative

- Provider-specific session mechanisms must be managed. Each provider adapter needs to support session creation, resumption, and cleanup.
- Session state is held server-side by the provider, adding a dependency on provider session durability.
- Misconfigured groups (same group, different models) produce warnings that users must understand.

### Neutral

- The `provider_session_ref` is opaque to the orchestrator. Provider adapters own the format and lifecycle.
- Built-in templates use session groups that follow the resolution rules correctly. Users who clone and edit templates are responsible for valid group assignments.
