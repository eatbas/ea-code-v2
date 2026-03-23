# EA Code v2 Implementation Plan

This folder contains the phased execution plan for the EA Code v2 rewrite.

## Planning assumptions

- Primary target is the desktop app in `frontend/desktop`.
- Keep the current app shell and interaction style (sidebar, session list, central workspace card, prompt bar).
- Replace direct CLI execution with hive-api while preserving existing pause, cancel, resume, and history behaviour.
- Deliver incrementally so each phase ends with a demonstrable vertical slice.

## Phase map

1. [Phase1](./Phase1.md) - Foundation and contracts
2. [Phase2](./Phase2.md) - hive-api integration and runtime control
3. [Phase3](./Phase3.md) - Pipeline templates, prompts, and persistence
4. [Phase4](./Phase4.md) - Dynamic orchestrator and session groups
5. [Phase5](./Phase5.md) - Frontend builder and execution UX
6. [Phase6](./Phase6.md) - Migration, hardening, and release readiness

## Success definition

v2 is complete when users can create and run custom pipelines end-to-end, stages can resume shared provider sessions by session group, prompts are editable per stage, and history is stable under interruption and restart.


