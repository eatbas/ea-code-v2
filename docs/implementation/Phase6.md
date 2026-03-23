# Phase 6 - Migration, Hardening, and Release Readiness

## Objective

Safely migrate existing users to v2, remove obsolete v1 paths, and prepare a stable release.

## Scope

- Migrate settings, prompt defaults, and agent assignments.
- Remove retired CLI-specific modules and UI.
- Validate reliability, performance, and upgrade safety.
- Update CI/CD for hive-api bundling.
- Update all project documentation.

## Work items

### 1. Settings migration (v1 â†’ v2)

**File: `frontend/desktop/src-tauri/src/storage/migration.rs` (EXTEND)**

On first v2 launch, detect and migrate `settings.json`:

1. **Read** v1 settings.
2. **Create default pipeline template** from v1 per-stage agent/model assignments:
   - Map v1 `enhancer_agent` â†’ v2 template stage 0 (Analyse) provider/model.
   - Map v1 `planner_agent` â†’ v2 template stage 0 (Analyse) â€” merged with enhancer.
   - Map v1 `coder_agent` â†’ v2 template stage 2 (Implement) provider/model.
   - Map v1 `reviewer_agent` â†’ v2 template stage 1 (Review) provider/model.
   - Map v1 `judge_agent` â†’ iteration judge logic (not a template stage).
   - Save as a user template named "Migrated from v1".
3. **Remove** CLI path fields (`claude_path`, `codex_path`, `gemini_path`, `kimi_path`, `opencode_path`).
4. **Add** hive-api connection fields with defaults (`hive_api_host: "127.0.0.1"`, `hive_api_port: 8000`, `auto_start_hive_api: true`).
5. **Set** `default_pipeline_id` to the migrated template id for upgraded users (preserve prior behaviour). Use "Full Review Loop" only for fresh installs with no v1 settings.
6. **Increment** schema version field (add `settings_version: 2` to distinguish v1 from v2 settings).
7. **Write** v2 settings using atomic write pattern.

**Safety:** Before writing, create a backup at `~/.ea-code/settings.v1.backup.json`. Migration is idempotent â€” running it twice produces the same result.

### 2. Prompt migration

The 11 v1 prompt files are already migrated to built-in template defaults in Phase 3. This step verifies and cleans up:

| v1 File | v2 Location | Status |
|---------|-------------|--------|
| `orchestrator/prompts/enhancer.rs` | Prompt enhance flow (not a stage) | Embedded in Phase 3 |
| `orchestrator/prompts/planner.rs` | Full Review Loop â†’ Analyse stage prompt | Migrated |
| `orchestrator/prompts/plan_auditor.rs` | Plan gate logic | Embedded in Phase 4 |
| `orchestrator/prompts/generator.rs` | Full Review Loop â†’ Implement stage prompt | Migrated |
| `orchestrator/prompts/reviewer.rs` | Full Review Loop â†’ Review stage prompt | Migrated |
| `orchestrator/prompts/review_merger.rs` | Merged into review stage prompt | Migrated |
| `orchestrator/prompts/fixer.rs` | Merged into implement stage prompt | Migrated |
| `orchestrator/prompts/judge.rs` | Iteration judge logic | Embedded in Phase 4 |
| `orchestrator/prompts/executive_summary.rs` | Optional summary stage template | Available |
| `orchestrator/prompts/skills.rs` | Skill selection logic | Embedded in Phase 4 |

Verify all prompt content is preserved in the new locations before deleting source files.

### 3. File deletion list

**Backend files to delete:**

