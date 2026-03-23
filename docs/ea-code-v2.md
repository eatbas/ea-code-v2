# EA Code v2 — Complete Architecture & Rewrite Analysis

**Date:** 2026-03-23
**Status:** Architecture Design Phase
**Authors:** EA Code Team + AI Analysis

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Why v2 — The Case for a Rewrite](#2-why-v2--the-case-for-a-rewrite)
3. [v1 Codebase Audit](#3-v1-codebase-audit)
4. [hive-api Integration](#4-hive-api-integration)
5. [v2 Architecture Overview](#5-v2-architecture-overview)
6. [Configurable Pipelines](#6-configurable-pipelines)
7. [Session Resume & Session Groups](#7-session-resume--session-groups)
8. [Custom Prompts & Template Variables](#8-custom-prompts--template-variables)
9. [Data Model & Storage](#9-data-model--storage)
10. [Frontend Architecture](#10-frontend-architecture)
11. [Backend Architecture](#11-backend-architecture)
12. [Migration Strategy — What to Keep, Remove, Rewrite](#12-migration-strategy--what-to-keep-remove-rewrite)
13. [Implementation Roadmap](#13-implementation-roadmap)

---

## 1. Executive Summary

EA Code v2 is a **ground-up rebuild** of the desktop AI orchestration app. The core change: replacing direct CLI process spawning with **hive-api**, a Python FastAPI service that manages AI provider CLIs as warm drone processes.

### Key Architectural Shifts

| Area | v1 (Current) | v2 (New) |
|------|-------------|----------|
| Agent execution | Spawn CLI process per stage | HTTP POST to hive-api `/v1/chat` |
| Pipeline structure | Hardcoded 13-stage loop | User-configurable, drag-and-drop |
| Session continuity | Cold start every stage | Resume sessions within stage groups |
| Prompts | Hardcoded in Rust | User-editable with template variables |
| Pipeline count | One pipeline for all tasks | Multiple named pipelines |
| Iteration limits | Global setting | Per-pipeline configuration |
| Providers | 5 (Claude, Codex, Gemini, Kimi, OpenCode) | 6 (+ Copilot via hive-api) |
| CLI management | App manages CLI installs/updates | hive-api manages CLI lifecycle |
| Windows process mgmt | Custom Git Bash wrapper | hive-api handles it |

### What This Enables

1. **Compounding intelligence** — Analyse + Review share a session, so the reviewer has full context from the analyser (no cold restart).
2. **User-owned pipelines** — Users drag-and-drop stages, assign providers, write custom prompts, and save reusable pipeline templates.
3. **Multi-pipeline workflows** — "Quick Fix" (1 iteration), "Full Review Loop" (5 iterations), "Research Only" (analysis + review, no implementation).
4. **Prompt engineering** — Every stage prompt is editable, with an AI "Enhance" button to improve prompts.
5. **Simpler backend** — Remove ~1,100 lines of CLI process management code. hive-api owns all of it.

---

## 2. Why v2 — The Case for a Rewrite

### Problems With v1

1. **CLI spawning is fragile** — Every stage spawns a fresh CLI process. Windows requires Git Bash workarounds, temp prompt files, and process tree killing. 433 lines of agent code + 500 lines of CLI commands handle this complexity.

2. **No session continuity** — The "Review" agent has zero memory of what "Analyse" found. Context is passed only via text output in prompts, losing nuance and increasing token costs.

3. **Hardcoded pipeline** — The 13-stage pipeline is compiled into the orchestrator. Adding, removing, or reordering stages requires Rust code changes.

4. **No prompt customisation** — Prompt templates are embedded in Rust source files (11 files in `orchestrator/prompts/`). Users cannot modify them without rebuilding.

5. **Single pipeline** — Every task runs the same pipeline regardless of complexity. A typo fix goes through the same 13-stage process as a feature build.

### Why Rewrite Instead of Refactor

The pipeline structure is the **core abstraction** of the app. Changing from "hardcoded stages in Rust" to "user-configurable stages from a template" touches:

- The orchestrator (2,200 lines)
- The agent dispatch layer (433 lines)
- The commands layer (1,600 lines)
- The models layer (1,100 lines)
- The frontend components (8,095 lines)
- The type system (both Rust and TypeScript)

This is not a localised refactor — it's a fundamental redesign of the data flow. Starting fresh allows clean abstractions without backward-compatibility constraints.

---

## 3. v1 Codebase Audit

### 3.1 Overall Statistics

| Layer | Lines | Files | Reusable in v2 |
|-------|-------|-------|----------------|
| Rust Backend Total | 14,575 | ~60 | ~70% |
| — Orchestrator | 2,200 | 23 | Redesign (concepts reusable) |
| — Storage | 2,400 | 12 | **Keep** (proven patterns) |
| — Models | 1,100 | 11 | Partial (remove CLI fields) |
| — Commands | 1,600 | 13 | Partial (remove CLI commands) |
| — Agents | 433 | 8 | **Remove** (replaced by hive-api) |
| Frontend Total | 10,100 | 85+ | ~40% |
| — Components | 5,700 | ~50 | Shared UI reusable, views redesign |
| — Hooks | 1,771 | 17 | Partial (pipeline hooks redesign) |
| — Types | 939 | 10 | Partial (pipeline types redesign) |
| — Utils | 684 | 6 | Mostly reusable |

### 3.2 Rust Backend — What Exists

#### Agents Layer (433 lines — ALL REMOVED in v2)

| File | Lines | Purpose |
|------|-------|---------|
| `agents/base/mod.rs` | 303 | Core `run_cli_agent()`: spawn process, pipe stdout/stderr, emit log events |
| `agents/base/windows.rs` | 113 | Windows Git Bash wrapper, temp prompt files, path conversion |
| `agents/claude.rs` | 148 | Claude CLI args (--print --verbose stream-json) |
| `agents/codex.rs` | 60 | Codex CLI args (exec --full-auto) |
| `agents/gemini.rs` | 50 | Gemini CLI args (--approval-mode yolo) |
| `agents/kimi.rs` | 138 | Kimi CLI args (--print stream-json, PYTHONIOENCODING=utf-8) |
| `agents/opencode.rs` | 42 | OpenCode CLI args (stdin prompt) |
| `agents/mcp.rs` | 72 | Temporary MCP config builder |

**v2 replacement:** Single HTTP client module (~200 lines) calling hive-api `/v1/chat`.

#### Orchestrator (2,200 lines — REDESIGN for configurable pipelines)

**Current flow (hardcoded):**
```
PromptEnhance → SkillSelect → Plan(1-4) → PlanAudit → Coder
→ CodeReviewer(1-4) → ReviewMerge → CodeFixer → Judge → [loop or complete]
→ ExecutiveSummary
```

**Key modules:**
- `pipeline/mod.rs` (233 lines) — Main `run_pipeline()` loop
- `iteration/mod.rs` (192 lines) — Single iteration dispatch
- `iteration_planning/mod.rs` (219 lines) — Parallel planners + auditor
- `iteration_review/mod.rs` (236 lines) — Parallel reviewers + merger + fixer
- `helpers.rs` (433 lines) — `dispatch_agent()` routing, event emission
- `parallel_stage.rs` (105 lines) — Parallel execution runner
- `prompts/` (11 files, ~1,300 lines) — All prompt templates

**v2 redesign:** Pipeline reads stage order from a template, dispatches via hive-api HTTP, manages session groups for resume.

#### Commands Layer (1,600 lines — PARTIAL keep)

**Keep (core IPC):**
- `pipeline.rs` (197 lines) — run/cancel/pause/resume/answer (redesign for new pipeline)
- `workspace.rs` (48 lines) — workspace selection
- `settings.rs` (14 lines) — get/save settings
- `skills.rs` (90 lines) — CRUD
- `history.rs` (119 lines) — session/run history
- `mcp.rs` (246 lines) — MCP server management

**Remove (CLI-specific):**
- `cli.rs` (278 lines) — CLI health checks → hive-api `/health`
- `cli_version.rs` (189 lines) — version checks → hive-api `/v1/cli-versions`
- `cli_http.rs` (53 lines) — HTTP utils for npm registry
- `cli_util.rs` (44 lines) — CLI path resolution
- `git_bash.rs` (155 lines) — Windows Git Bash detection

#### Storage Layer (2,400 lines — KEEP as-is)

Rock-solid file-based persistence. No database dependency.

**Architecture:**
- **Atomic writes:** `.tmp` → `.bak` → rename (3-step recovery)
- **Per-file locks:** SETTINGS, PROJECTS, SESSION, MCP, SKILLS, INDEX
- **Append-only events:** `events.jsonl` with explicit flush
- **Fast lookups:** `index.json` maps run→session, session→project

**Layout:**
```
~/.ea-code/
├── settings.json
├── projects.json
├── index.json
├── mcp.json
├── projects/{pid}/sessions/{sid}/
│   ├── session.json
│   ├── messages.jsonl
│   └── runs/{rid}/
│       ├── summary.json
│       └── events.jsonl
└── skills/{id}.json
```

**Crash recovery:** Startup scan finds orphaned "running" runs, appends synthetic `RunEnd` event.

#### Models Layer (1,100 lines — PARTIAL keep)

**Key types (keep + extend):**
- `PipelineStage` — Currently enum with 13 variants. v2: dynamic from template.
- `PipelineRun` — Keep structure, add `pipeline_template_id`.
- `AppSettings` — Remove CLI paths, add hive-api connection config.
- `RunEvent` — Keep as-is (versioned, append-only).
- `RunSummary` — Add template reference.

**Key types (remove):**
- `CliHealth`, `CliStatus`, `CliVersionInfo`, `AllCliVersions` — hive-api owns this.
- CLI path fields in `AppSettings` (claude_path, codex_path, etc.)

### 3.3 Frontend — What Exists

#### Components (4,500 lines — MIXED)

**Reusable (keep):**
- `PromptInputBar` — Generic text input + toggles
- `Toast` / `ToastProvider` — Notification system
- `FormInputs` — TextInput, NumberInput, Select, Checkbox, Toggle
- `PopoverSelect` — Dropdown
- `Sidebar` — Navigation (minor updates for pipeline selector)
- `Header`, `StatusBar` — Layout chrome
- `ProjectLoadingOverlay`, `UpdateInstallBanner` — Overlays
- `WorkspaceFooter` — Workspace path + git info
- `SkillsView` — Skills CRUD (unchanged)
- `McpView` — MCP management (unchanged)

**Redesign (pipeline-coupled):**
- `ChatView` (196 lines) — Entire view is hardcoded 13-stage pipeline
- `AgentsView` (300+ lines) — Fixed 9-stage agent assignment grid → new pipeline builder
- `CliSetupView` (200+ lines) — CLI path management → hive-api status view
- `RunTimeline` (181 lines) — Fixed iteration/stage timeline → dynamic from template
- `RichStageCard` (150 lines) — Stage-specific rendering
- `AppContentRouter` (199 lines) — 34-prop drilling → needs state management rethink

#### Hooks (1,517 lines — MIXED)

**Keep:**
- `useWorkspace` — Workspace management
- `useSettings` — Settings load/save
- `useHistory` — Session/run history
- `useSkills` — Skills CRUD
- `useMcpServers` / `useMcpRuntime` — MCP management
- `useUpdateCheck` — App updates
- `useClickOutside` — Utility
- `useElapsedTimer` — Timer formatting

**Redesign:**
- `usePipeline` (144 lines) — Pipeline lifecycle (adapt for hive-api)
- `usePipelineEvents` (196 lines) — Event subscription (adapt for SSE)
- `useAppViewState` (201 lines) — Navigation + orchestration

**Remove:**
- `useCliHealth` (61 lines) — replaced by hive-api health
- `useCliVersions` (91 lines) — replaced by hive-api version management

#### Types (800 lines)

**Keep:** `PipelineStatus`, `StageStatus`, `JudgeVerdict`, `RunEvent` variants, `SessionMeta`, `ProjectEntry`, `ChatMessage`, `Skill`, `McpServer`

**Redesign:** `PipelineStage` (dynamic from template), `AppSettings` (remove CLI paths, add pipeline templates), `PipelineRun` (add template ref)

**New types needed:** `PipelineTemplate`, `PipelineStageDefinition`, `SessionGroup`, `CustomPrompt`

### 3.4 IPC Surface (Tauri Commands — 37 total)

**Keep (22):**
- Pipeline: run, cancel, pause, resume, answer_question (5)
- Workspace: select, validate, open_in_vscode (3)
- Settings: get, save (2)
- Skills: list, get, create, update, delete (5)
- MCP: list, create, update, delete, get_config, save_config, runtime_status, fix (8+)
- History: list_projects, list_sessions, get_session_detail, create_session, delete_session, load_more_runs, get_run_detail, get_run_events (8)

**Remove (7):**
- CLI: check_cli_health, check_cli_versions, update_cli, invalidate_cli_cache, fetch_cli_versions (5)
- Git Bash: detect_git_bash (1)
- App: has_live_sessions (redesign) (1)

**New commands needed:**
- Pipeline templates: CRUD (list, get, create, update, delete)
- hive-api: start, stop, status, health
- Session groups: get/set

---

## 4. hive-api Integration

### 4.1 What Is hive-api

A **Python FastAPI** service (localhost:8000) that manages AI provider CLIs as warm drone processes. It handles:

- **6 providers:** Claude, Codex, Gemini, Kimi, Copilot, OpenCode
- **SSE streaming:** Real-time output via Server-Sent Events
- **Session resume:** Provider-specific session references for continuing conversations
- **CLI lifecycle:** Version checking, auto-updates, health monitoring
- **Windows support:** Git Bash subprocess management with `CREATE_NO_WINDOW`

### 4.2 API Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/health` | System status, drone availability |
| `GET` | `/v1/providers` | Available provider capabilities |
| `GET` | `/v1/models` | All models with status |
| `GET` | `/v1/drones` | Active drone inventory |
| `POST` | `/v1/chat` | **Core:** Submit prompt, get SSE stream |
| `POST` | `/v1/chat/{job_id}/stop` | Cancel a running job (sends Ctrl-C to drone) |
| `GET` | `/v1/cli-versions` | Cached version info |
| `POST` | `/v1/cli-versions/check` | Trigger version check |
| `POST` | `/v1/cli-versions/{provider}/check` | Check single provider |
| `POST` | `/v1/cli-versions/{provider}/update` | Update provider CLI |
| `POST` | `/v1/test/verify` | Validate test results via keyword matching |
| `POST` | `/v1/test/generate-scenario` | AI-generated test scenarios |

### 4.3 Chat Request (Core API)

```json
{
  "provider": "claude",
  "model": "opus",
  "workspace_path": "/path/to/project",
  "mode": "new",
  "prompt": "Analyse the authentication module...",
  "stream": true,
  "provider_session_ref": null,
  "provider_options": {}
}
```

**For resume:**
```json
{
  "provider": "claude",
  "model": "opus",
  "workspace_path": "/path/to/project",
  "mode": "resume",
  "prompt": "Now review your analysis for security issues...",
  "stream": true,
  "provider_session_ref": "abc-123-session-id"
}
```

**Response (non-streaming):**
```json
{
  "provider": "claude",
  "model": "opus",
  "provider_session_ref": "abc-123",
  "final_text": "...",
  "exit_code": 0,
  "warnings": [],
  "job_id": "hex-uuid"
}
```

**Error responses:**
- HTTP 400: Provider CLI not installed ("Provider 'X' is not available")
- HTTP 404: No drone for provider/model pair
- HTTP 500: CLI crash

### 4.4 SSE Event Stream (7 event types)

| Event | Payload | Purpose |
|-------|---------|---------|
| `run_started` | `{ provider, model, job_id }` | Drone acquired (capture job_id for cancellation) |
| `provider_session` | `{ provider_session_ref }` | Session ref for resume |
| `output_delta` | `{ text }` | Incremental output |
| `completed` | `{ provider, model, provider_session_ref, final_text, exit_code, warnings }` | Done |
| `failed` | `{ provider, model, provider_session_ref, exit_code, warnings, error }` | Error |
| `stopped` | `{ provider, model, job_id }` | Cancelled via POST /v1/chat/{job_id}/stop |

### 4.4.1 Job Cancellation

Cancel a running or queued job:

```
POST /v1/chat/{job_id}/stop
→ { "job_id": "abc", "status": "stopped", "provider": "claude", "model": "opus" }
```

Internally sends Ctrl-C (`\x03\n`) to the drone's bash stdin. Queued jobs are removed immediately. The `stopped` SSE event is emitted to close the stream.

**Job lifecycle:** `QUEUED → RUNNING → COMPLETED | FAILED | STOPPED`

Colony maintains a job registry (max 1,000 completed entries with LRU eviction) for status tracking.

### 4.5 Provider Session Resume

| Provider | Session Mechanism | Session Ref Format | Models |
|----------|-------------------|--------------------|--------|
| Claude | `--session-id` / `--resume` | UUID (client-generated) | opus, sonnet, haiku |
| Codex | `exec resume` | thread-id (from `thread.started`) | codex-5.3, gpt-5.4, gpt-5.4-mini |
| Gemini | `--resume <index>` via session list lookup | UUID (from `init` event) | gemini-3.1-pro-preview, gemini-3-flash-preview |
| Kimi | `--session` | opaque-string (client-generated UUID) | kimi-code/kimi-for-coding |
| Copilot | `--resume` | UUID (from `result` event) | claude-sonnet-4.6, claude-haiku-4.5, claude-opus-4.6, gpt-5.4, gpt-5.3-codex, gpt-5.4-mini |
| OpenCode | `--session` | opaque-string (from JSON events) | glm-5, glm-5-turbo, glm-4.7 |

**Resume validation:** Drone checks that the session ref was created with the same model. Cross-model resume is rejected with `ShellSessionError`.

### 4.6 Configuration

```toml
[server]
host = "127.0.0.1"
port = 8000

[colony]
boot_timeout_sec = 120
job_timeout_sec = 600

[providers.claude]
enabled = true
models = ["opus", "sonnet", "haiku"]
default_model = "sonnet"

[providers.codex]
enabled = true
models = ["codex-5.3", "gpt-5.4"]

# ... similar for gemini, kimi, copilot, opencode

[updater]
check_interval_sec = 3600
auto_update = true
```

### 4.7 Integration Architecture

```
┌──────────────────────────┐      HTTP/SSE        ┌─────────────────────┐
│   EA Code v2 (Tauri)     │ ──────────────────▶   │    hive-api          │
│                          │                       │    (FastAPI)         │
│  Orchestrator            │  POST /v1/chat        │                     │
│  ├── Stage 1 (Analyse)   │ ◀── SSE stream ─────  │  Colony              │
│  ├── Stage 2 (Review)    │                       │  ├── Claude Drone    │
│  │   (resume session)    │  session_ref passed   │  ├── Codex Drone     │
│  ├── Stage 3 (Implement) │  back in next request │  ├── Gemini Drone    │
│  └── Stage 4 (Test)      │                       │  ├── Kimi Drone      │
│                          │                       │  ├── Copilot Drone   │
│  Storage (~/.ea-code/)   │                       │  └── OpenCode Drone  │
│  Frontend (React)        │                       │                     │
└──────────────────────────┘                       └─────────────────────┘
```

**hive-api lifecycle managed by Tauri:**
1. On app start: spawn `uvicorn hive_api.main:app --host 127.0.0.1 --port <port>` as child process
2. Wait for `GET /health` to return `drones_booted: true`
3. During run: cancel via `POST /v1/chat/{job_id}/stop` (sends Ctrl-C to drone)
4. On app close: send SIGTERM, wait for graceful shutdown (lifespan handler cleans up drones)

**Environment variables for hive-api:**
- `HIVE_API_CONFIG` — path to config.toml
- `HIVE_API_HOST` — bind address (default: 127.0.0.1)
- `HIVE_API_PORT` — port (default: 8000)

**Windows note:** hive-api requires Git Bash. It detects `C:\Program Files\Git\bin\bash.exe` automatically. Same requirement as ea-code v1's agent layer — but now hive-api owns it.

---

## 5. v2 Architecture Overview

### 5.1 Layer Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND (React 19 + Tailwind v4)        │
│                                                                  │
│  Pipeline Builder UI    Chat View    History    Settings         │
│  (drag-and-drop)       (SSE live)   (sessions) (templates)      │
│                                                                  │
├─────────────────────── Tauri IPC ────────────────────────────────┤
│                                                                  │
│                     BACKEND (Rust / Tauri v2)                    │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │  Orchestrator │  │  Storage     │  │  hive-api Client       │ │
│  │  (dynamic     │  │  (file-based │  │  (reqwest + SSE)       │ │
│  │   pipeline    │  │   atomic     │  │                        │ │
│  │   from        │  │   writes)    │  │  POST /v1/chat         │ │
│  │   template)   │  │              │  │  session_ref tracking  │ │
│  └──────────────┘  └──────────────┘  └────────────────────────┘ │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│                     hive-api (Python / FastAPI)                   │
│                                                                  │
│  Colony → Drones (warm bash processes per provider+model)        │
│  SSE streaming, session resume, CLI version management           │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 5.2 Key Design Principles

1. **Pipeline as data, not code** — Stage order, agent assignments, and prompts live in JSON templates, not compiled Rust.
2. **Session groups for continuity** — Stages sharing a session group resume from each other via hive-api session refs.
3. **hive-api owns CLI complexity** — No process spawning, no Git Bash workarounds, no version management in the Tauri app.
4. **Storage patterns preserved** — Atomic writes, per-file locks, append-only events, index lookups all carry forward.
5. **Frontend state management** — Reduce prop drilling with a lightweight state solution.

---

## 6. Configurable Pipelines

### 6.1 Data Model

```
PipelineTemplate
├── id: uuid
├── name: string                     "Full Review Loop"
├── description: string              "Complete analysis, review, implementation and test"
├── is_builtin: bool                 true = shipped with app, not editable
├── max_iterations: u32              5
├── stop_on_first_pass: bool         true = stop iterating if tests pass
├── created_at: string (RFC 3339)
├── updated_at: string (RFC 3339)
└── stages: Vec<StageDefinition>     ordered list

StageDefinition
├── id: uuid
├── label: string                    "Security Review"
├── stage_type: string               "analyse" | "review" | "implement" | "test" | "custom"
├── position: u32                    0, 1, 2, 3... (drag-drop changes this)
├── provider: string                 "claude" | "gemini" | "codex" | ...
├── model: string                    "opus" | "sonnet" | "flash" | ...
├── session_group: string            "A" | "B" | ... (same group = resume)
├── prompt_template: string          the full prompt text with {{variables}}
├── enabled: bool                    toggle without deleting
└── execution_intent: string         "text" (read-only) | "code" (writes files)
```

### 6.2 Built-in Pipeline Templates

**1. Full Review Loop** (default, max 5 iterations)
```
[Analyse] ──resume──▶ [Review] ──new session──▶ [Implement] ──resume──▶ [Test]
 Claude Opus            Claude Opus               Claude Sonnet           Claude Sonnet
 group: "A"             group: "A"                group: "B"              group: "B"
 intent: text           intent: text              intent: code            intent: code
```

**2. Quick Fix** (1 iteration)
```
[Implement] ──resume──▶ [Test]
 Claude Sonnet           Claude Sonnet
 group: "A"              group: "A"
 intent: code            intent: code
```

**3. Research Only** (1 iteration)
```
[Analyse] ──resume──▶ [Review]
 Claude Opus            Claude Opus
 group: "A"             group: "A"
 intent: text           intent: text
```

**4. Multi-Brain Review** (3 iterations)
```
[Analyse] ──▶ [Review] ──▶ [Review] ──▶ [Implement] ──resume──▶ [Test]
 Claude Opus   Gemini Pro   Codex        Claude Sonnet            Claude Sonnet
 group: "A"    group: "B"   group: "C"   group: "D"               group: "D"
 (3 independent perspectives, then implementation)
```

**5. Security Audit** (2 iterations)
```
[Analyse] ──resume──▶ [Security Review] ──resume──▶ [Review] ──▶ [Implement] ──resume──▶ [Test]
 Claude Opus            Claude Opus (custom)          Claude Opus   Claude Sonnet           Claude Sonnet
 group: "A"             group: "A"                    group: "A"    group: "B"              group: "B"
```

### 6.3 Storage

```
~/.ea-code/
├── pipeline-templates/
│   ├── full-review-loop.json      (built-in)
│   ├── quick-fix.json             (built-in)
│   ├── research-only.json         (built-in)
│   ├── multi-brain-review.json    (built-in)
│   ├── security-audit.json        (built-in)
│   └── {user-template-id}.json    (user-created)
```

Uses same atomic write + file lock pattern as existing storage modules.

### 6.4 User Flow

1. **New user:** Sees template gallery, picks "Full Review Loop", it just works.
2. **Customise:** "Use as Template" clones a built-in → user edits stages, prompts, models.
3. **Power user:** "+ New Pipeline" → adds stages from scratch, writes custom prompts.
4. **Running:** Pipeline selector dropdown on main view, pick which pipeline to run.

---

## 7. Session Resume & Session Groups

### 7.1 The Problem With v1

Every stage spawns a **fresh** CLI process with zero memory:

```
v1: Analyse (cold) → Review (cold, re-reads everything) → Implement (cold, re-reads everything)
```

### 7.2 v2 Solution — Session Groups

Stages with the same `session_group` value share a hive-api session:

```
v2: Analyse (new session "A") → Review (resume "A", full context) → Implement (new session "B")
```

**The `provider_session_ref` flow:**

1. Stage 1 (Analyse) sends `POST /v1/chat` with `mode: "new"`
2. hive-api returns `provider_session` SSE event with `provider_session_ref: "abc-123"`
3. Orchestrator stores `session_refs["A"] = "abc-123"`
4. Stage 2 (Review) sends `POST /v1/chat` with `mode: "resume"`, `provider_session_ref: "abc-123"`
5. The agent continues the conversation — full context preserved

### 7.3 Session Group Rules

- Same `session_group` + same `provider` + same `model` = **resume** (pass session ref)
- Same `session_group` but different provider/model = **new session** (can't resume across providers)
- Different `session_group` = **new session** (output passed as prompt context, not resumed)

### 7.4 Design Decision: Analyse + Review Share, Implement Separate

| Group | Stages | Rationale |
|-------|--------|-----------|
| "A" (Understanding) | Analyse + Review | Agent builds deep understanding, then refines it. No cognitive load of writing code. |
| "B" (Execution) | Implement + Test | Agent gets a clean brief from the analysis group. Can use a cheaper/faster model. |

**Benefits:**
- The analyst stays objective (thinking only, not coding)
- The implementer gets a polished brief, not a stream of consciousness
- If implementation fails, the analysis session is not polluted
- Cost optimisation: Opus for thinking, Sonnet for coding
- On new iterations: the analysis group can resume to remember what failed

### 7.5 Data Flow

```
Orchestrator maintains per-run:

  session_refs: HashMap<String, String>
  // Maps session_group → provider_session_ref

  For each stage in pipeline:
    1. Look up stage.session_group in session_refs
    2. If found → mode: "resume", pass ref
    3. If not found → mode: "new"
    4. On provider_session SSE event → store ref in session_refs
```

---

## 8. Custom Prompts & Template Variables

### 8.1 Template Variables

Injected by the orchestrator at runtime:

| Variable | Source | Available |
|----------|--------|-----------|
| `{{task}}` | User's original prompt | Always |
| `{{workspace_path}}` | Absolute project path | Always |
| `{{file_list}}` | Files in workspace | Always |
| `{{code_context}}` | Relevant file contents | Always |
| `{{previous_output}}` | Output from prior stage | Position > 0 |
| `{{iteration_number}}` | Current loop iteration | Always |
| `{{max_iterations}}` | Pipeline iteration limit | Always |
| `{{test_results}}` | Test output from last run | Iteration > 1 |
| `{{judge_feedback}}` | Judge's reasoning (if looping) | Iteration > 1 |
| `{{git_branch}}` | Current git branch | If git repo |
| `{{git_diff}}` | Working tree changes | If git repo |

### 8.2 Example Custom Prompt

```
You are a senior security engineer performing a thorough code audit.

Task: {{task}}
Workspace: {{workspace_path}}
Files: {{file_list}}

Analyse the following for:
1. SQL injection and parameterised query validation
2. Cross-site scripting (XSS) - both reflected and stored
3. Authentication bypass risks
4. Secrets or credentials in code
5. OWASP Top 10 vulnerabilities

{{#if previous_output}}
Previous analysis findings:
{{previous_output}}
{{/if}}

Provide findings as:
- CRITICAL: [issue]
- WARNING: [issue]
- INFO: [observation]
```

### 8.3 Prompt Enhance Button

When the user writes a rough prompt and clicks "Enhance":

1. EA Code sends the draft prompt to hive-api `/v1/chat` with a cheap model (Sonnet/Flash)
2. Meta-prompt instructs the AI to improve clarity, add structure, use template variables
3. Shows enhanced version in a **diff view** (before/after)
4. User can Accept, Edit further, or Reject

**Enhance meta-prompt:**
```
You are a prompt engineer. Improve this prompt for an AI coding agent.
Make it clearer, more structured, and more effective.
Preserve the user's intent exactly. Add specificity where vague.
Use available template variables: {{task}}, {{code_context}}, {{previous_output}},
{{file_list}}, {{iteration_number}}, {{test_results}}.
Return ONLY the improved prompt.
```

### 8.4 Storage

Custom prompts are stored inline in the `StageDefinition.prompt_template` field within each pipeline template JSON. No separate prompt files needed.

---

## 9. Data Model & Storage

### 9.1 New Storage Files

```
~/.ea-code/
├── settings.json                    # MODIFIED: remove CLI paths, add hive-api config
├── projects.json                    # UNCHANGED
├── index.json                       # UNCHANGED
├── mcp.json                         # UNCHANGED
│
├── pipeline-templates/              # NEW
│   ├── full-review-loop.json
│   ├── quick-fix.json
│   ├── research-only.json
│   └── {user-template-id}.json
│
├── projects/                        # UNCHANGED structure
│   └── {pid}/sessions/{sid}/
│       ├── session.json
│       ├── messages.jsonl
│       └── runs/{rid}/
│           ├── summary.json         # MODIFIED: add pipeline_template_id, session_refs
│           └── events.jsonl         # MODIFIED: new event types for dynamic stages
│
├── skills/{id}.json                 # UNCHANGED
└── hive-api/                        # NEW: bundled hive-api
    ├── config.toml
    └── ...
```

### 9.2 New Rust Types

```rust
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_builtin: bool,
    pub max_iterations: u32,
    pub stop_on_first_pass: bool,
    pub stages: Vec<StageDefinition>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StageDefinition {
    pub id: String,
    pub label: String,
    pub stage_type: String,
    pub position: u32,
    pub provider: String,
    pub model: String,
    pub session_group: String,
    pub prompt_template: String,
    pub enabled: bool,
    pub execution_intent: String, // "text" or "code"
}
```

### 9.3 New TypeScript Types

```typescript
export interface PipelineTemplate {
  id: string;
  name: string;
  description: string;
  isBuiltin: boolean;
  maxIterations: number;
  stopOnFirstPass: boolean;
  stages: StageDefinition[];
  createdAt: string;
  updatedAt: string;
}

export interface StageDefinition {
  id: string;
  label: string;
  stageType: string;
  position: number;
  provider: string;
  model: string;
  sessionGroup: string;
  promptTemplate: string;
  enabled: boolean;
  executionIntent: "text" | "code";
}
```

### 9.4 Modified RunSummary

```rust
pub struct RunSummary {
    // ... existing fields ...
    pub pipeline_template_id: Option<String>,  // NEW: which template was used
    pub pipeline_template_name: Option<String>, // NEW: snapshot of template name
    pub session_refs: HashMap<String, String>,  // NEW: group → provider_session_ref
}
```

### 9.5 Modified AppSettings

```rust
pub struct AppSettings {
    // REMOVED: claude_path, codex_path, gemini_path, kimi_path, opencode_path
    // REMOVED: per-stage agent/model assignments (moved to pipeline templates)

    // KEPT:
    pub max_iterations: u32,           // global default (templates can override)
    pub require_git: bool,
    pub require_plan_approval: bool,
    pub plan_auto_approve_timeout_sec: u32,
    pub retention_days: u32,
    pub agent_retry_count: u32,
    pub agent_timeout_ms: u64,
    pub agent_max_turns: u32,

    // NEW:
    pub hive_api_host: String,         // default: "127.0.0.1"
    pub hive_api_port: u16,            // default: 8000
    pub default_pipeline_id: String,   // which template to use by default
    pub auto_start_hive_api: bool,     // start hive-api on app launch
}
```

---

## 10. Frontend Architecture

### 10.1 New View Structure

```
Views:
├── Home (IdleView)           — Workspace selector + prompt input + pipeline selector
├── Chat (ChatView)           — Live pipeline execution (dynamic stages from template)
├── Session Detail            — Past run history
├── Pipeline Builder          — NEW: drag-and-drop pipeline configuration
├── Pipeline Gallery          — NEW: browse/clone built-in templates
├── Skills                    — Skills catalogue (unchanged)
├── MCP Servers               — MCP management (unchanged)
├── Settings                  — App settings (simplified, no CLI paths)
└── hive-api Status           — NEW: provider status, drone health
```

### 10.2 Pipeline Builder UI

```
┌─────────────────────────────────────────────────────────────────────┐
│  Pipeline Settings                                                   │
├──────────────────────┬──────────────────────────────────────────────┤
│                      │                                               │
│  TEMPLATES           │  ── Full Review Loop ─────────────────────   │
│  ┌────────────────┐  │                                               │
│  │ Full Review    │  │  Description: [ Complete analysis, review,  ] │
│  │ Quick Fix      │  │               [ implementation and test     ] │
│  │ Research Only  │  │                                               │
│  │ Multi-Brain    │  │  Max Iterations: [5]  [ ] Stop on first pass │
│  │ Security Audit │  │                                               │
│  └────────────────┘  │  Stages:                        [+ Add Stage] │
│  [Use as Template]   │                                               │
│                      │  :: ┌───────────────────────────────────────┐ │
│  MY PIPELINES        │     │ 1. Analyse       (A)  Claude . Opus  │ │
│  ┌────────────────┐  │     │ [Edit Prompt]            [on]   [x]  │ │
│  │ My Custom      │  │     └───────────────────────────────────────┘ │
│  │ PR Review      │  │  :: ┌───────────────────────────────────────┐ │
│  └────────────────┘  │     │ 2. Review        (A)  Claude . Opus  │ │
│  [+ New Pipeline]    │     │ [Edit Prompt]            [on]   [x]  │ │
│                      │     └───────────────────────────────────────┘ │
│                      │            - - - session break - - -          │
│                      │  :: ┌───────────────────────────────────────┐ │
│                      │     │ 3. Implement     (B)  Claude . Sonnet│ │
│                      │     │ [Edit Prompt]            [on]   [x]  │ │
│                      │     └───────────────────────────────────────┘ │
│                      │  :: ┌───────────────────────────────────────┐ │
│                      │     │ 4. Test          (B)  Claude . Sonnet│ │
│                      │     │ [Edit Prompt]            [on]   [x]  │ │
│                      │     └───────────────────────────────────────┘ │
│                      │                                               │
│                      │  [+ Add Stage]                                │
│                      │                                               │
│                      │         [Delete Pipeline]    [Save Changes]   │
└──────────────────────┴──────────────────────────────────────────────┘
```

**Key interactions:**
- `::` = drag handle (reorder stages)
- `(A)` / `(B)` = session group indicator (colour coded)
- `[Edit Prompt]` = opens prompt editor with template variables + Enhance button
- `[on]` = toggle stage enabled/disabled
- `[x]` = delete stage
- Session break line = automatically shown between different session groups

### 10.3 Session Group UX

**Default mode (simple):** Each stage has a toggle "Resume from previous?" If yes, joins the previous stage's group. If no, starts new group. Groups auto-calculated.

**Advanced mode:** User picks explicit group names (A, B, C...) for full control.

### 10.4 Prompt Editor

```
┌──────────────────────────────────────────────────────────────┐
│  Stage: Review                                                │
│                                                               │
│  Label:    [ Security Review          ]                       │
│  Type:     [ Custom         v]                                │
│  Provider: [ Claude         v]  Model: [ Opus          v]    │
│  Session:  ( ) Resume from previous  (o) Start new session   │
│                                                               │
│  --- Prompt Template ---                                      │
│                                                               │
│  Variables: {{task}} {{code_context}} {{previous_output}}     │
│             {{file_list}} {{iteration_number}} {{test_results}}│
│                                                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ You are a senior security engineer reviewing code      │  │
│  │ changes for {{task}}.                                  │  │
│  │                                                        │  │
│  │ Previous analysis: {{previous_output}}                 │  │
│  │                                                        │  │
│  │ Focus on OWASP Top 10 vulnerabilities...               │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
│  [ Enhance ]  Enhance with: [Claude Sonnet v]                │
│                                                               │
│           [Cancel]                        [Save Stage]        │
└──────────────────────────────────────────────────────────────┘
```

### 10.5 New Hooks

```typescript
// Pipeline template CRUD
usePipelineTemplates()
  → { templates, createTemplate, updateTemplate, deleteTemplate, cloneTemplate }

// hive-api connection management
useHiveApi()
  → { status, providers, models, drones, startApi, stopApi, checkHealth }

// Dynamic pipeline execution (replaces usePipeline + usePipelineEvents)
usePipelineExecution()
  → { run, startPipeline, pausePipeline, resumePipeline, cancelPipeline }
  // Internally consumes SSE from hive-api via orchestrator
```

### 10.6 State Management

v1 suffers from 34-prop drilling through `AppContentRouter`. v2 options:

**Recommended: React Context + useReducer**
- `PipelineContext` — active run, stage status, logs
- `AppContext` — active view, workspace, settings
- `TemplateContext` — pipeline templates, active template

Lightweight, no new dependencies, integrates naturally with existing hook pattern.

---

## 11. Backend Architecture

### 11.1 New Module Structure

```
src-tauri/src/
├── main.rs                          # Unchanged
├── lib.rs                           # Updated: register new commands
├── events.rs                        # Updated: new event types
├── git.rs                           # Unchanged
│
├── hive_client/                     # NEW: HTTP client for hive-api
│   ├── mod.rs                       # HiveClient struct, connection management
│   ├── chat.rs                      # POST /v1/chat, SSE stream consumption
│   ├── providers.rs                 # GET /v1/providers, /v1/models, /v1/drones
│   ├── health.rs                    # GET /health
│   └── versions.rs                  # CLI version management endpoints
│
├── orchestrator/                    # REDESIGNED: dynamic pipeline execution
│   ├── pipeline.rs                  # run_pipeline() reads from template
│   ├── stage_runner.rs              # Execute single stage via hive_client
│   ├── session_manager.rs           # Track session_refs per group
│   ├── prompt_renderer.rs           # Inject template variables into prompts
│   ├── iteration.rs                 # Iteration loop logic
│   ├── parallel.rs                  # Parallel stage execution
│   ├── user_questions.rs            # Mid-run Q&A (kept)
│   ├── plan_gate.rs                 # Plan approval (kept)
│   └── helpers.rs                   # Event emission, cancellation
│
├── commands/                        # UPDATED
│   ├── mod.rs                       # AppState (kept)
│   ├── pipeline.rs                  # run/cancel/pause/resume (updated)
│   ├── templates.rs                 # NEW: pipeline template CRUD
│   ├── hive_api.rs                  # NEW: start/stop/status hive-api
│   ├── workspace.rs                 # Unchanged
│   ├── settings.rs                  # Unchanged
│   ├── skills.rs                    # Unchanged
│   ├── mcp.rs                       # Unchanged
│   └── history.rs                   # Unchanged
│
├── models/                          # UPDATED
│   ├── pipeline.rs                  # Dynamic stages from templates
│   ├── templates.rs                 # NEW: PipelineTemplate, StageDefinition
│   ├── settings.rs                  # Simplified (no CLI paths)
│   ├── storage.rs                   # Updated (new RunSummary fields)
│   ├── events.rs                    # Updated (dynamic stage names)
│   ├── agents.rs                    # Simplified (provider enum only)
│   ├── questions.rs                 # Unchanged
│   ├── skills.rs                    # Unchanged
│   └── mcp.rs                       # Unchanged
│
├── storage/                         # EXTENDED
│   ├── mod.rs                       # Unchanged (atomic writes, locks)
│   ├── templates.rs                 # NEW: pipeline template persistence
│   ├── projects.rs                  # Unchanged
│   ├── sessions.rs                  # Unchanged
│   ├── runs/                        # Updated (new summary fields)
│   ├── settings.rs                  # Updated (new AppSettings shape)
│   ├── skills.rs                    # Unchanged
│   ├── mcp.rs                       # Unchanged
│   ├── messages.rs                  # Unchanged
│   ├── index.rs                     # Unchanged
│   ├── migration.rs                 # Updated (v1 → v2 settings migration)
│   ├── recovery.rs                  # Unchanged
│   └── cleanup.rs                   # Unchanged
│
├── agents/                          # DELETED (replaced by hive_client/)
└── swarm/                           # Reserved (future)
```

### 11.2 hive_client Module — The Core Replacement

```rust
// hive_client/chat.rs — Core agent invocation

pub struct ChatRequest {
    pub provider: String,
    pub model: String,
    pub workspace_path: String,
    pub mode: String,              // "new" or "resume"
    pub prompt: String,
    pub stream: bool,
    pub provider_session_ref: Option<String>,
    pub provider_options: HashMap<String, Value>,
}

pub enum SseEvent {
    RunStarted { provider: String, model: String },
    ProviderSession { provider_session_ref: String },
    OutputDelta { text: String },
    Completed { final_text: String, exit_code: i32, session_ref: Option<String> },
    Failed { error: String, exit_code: i32 },
}

/// Sends a chat request to hive-api and streams SSE events back.
/// Emits pipeline:log events for each output_delta.
/// Returns the final output text and session ref.
pub async fn run_hive_chat(
    client: &reqwest::Client,
    base_url: &str,
    request: ChatRequest,
    app_handle: &AppHandle,
    run_id: &str,
    stage_label: &str,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(String, Option<String>), String> {
    // POST /v1/chat
    // Consume SSE stream
    // Emit pipeline:log for each output_delta
    // Return (final_text, provider_session_ref)
}
```

### 11.3 New Orchestrator Flow

```rust
pub async fn run_pipeline(
    template: &PipelineTemplate,
    prompt: &str,
    workspace_path: &str,
    app_handle: &AppHandle,
    // ... flags, channels
) -> Result<(), String> {
    let mut session_refs: HashMap<String, String> = HashMap::new();
    let enabled_stages: Vec<&StageDefinition> = template.stages
        .iter()
        .filter(|s| s.enabled)
        .collect();

    for iteration in 1..=template.max_iterations {
        for stage in &enabled_stages {
            // 1. Render prompt template with variables
            let rendered_prompt = render_prompt(
                &stage.prompt_template,
                prompt, workspace_path, &previous_output,
                iteration, template.max_iterations,
            );

            // 2. Determine resume mode
            let (mode, session_ref) = if let Some(ref_id) = session_refs.get(&stage.session_group) {
                ("resume", Some(ref_id.clone()))
            } else {
                ("new", None)
            };

            // 3. Call hive-api
            let (output, new_ref) = run_hive_chat(
                &client, &base_url,
                ChatRequest {
                    provider: stage.provider.clone(),
                    model: stage.model.clone(),
                    workspace_path: workspace_path.to_string(),
                    mode: mode.to_string(),
                    prompt: rendered_prompt,
                    stream: true,
                    provider_session_ref: session_ref,
                    provider_options: HashMap::new(),
                },
                app_handle, run_id, &stage.label, cancel_flag.clone(),
            ).await?;

            // 4. Store session ref for group
            if let Some(ref_id) = new_ref {
                session_refs.insert(stage.session_group.clone(), ref_id);
            }

            previous_output = output;
        }

        // Judge / iteration logic...
    }
}
```

---

## 12. Migration Strategy — What to Keep, Remove, Rewrite

### 12.1 Summary Table

| Module | Action | Lines | Notes |
|--------|--------|-------|-------|
| **agents/** | DELETE | 433 | Replaced by hive_client/ |
| **commands/cli*.rs, git_bash.rs** | DELETE | 719 | hive-api owns CLI management |
| **orchestrator/** | REWRITE | 2,200 | Same concepts, dynamic pipeline from template |
| **orchestrator/prompts/** | MIGRATE | 1,300 | Move to built-in template defaults |
| **hive_client/** | NEW | ~400 | HTTP client + SSE consumer |
| **storage/** | KEEP + EXTEND | 2,400 | Add templates.rs, update settings.rs |
| **models/** | UPDATE | 1,100 | Add template types, remove CLI types |
| **commands/ (core)** | KEEP + UPDATE | 881 | Add template/hive commands |
| **events.rs** | UPDATE | 14 | Support dynamic stage names |
| **git.rs** | KEEP | 76 | Unchanged |
| **Frontend shared/** | KEEP | ~1,200 | Toast, FormInputs, PopoverSelect, etc. |
| **Frontend views/** | REWRITE | ~2,000 | Pipeline builder, dynamic ChatView |
| **Frontend hooks/** | PARTIAL | ~1,500 | New pipeline/template hooks |
| **Frontend types/** | UPDATE | ~800 | Add template types, remove CLI types |

### 12.2 Lines Removed vs Added

| | Lines |
|---|---|
| Removed (CLI layer, old orchestrator, old views) | ~7,500 |
| Kept (storage, core commands, shared UI, utils) | ~10,000 |
| New (hive_client, templates, pipeline builder, new hooks) | ~5,000 |
| **Net v2 estimate** | **~15,000** |

### 12.3 Prompt Migration

The 11 prompt template files in `orchestrator/prompts/` (1,300 lines) become the **default prompt_template** values in built-in pipeline templates:

| v1 Prompt File | v2 Location |
|----------------|-------------|
| `enhancer.rs` | `full-review-loop.json → stages[0].prompt_template` |
| `planner.rs` | `full-review-loop.json → stages[1].prompt_template` |
| `plan_auditor.rs` | (built into plan gate logic, not a separate stage) |
| `generator.rs` | `full-review-loop.json → stages[2].prompt_template` |
| `reviewer.rs` | `full-review-loop.json → stages[3].prompt_template` |
| `fixer.rs` | (merged into implement stage prompt) |
| `judge.rs` | (built into iteration logic) |
| `executive_summary.rs` | (optional final stage) |

### 12.4 Settings Migration

On first v2 launch, migrate `settings.json`:

1. Read v1 settings
2. Create default pipeline template from v1 agent/model assignments
3. Remove CLI path fields
4. Add hive-api connection fields
5. Write v2 settings with incremented schema version

---

## 13. Implementation Roadmap

### Phase 1: Foundation (hive-api + Client)

1. Pull hive-api into project as `backend/hive-api/`
2. Create `hive_client/` Rust module (reqwest + SSE)
3. Add hive-api lifecycle management (start/stop as child process)
4. Verify basic chat round-trip: Tauri → hive-api → Claude → SSE → Tauri

### Phase 2: Pipeline Templates

1. Define `PipelineTemplate` and `StageDefinition` types (Rust + TS)
2. Create `storage/templates.rs` with CRUD + atomic writes
3. Ship 5 built-in templates with default prompts
4. Add Tauri commands for template CRUD

### Phase 3: Dynamic Orchestrator

1. Rewrite `run_pipeline()` to read stages from template
2. Implement session group tracking (`session_refs` map)
3. Implement prompt template rendering with `{{variables}}`
4. Wire up iteration logic with dynamic stage lists
5. Preserve cancel/pause/resume/question flows

### Phase 4: Frontend — Pipeline Builder

1. Pipeline template list view (gallery + user templates)
2. Drag-and-drop stage editor (reorder, add, remove)
3. Per-stage configuration (provider, model, session group)
4. Prompt editor with template variables + Enhance button
5. Pipeline selector on main view

### Phase 5: Frontend — Execution View

1. Dynamic ChatView (renders stages from template, not hardcoded)
2. Session group visualisation (colour-coded groups)
3. hive-api status indicator (replaces CLI health)
4. Updated RunTimeline for dynamic stages

### Phase 6: Migration & Polish

1. v1 → v2 settings migration
2. v1 prompt templates → built-in pipeline defaults
3. Remove all CLI-specific code
4. End-to-end testing
5. Update CLAUDE.md and documentation

---

## Appendix A: v1 File Inventory (Complete)

### Rust Backend (14,575 lines)

```
src-tauri/src/
├── main.rs (7)
├── lib.rs (147)
├── events.rs (14)
├── git.rs (76)
├── agents/
│   ├── mod.rs (14)
│   ├── base/mod.rs (303)
│   ├── base/windows.rs (113)
│   ├── claude.rs (148)
│   ├── codex.rs (60)
│   ├── gemini.rs (50)
│   ├── kimi.rs (138)
│   ├── opencode.rs (42)
│   └── mcp.rs (72)
├── orchestrator/
│   ├── pipeline/mod.rs (233)
│   ├── pipeline/direct_task.rs (63)
│   ├── iteration/mod.rs (192)
│   ├── iteration/generate.rs (199)
│   ├── iteration/prompt_enhance.rs (76)
│   ├── iteration/stages.rs (67)
│   ├── iteration_planning/mod.rs (219)
│   ├── iteration_planning/persistence.rs (79)
│   ├── iteration_review/mod.rs (236)
│   ├── iteration_review/judge.rs (75)
│   ├── iteration_review/stages.rs (101)
│   ├── stages/mod.rs (127)
│   ├── stages/execution.rs (106)
│   ├── run_setup/mod.rs (157)
│   ├── run_setup/persistence.rs (83)
│   ├── parsing/mod.rs (89)
│   ├── parsing/plan.rs (78)
│   ├── parsing/reviewer.rs (127)
│   ├── helpers.rs (433)
│   ├── parallel_stage.rs (105)
│   ├── context_summary.rs (228)
│   ├── session_memory.rs (160)
│   ├── skill_selection.rs (123)
│   ├── skill_stage.rs (234)
│   ├── user_questions.rs (190)
│   ├── plan_gate.rs (286)
│   └── prompts/ (11 files, ~1,300)
├── commands/
│   ├── mod.rs (36)
│   ├── pipeline.rs (197)
│   ├── app.rs (14)
│   ├── workspace.rs (48)
│   ├── settings.rs (14)
│   ├── skills.rs (90)
│   ├── mcp.rs (246)
│   ├── mcp_runtime/ (4 files, ~429)
│   ├── cli.rs (278)
│   ├── cli_version.rs (189)
│   ├── cli_http.rs (53)
│   ├── cli_util.rs (44)
│   ├── git_bash.rs (155)
│   └── history.rs (119)
├── models/ (11 files, ~1,100)
└── storage/ (12 files + runs/, ~2,400)
```

### React Frontend (10,100 lines, v0.6.3)

```
src/
├── main.tsx (14)
├── App.tsx (174)
├── index.css (106)
├── components/
│   ├── AppContentRouter.tsx (201)
│   ├── ChatView.tsx (405)
│   ├── IdleView.tsx (173)
│   ├── SessionDetailView.tsx (289)
│   ├── Sidebar.tsx (145)
│   ├── SidebarHome.tsx (108)
│   ├── SidebarSettings.tsx (79)
│   ├── SidebarCollapsed.tsx (59)
│   ├── StatusBar.tsx (119)
│   ├── ProjectThreadsList.tsx (195)
│   ├── QuestionDialog.tsx (108)
│   ├── RunCard.tsx (340)
│   ├── RunTimeline.tsx (186)
│   ├── SkillsView.tsx (244)
│   ├── AgentsView/ (6 files, ~752)
│   ├── CliSetupView/ (2 files, ~393)
│   ├── McpView/ (3 files, ~428)
│   └── shared/ (23 files, ~2,218)
│       ├── TabbedParallelStageCard.tsx (313)
│       ├── ResultCard.tsx (207)
│       ├── PromptInputBar.tsx (197)
│       ├── StageInputOutputCard.tsx (138)
│       ├── ReviewerProgressRow.tsx (109)
│       ├── PlannerProgressRow.tsx (109)
│       ├── FormInputs.tsx (99)
│       ├── Toast.tsx (97)
│       ├── PromptCard.tsx (94)
│       ├── StageCard.tsx (93)
│       ├── RichStageCard.tsx (86)
│       ├── RecentTerminalPanel.tsx (79)
│       ├── PipelineControlBar.tsx (77)
│       ├── PopoverSelect.tsx (75)
│       ├── FinalPlanCard.tsx (71)
│       ├── TabbedReviewCard.tsx (69)
│       ├── TabbedPlanCard.tsx (69)
│       ├── ThinkingIndicator.tsx (51)
│       ├── ArtifactCard.tsx (44)
│       ├── PromptReceivedCard.tsx (41)
│       ├── WorkspaceFooter.tsx (33)
│       ├── AssistantMessageBubble.tsx (30)
│       ├── UpdateInstallBanner.tsx (19)
│       └── ProjectLoadingOverlay.tsx (18)
├── hooks/ (17 files, ~1,771)
├── types/ (10 files, ~939)
└── utils/ (6 files, ~684)
```

---

## Appendix B: hive-api API Reference

### POST /v1/chat

**Request:**
```json
{
  "provider": "claude | codex | gemini | kimi | copilot | opencode",
  "model": "string",
  "workspace_path": "/absolute/path",
  "mode": "new | resume",
  "prompt": "string",
  "stream": true,
  "provider_session_ref": "string | null",
  "provider_options": {}
}
```

**SSE Events:**
```
event: run_started
data: {"provider": "claude", "model": "opus"}

event: provider_session
data: {"provider_session_ref": "abc-123"}

event: output_delta
data: {"text": "incremental output..."}

event: completed
data: {"final_text": "...", "exit_code": 0, "session_ref": "abc-123", "warnings": []}

event: failed
data: {"error": "message", "exit_code": 1, "warnings": []}
```

### GET /health

```json
{
  "status": "healthy",
  "shell_available": true,
  "drones_booted": 6,
  "drones_total": 8
}
```

### GET /v1/providers

```json
[
  {
    "name": "claude",
    "available": true,
    "models": ["opus", "sonnet", "haiku"],
    "supports_resume": true
  }
]
```

### GET /v1/drones

```json
[
  {
    "provider": "claude",
    "model": "opus",
    "status": "idle",
    "queue_depth": 0
  }
]
```

---

*Document generated from comprehensive analysis of ea-code v1 codebase (14,575 Rust + 8,095 TypeScript lines) and hive-api project.*
