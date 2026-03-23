# ADR 003 — hive-api Bundling Strategy

| Field   | Value                        |
|---------|------------------------------|
| Status  | **Pending**                  |
| Date    | 2026-03-23                   |
| Scope   | Build, Packaging, Desktop UX |

## Context

hive-api is a Python FastAPI service that provides the unified provider interface for v2 pipeline execution. The Tauri desktop app is a Rust-based application distributed as a native installer (NSIS on Windows, DMG on macOS).

Shipping a Python service alongside a Rust desktop app is a packaging problem with no single obvious solution. The choice affects install size, startup time, cross-platform support, and the user's first-run experience.

## Options Under Evaluation

### Option 1 — Sidecar Process (Recommended)

Bundle a minimal Python environment (e.g., `python-build-standalone` or embedded Python) with the Tauri app. On startup, Tauri spawns `uvicorn` as a child process managed via the Tauri sidecar API.

**Advantages:**
- Simplest implementation. Tauri's sidecar API handles process lifecycle.
- Fast iteration during development — change Python files, restart.
- Python ecosystem fully available (pip packages, venvs).

**Disadvantages:**
- Adds ~40-80 MB to the installer for the bundled Python runtime.
- Python version management across platforms.
- Must handle sidecar crash recovery and port conflicts.

**Implementation sketch:**
1. Bundle `python-build-standalone` in the Tauri resource directory.
2. On first launch, install hive-api dependencies into a bundled venv.
3. Spawn `uvicorn hive_api.main:app --port <dynamic>` as a Tauri sidecar.
4. Health-check loop until `/health` returns `drones_booted: true`.
5. On app exit, terminate the sidecar process tree.

### Option 2 — PyInstaller Binary

Compile hive-api into a standalone executable using PyInstaller (or Nuitka). Distribute as a single binary alongside the Tauri app.

**Advantages:**
- No Python runtime dependency on the user's machine.
- Single binary, simpler process management.

**Disadvantages:**
- Larger bundle size (~100-200 MB depending on dependencies).
- Longer build times. PyInstaller builds are fragile across OS versions.
- Debugging production issues is harder (no source access in compiled binary).
- Each target platform needs a separate PyInstaller build in CI.

### Option 3 — Separate Installation

hive-api is installed and managed independently. The Tauri app connects to a user-provided URL (default `localhost:8000`).

**Advantages:**
- Maximum flexibility. Users can run hive-api on a remote server.
- Smallest installer size.
- Independent versioning and updates.

**Disadvantages:**
- Worst first-run UX. Users must install Python, clone hive-api, and start it manually.
- Version compatibility between Tauri app and hive-api must be managed.
- Support burden increases significantly.

## Decision

**Pending.** A decision is required before Phase 2 implementation begins.

The current recommendation is **Option 1 (Sidecar Process)** based on:
- Fastest path to a working prototype.
- Acceptable install size trade-off for a desktop application.
- Tauri's existing sidecar infrastructure reduces custom code.

## Evaluation Criteria

The final decision should weigh:

| Criterion          | Weight | Option 1 | Option 2 | Option 3 |
|--------------------|--------|-----------|----------|----------|
| Install size       | Medium | ~60 MB    | ~150 MB  | ~0 MB    |
| First-run UX       | High   | Good      | Good     | Poor     |
| Build complexity   | Medium | Low       | High     | None     |
| Debug-ability      | Medium | High      | Low      | High     |
| Cross-platform     | High   | Medium    | Low      | High     |
| Time to implement  | High   | Low       | Medium   | Low      |

## Consequences (Deferred)

To be documented once the decision is finalised.
