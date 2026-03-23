# Project Guidelines

## Stack

- **Backend:** Rust (Tauri)
- **Frontend:** React, TypeScript, TailwindCSS

## DRY — Don't Repeat Yourself

- Never duplicate logic. Reuse existing functions, utilities, and patterns.
- Before writing new code, search the codebase for existing implementations.
- Extract shared logic into well-named, reusable modules.

## File Size Limit

- **Maximum 300 lines per file.** No exceptions.
- When a file exceeds 300 lines, split it into a well-named folder with clearly named sub-files.
- Folder and file names must be descriptive and reflect their purpose (e.g., `commands/auth.rs`, not `utils2.rs`).

## No Quick Wins — Future-Proof Only

- Never write throwaway or "just make it work" code.
- Every solution must be stable, maintainable, and future-proof.
- Prefer long-term correctness over short-term speed.
- Design for extensibility — new providers, models, or features should slot in cleanly.

## Best Practices

- Follow Rust and Tauri best practices at all times.
- Use strong typing, idiomatic Rust patterns (Result, Option, traits), and async correctly.
- Follow React and TypeScript best practices on the frontend.
- Use proper TypeScript types — avoid `any`.
- Keep components focused — single responsibility.
- Use TailwindCSS utility classes; avoid inline astyles and custom CSS where Tailwind suffices.
- Write clean, readable code with clear intent.
- Keep functions focused — single responsibility.

## Latest Libraries & Versions

- Always use the **latest stable versions** of all libraries.
- **Use context7** MCP tool to look up current documentation before writing code that depends on a library.
- Do not rely on memorized or outdated API signatures — verify with context7.

## Post-Edit Checks

### Rust / Tauri Backend
- After modifying any Rust file, run `cargo check` from the relevant crate directory.
- Fix **all** errors and warnings before considering the task done. No warnings left behind.

### TypeScript / React Frontend
- After modifying any `.ts` or `.tsx` file, run `tsc --noEmit` (or the project's type-check script) to surface type errors.
- Fix **all** type errors before considering the task done.

## Don't Assume — Ask

- If requirements are ambiguous, **ask** before implementing.
- If you're unsure about a design decision, **ask** before committing to it.
- Use **context7** to verify library APIs, patterns, and best practices rather than guessing.
- Never assume a function exists or an API works a certain way — look it up.
