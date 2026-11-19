# Keep All Domain Logic in a Pure Rust Crate (`sudoku-core`)

All Sudoku domain rules — puzzle generation, deterministic Daily Challenge seeding, human-technique grading, Smart Hints, Mistake detection, Notes auto-clear, Undo/Redo, Scoring, Stats aggregation, and persistence contracts — live in a Tauri-agnostic Rust library crate (`sudoku-core`), while the React/TypeScript frontend acts strictly as a rendering and input layer over thin Tauri IPC commands. This guarantees a single source of truth for invariants (such as Undo never erasing Mistakes or Penalties), enables fast offline `cargo test` TDD without a WebView, and keeps the frontend bundle minimal.

## Considered Options

- **Split logic (Rust for generation/I/O, TypeScript for live board state and scoring)**: Avoids IPC round-trips on cell edits, but duplicates Sudoku rules and Smart Hint awareness across two languages and risks state drift.

## Consequences

- Every user move crosses local Tauri IPC (<0.5 ms on macOS) and returns an authoritative snapshot for React to render.
- `sudoku-core` depends on traits for persistence and clock/date lookup so it remains cleanly testable and reusable across future UIs (CLI, mobile, or alternative desktop shells).
