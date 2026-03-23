# Project Engineering Guidelines

## Stack
- Backend: Rust (Tauri)
- Frontend: React, TypeScript, TailwindCSS

## Core Principles
- Prefer clarity over cleverness.
- Prefer correctness over speed of delivery.
- Prefer extensibility over shortcuts.
- Keep responsibilities narrow and explicit.
- Build for long-term maintenance, not one-off delivery.

## Architecture
- Keep UI, application logic, domain logic, and infrastructure separate.
- Tauri command handlers must stay thin and delegate business logic to services.
- Frontend components must not contain backend orchestration or heavy business rules.
- Shared logic must live in reusable, well-named modules.
- New providers, models, and integrations must plug into existing abstractions rather than branching special cases through the codebase.

## DRY
- Never duplicate business logic.
- Before adding new code, search for an existing abstraction or pattern.
- Extract repeated logic into named utilities, services, or domain modules.
- Do not over-abstract prematurely; extract only when the shared concept is real.

## File Size
- Target: under 300 lines per file.
- Soft limit: 300 lines.
- Hard limit: 400 lines only if the file remains highly cohesive.
- Split files by responsibility, not only by length.
- Avoid meaningless names like `utils2`, `helpersNew`, or `temp`.
- When a file exceeds 400 lines, split it into a well-named folder with clearly named sub-files. 
- Folder and file names must be descriptive and reflect their purpose (e.g., commands/auth.rs, not utils2.rs).

## Rust / Tauri Standards
- Follow idiomatic Rust patterns: `Result`, `Option`, enums, traits, and ownership-aware design.
- Use strong types instead of loosely typed primitives where meaning matters.
- Do not use `unwrap` or `expect` in production paths unless failure is intentionally unrecoverable and documented.
- All command inputs must be validated on the Rust side.
- Keep Tauri permissions and capabilities minimal.
- Expose the smallest possible native surface area to the frontend.

## React / TypeScript Standards
- Use function components only.
- Keep components focused and composable.
- Components and hooks must be pure.
- Avoid `any`; prefer `unknown` plus narrowing.
- Validate external data at boundaries.
- Prefer derived state over duplicated state.
- Keep side effects isolated and explicit.
- TailwindCSS should be preferred over custom CSS unless design reuse clearly justifies abstraction.

## TypeScript Compiler Rules
- `strict: true`
- `noUncheckedIndexedAccess: true`
- `exactOptionalPropertyTypes: true`
- `noImplicitOverride: true`
- `noUnusedLocals: true`
- `noUnusedParameters: true`

## Naming
- Names must reflect business meaning, not implementation accidents.
- Use domain language consistently across frontend and backend.
- A file’s purpose should be obvious from its name.

## Error Handling
- Errors must be typed, contextual, and actionable.
- Do not swallow errors.
- User-facing errors must be understandable.
- Internal logs must preserve debugging context.

## Testing
- Every bug fix should include a regression test where practical.
- Business logic must be unit tested.
- Critical flows must have integration coverage.
- Frontend tests should verify behavior, not implementation details.

## Documentation
- Public types and functions should be documented when non-obvious.
- Every Tauri command must document its purpose, inputs, outputs, and failure modes.
- Complex design decisions should be recorded in lightweight ADRs.

## Dependency Policy
- Use current stable, ecosystem-compatible library versions.
- Verify APIs against current official documentation before implementation.
- Do not rely on memory for library APIs.
- Upgrade dependencies intentionally, not opportunistically.

## Post-Edit Checks

### Rust / Tauri
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo check --all-targets --all-features`
- Run relevant tests for touched code

### TypeScript / React
- `tsc --noEmit`
- Run lint
- Run relevant tests for touched code

## Decision-Making
- Ask questions when ambiguity affects architecture, UX, security, or external behavior.
- Otherwise choose the safest extensible default and state the assumption.
- Never invent APIs, library behavior, or undocumented project conventions.