```
frontend/desktop/src-tauri/src/agents/                     # ENTIRE DIRECTORY (replaced by hive_client/)
â”œâ”€â”€ mod.rs
â”œâ”€â”€ base/mod.rs
â”œâ”€â”€ base/windows.rs
â”œâ”€â”€ claude.rs
â”œâ”€â”€ codex.rs
â”œâ”€â”€ gemini.rs
â”œâ”€â”€ kimi.rs
â”œâ”€â”€ opencode.rs
â””â”€â”€ mcp.rs

frontend/desktop/src-tauri/src/commands/cli.rs             # CLI health checks â†’ hive-api /health
frontend/desktop/src-tauri/src/commands/cli_version.rs     # Version checks â†’ hive-api /v1/cli-versions
frontend/desktop/src-tauri/src/commands/cli_http.rs        # HTTP utils for npm registry
frontend/desktop/src-tauri/src/commands/cli_util.rs        # CLI path resolution
frontend/desktop/src-tauri/src/commands/git_bash.rs        # Windows Git Bash detection

frontend/desktop/src-tauri/src/orchestrator/prompts/       # ENTIRE DIRECTORY (migrated to templates)
â”œâ”€â”€ enhancer.rs
â”œâ”€â”€ planner.rs
â”œâ”€â”€ plan_auditor.rs
â”œâ”€â”€ generator.rs
â”œâ”€â”€ reviewer.rs
â”œâ”€â”€ review_merger.rs
â”œâ”€â”€ fixer.rs
â”œâ”€â”€ judge.rs
â”œâ”€â”€ executive_summary.rs
â””â”€â”€ skills.rs
```

**Frontend files to delete:**

```
frontend/desktop/src/components/AgentsView/                # ENTIRE DIRECTORY (replaced by PipelineBuilderView)
â”œâ”€â”€ index.tsx
â”œâ”€â”€ StageCard.tsx
â”œâ”€â”€ CascadingSelect.tsx
â”œâ”€â”€ InlineStageSlot.tsx
â””â”€â”€ agentHelpers.ts

frontend/desktop/src/components/CliSetupView/              # ENTIRE DIRECTORY (replaced by HiveApiStatusView)
â”œâ”€â”€ index.tsx
â””â”€â”€ CliCard.tsx

frontend/desktop/src/hooks/useCliHealth.ts                 # Replaced by useHiveApi
frontend/desktop/src/hooks/useCliVersions.ts               # Replaced by useHiveApi
```

**Backend files to modify (remove dead references):**

- `frontend/desktop/src-tauri/src/lib.rs` â€” Remove registration of deleted CLI commands, remove `pub mod agents;`.
- `frontend/desktop/src-tauri/src/commands/mod.rs` â€” Remove CLI command re-exports.
- `frontend/desktop/src-tauri/src/orchestrator/mod.rs` â€” Remove `pub mod prompts;`.
- `frontend/desktop/src-tauri/src/models/agents.rs` â€” Remove CLI path helpers, keep `AgentRole` and `AgentBackend` enums.
- `frontend/desktop/src-tauri/src/models/environment.rs` â€” Remove `CliHealth`, `CliStatus`, `CliVersionInfo`, `AllCliVersions`.

**Frontend files to modify:**

- `frontend/desktop/src/types/agents.ts` â€” Remove `CliHealth`, `CliStatus`, `CliVersionInfo`, `AllCliVersions`.
- `frontend/desktop/src/types/index.ts` â€” Remove deleted type re-exports.
- `frontend/desktop/src/components/Sidebar.tsx` â€” Remove "CLI Setup" navigation item.

### 4. Regression test suite

Add or verify tests covering:

| Area | Test | Expectation |
|------|------|-------------|
| Run lifecycle | Start â†’ stages â†’ complete | All stage events in order, run summary written |
| Run lifecycle | Start â†’ cancel mid-stage | hive-api job stopped, run marked cancelled |
| Run lifecycle | Start â†’ pause â†’ resume â†’ complete | Run completes after resume |
| Crash recovery | Kill app during run, relaunch | Orphaned run marked failed with synthetic RunEnd |
| Session history | Load projects â†’ sessions â†’ runs | All v1 and v2 runs display correctly |
| Template CRUD | Create â†’ update â†’ delete | File persistence verified |
| Template execution | Run each of 5 built-in templates | All complete successfully |
| Settings migration | v1 settings file â†’ v2 migration | CLI paths removed, hive-api fields added, user template created |
| hive-api lifecycle | App start â†’ hive-api ready â†’ app close â†’ hive-api stopped | Clean lifecycle |
| Large event logs | Run with 1000+ events | History view loads within 2 s |
| Long sessions | 5-iteration run with 4 stages each | All 20 stages execute, events log grows correctly |

### 5. Performance validation

