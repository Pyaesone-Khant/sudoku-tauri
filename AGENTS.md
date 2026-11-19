# AGENTS.md

Project instructions for all coding agents working in this repository.

## Documentation First

- **Domain language:** Always read [CONTEXT.md](./CONTEXT.md) before designing, naming types, or writing UI copy. Use the canonical terms (`Puzzle`, `Game`, `Daily Challenge`, `Difficulty`, `Given`, `Notes`, `Mistake`, `Hint`, `Solve Time`, `Earned Score`, `Time Bonus`, `Penalty`, `Final Score`, `Flawless Solve`, `Daily Streak`, `Win Streak`) and avoid the listed `_Avoid_` synonyms. Keep `CONTEXT.md` strictly as a domain glossary—never put implementation details or workflow rules in it.
- **Architectural decisions:** Check [docs/adr/](./docs/adr/) before proposing structural changes. Record new ADRs only when a decision is hard to reverse, surprising without context, and the result of a real trade-off.

## Architecture & Maintainability

- **Stack:** Tauri v2 (macOS-first desktop app) with a Rust backend and React + TypeScript + Tailwind CSS frontend ([ADR-0001](./docs/adr/0001-tauri-v2-stack.md)).
- **Clean Architecture boundary:** All domain rules, puzzle generation, deterministic Daily Challenge seeding, human-technique solver/Smart Hints, scoring, stats, and persistence contracts live in a pure, Tauri-agnostic Rust crate (`sudoku-core`). External concerns (disk storage, system clock/date) cross boundaries via Rust traits ([ADR-0002](./docs/adr/0002-rust-owned-domain-engine.md)).
- **Thin UI layer:** The React/TypeScript frontend strictly renders authoritative snapshots from Tauri IPC commands and handles input, focus, animations, and sound cues. Never duplicate Sudoku validation, scoring, or Undo invariants in TypeScript.
- **Bundle size discipline:** Keep the macOS `.app` bundle as small as possible (~6–10 MB). Avoid heavy UI component libraries, unneeded Rust crates, or bundled audio/media assets (prefer synthesized Web Audio and system typography).

## Coding Standards & Skills

Apply these installed skills and standards whenever writing or reviewing code:

- **Rust:** Follow `rust-best-practices`, `rust-patterns`, and `rust-testing`. Enforce `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `thiserror` domain error enums, and type-safe states (no `.unwrap()` in production paths).
- **Tauri v2:** Follow `tauri-development` for IPC command boundaries, capabilities/permissions, and native macOS window integration.
- **React & TypeScript:** Follow `vercel-composition-patterns` and `vercel-react-best-practices`. Maintain strict TypeScript (`no any`), 2-space indentation, and JSDoc on public functions/interfaces.

## Development Workflow (TDD + Atomic Git Commits)

1. **Test-Driven Development (TDD):**
   - Reproduce bugs with a failing test before fixing them.
   - Write unit/property tests alongside every new feature (especially `sudoku-core` invariants: unique solutions, technique-based Difficulty grading, deterministic Daily Challenge seeding, permanent Mistakes across Undo/Erase, single-award cell scoring).
2. **Verify before committing:**
   - Rust changes: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` must all pass.
   - Frontend changes: TypeScript typecheck, lint, and tests must all pass.
3. **Commit every completed, tested increment:**
   - **Whenever any implementation is successfully implemented and tested—even a small feature, fix, or refactor—commit it to git immediately.**
   - Keep commits small, atomic, and scoped to one logical change with clear Conventional Commit messages (e.g., `feat(core): ...`, `fix(ui): ...`, `test(core): ...`, `docs: ...`).
   - Never commit broken or untested code, and never push to a remote unless explicitly asked by the user.
