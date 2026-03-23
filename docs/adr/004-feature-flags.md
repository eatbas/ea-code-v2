# ADR 004 — Feature Flags for v1-to-v2 Rollout

| Field   | Value                     |
|---------|---------------------------|
| Status  | Accepted                  |
| Date    | 2026-03-23                |
| Scope   | Settings, Orchestrator, UI |

## Context

v2 replaces the core execution path: the hardcoded 13-stage pipeline gives way to template-driven dynamic execution via hive-api. This is not a minor enhancement — it changes how every pipeline run is dispatched, monitored, and stored.

A big-bang migration (remove v1 path, ship v2) risks stranding users if hive-api bundling is not ready, a provider adapter has bugs, or the new orchestrator has edge cases not covered by testing.

## Decision

Add a `settings_version: u32` field to `AppSettings`. The orchestrator checks this field at pipeline start to determine which execution path to use.

### Behaviour

| `settings_version` value | Execution path |
|:------------------------:|----------------|
| Absent or `1`            | v1 — existing CLI-based orchestrator (current `orchestrator/` code) |
| `2`                      | v2 — template-driven orchestrator via hive-api |

### Migration trigger

When a user upgrades to a build that includes v2 support:

1. The settings loader reads `settings.json`.
2. If `settings_version` is absent, it defaults to `1`. The user stays on the v1 path.
3. The UI shows a migration prompt explaining v2 features and asking the user to opt in.
4. On opt-in, `settings_version` is set to `2`, built-in templates are written to storage, and the v2 path activates.
5. On opt-out, the user remains on v1 indefinitely. The prompt does not reappear until the next major version.

### Rollback

If a user on v2 encounters issues, they can set `settings_version` back to `1` in settings. The v1 orchestrator path remains in the binary during the transition period.

### Cleanup

Once v2 is stable and v1 usage drops below the threshold (measured via opt-in telemetry if available, or after a fixed number of releases), the v1 path is removed:

- Delete the v1 orchestrator code.
- Set `settings_version` minimum to `2`.
- The migration prompt becomes a forced migration notice.

## Consequences

### Positive

- Safe, per-user rollout. Early adopters can test v2 while conservative users stay on v1.
- Revertible. A single settings change rolls back to v1 without reinstalling.
- No flag proliferation. A single version integer controls the execution path, not a matrix of boolean flags.

### Negative

- Temporary code duplication. Both v1 and v2 orchestrator paths must be maintained during the transition window.
- Testing surface doubles. CI must verify both paths until v1 is removed.
- The migration prompt adds UI complexity that is discarded after the transition.

### Neutral

- The `settings_version` field persists in `settings.json` permanently but costs nothing after the v1 path is removed.
- Built-in templates are written to storage on first v2 activation, not on app install.
