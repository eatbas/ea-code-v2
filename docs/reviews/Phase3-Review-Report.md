# Phase 3 Review Report (Recheck)

**Date:** 2026-03-23  
**Reviewer:** Codex  
**Scope:** Re-validation after latest Phase 3 fixes

## Findings (Ordered by Severity)

### [P2] Storage durability/concurrency alignment is still incomplete

Status: **Open**

Evidence:
- `write_template` still uses `tmp -> rename` only, without explicit backup file or per-file mutex serialisation: `frontend/desktop/src-tauri/src/storage/templates.rs:71`
- Storage tests still rely on helper functions (`write_to_dir`, `list_from_dir`) rather than directly exercising production APIs (`write_template`, `list_user_templates`) in their real path flow: `frontend/desktop/src-tauri/src/storage/templates.rs:150`, `frontend/desktop/src-tauri/src/storage/templates.rs:174`

Impact:
- Concurrent writes are not explicitly guarded, and persistence behaviour is weaker than the Phase 3 stated storage pattern.

## Previously Reported Findings - Recheck

### [P1] Template name uniqueness rule

Status: **Fixed**

Evidence:
- Added `check_name_unique(...)` and called from create/update/clone:
  - `frontend/desktop/src-tauri/src/commands/templates.rs:62`
  - `frontend/desktop/src-tauri/src/commands/templates.rs:145`
  - `frontend/desktop/src-tauri/src/commands/templates.rs:178`
  - `frontend/desktop/src-tauri/src/commands/templates.rs:219`

### [P2] Malformed template file breaks listing

Status: **Fixed**

Evidence:
- `list_user_templates` now skips unreadable/malformed files and continues:
  - `frontend/desktop/src-tauri/src/storage/templates.rs:38`
  - `frontend/desktop/src-tauri/src/storage/templates.rs:45`
- Added malformed listing test coverage:
  - `frontend/desktop/src-tauri/src/storage/templates.rs:249`

## Verification

Commands executed:
- `cd frontend/desktop/src-tauri && cargo check`
- `cd frontend/desktop/src-tauri && cargo test`
- `cd frontend/desktop && npx tsc --noEmit`

Results:
- `cargo check`: **PASS**
- `cargo test`: **PASS** (92 passed, 0 failed)
- `npx tsc --noEmit`: **PASS**

## Critical Coverage Check

| Area | Status | Notes |
|---|---|---|
| Template model/validation unit tests | Covered | Schema checks in `models/validation.rs` |
| Built-in template integrity tests | Covered | Count + validation + shape checks |
| Prompt renderer variable/conditional behaviour | Covered | Renderer tests pass |
| Name uniqueness behaviour in commands | Implemented, lightly covered | Logic present; no dedicated command-level behavioural tests for conflict paths |
| Malformed template file listing behaviour | Covered | New malformed JSON listing test present |
| Real storage API behaviour on production paths | Partially covered | Helper-path tests still dominate storage coverage |
| Frontend hook (`usePipelineTemplates`) behaviour/error transitions | Not covered | No frontend tests found |

## Conclusion

Two previously reported blockers are fixed (name uniqueness and malformed-file listing resilience).  
One meaningful storage robustness gap remains open around durability/concurrency guarantees.