- Large event logs (`events.jsonl` with 1000+ entries): verify history view load time < 2 s.
- Long sessions (5+ iterations): verify memory usage stays stable (no event accumulation leak).
- hive-api boot time: verify < 30 s (Phase 1 SLO).
- Stage start latency: verify < 500 ms from dispatch to first SSE event (Phase 1 SLO).

### 6. CI/CD updates

**File: `.github/workflows/release.yml` (MODIFY)**

- Add hive-api bundling step to the build pipeline (based on Phase 1 bundling decision).
- If sidecar: include hive-api Python package in the Tauri bundle resources.
- If PyInstaller: add build step to compile hive-api binary, include in NSIS installer.
- Update artifact paths for the new bundle structure.
- Verify signing with `TAURI_SIGNING_PRIVATE_KEY` still works with larger bundle.

**Files: `scripts/release.sh`, `scripts/release.ps1` (MODIFY)**

- Add hive-api version to the version bump step (if hive-api is versioned separately).
- Or: pin hive-api to the app version (single version number).

### 7. Documentation updates

**File: `CLAUDE.md` (UPDATE)**

- Update module map: add `hive_client/`, `contexts/`, `PipelineBuilderView/`, `HiveApiStatusView/`.
- Remove `agents/` from module listings.
- Remove CLI command references.
- Add new Tauri commands (template CRUD, hive-api lifecycle).
- Update pipeline stages section (dynamic from templates, not hardcoded 13-stage list).
- Update storage layout (add `pipeline-templates/` directory).
- Update IPC conventions (dynamic stage labels, new event types).
- Update type listings (add PipelineTemplate, StageDefinition, remove CLI types).

**File: `AGENTS.md` (UPDATE)**

- Update to reflect hive-api provider model (no direct CLI execution).
- Document supported providers and their session resume capabilities.

**File: `README.md` (UPDATE)**

- Update feature list for v2 (configurable pipelines, session resume, custom prompts).
- Update architecture diagram.
- Update getting started instructions (hive-api dependency).

### 8. Release checklist

- [ ] All built-in templates execute end-to-end on Windows.
- [ ] Settings migration runs successfully on a v1 installation.
- [ ] hive-api starts and stops cleanly with the app.
- [ ] All v1 session history loads correctly in v2.
- [ ] No high-severity defects open.
- [ ] CI build produces signed installer with hive-api bundled.
- [ ] `npx tsc --noEmit` passes (frontend).
- [ ] `cargo check` passes (backend).
- [ ] CLAUDE.md reflects v2 architecture.
- [ ] Release notes drafted with migration instructions.

### 9. Rollback plan

If critical issues found after release:
- Settings backup (`settings.v1.backup.json`) allows manual rollback.
- v1 run history is not modified â€” v2 adds fields but doesn't remove v1 data.
- hive-api can be disabled (`auto_start_hive_api: false`) to fall back to... (note: v1 CLI paths are removed, so true rollback requires reinstalling v1 binary).
- Recommended: keep v1 installer available for manual downgrade during the first 2 release cycles.

## Deliverables

- Automated migration with rollback-safe behaviour.
- Clean codebase without legacy CLI execution code paths.
- All deleted files verified as unused before removal.
- Release candidate build with documented known issues and mitigation notes.
- Updated CLAUDE.md, AGENTS.md, and README.md.

## Dependencies

- Phases 1 to 5 complete.
- hive-api bundling strategy implemented (from Phase 1 decision, Phase 2 implementation).

## Risks and mitigations

- Risk: Migration errors impact existing user data.
  Mitigation: Backup-before-write migration, idempotent migration guards, and manual rollback path.
- Risk: Deleting files breaks unnoticed cross-references.
  Mitigation: Run `cargo check` and `npx tsc --noEmit` after every deletion batch. Fix compilation errors before proceeding.
- Risk: hive-api bundling increases installer size significantly.
  Mitigation: Measure bundle size increase. If > 50 MB, evaluate compression or lazy download.

## Exit criteria

- Upgrade from latest v1 to v2 succeeds on Windows (and macOS when CI build is re-enabled).
- No high-severity defects open in run lifecycle, data integrity, or template execution.
- All regression tests pass.
- Performance SLOs met (event log load, boot time, stage latency).
- Release sign-off complete.

## Estimated duration

1 week



