# Offline macOS Sudoku App — Walkthrough

Built and verified the complete v1 offline macOS Sudoku desktop app in **Tauri v2**, structured around a pure Rust domain engine (`sudoku-core`), a thin macOS Tauri host (`src-tauri`), and a minimal React 19 + TypeScript + Tailwind CSS v4 frontend.

> **Bundle Size Result:** The compiled release macOS bundle (`target/release/bundle/macos/Sudoku.app`) weighs **`3.39 MiB`** — well under our 6–10 MB target, thanks to macOS's built-in WebView, zero bundled fonts/audio, LTO + symbol stripping, and an ~83 KB gzipped React UI.

---

## What Was Built

### 1. Pure Rust Domain Engine (`crates/sudoku-core`)
Completely Tauri-agnostic, zero `.unwrap()` in production paths (`#![cfg_attr(not(test), deny(clippy::unwrap_used))]`), and strictly aligned with [CONTEXT.md](../CONTEXT.md):
- **Domain Primitives & Calendar Date Math** (`model/mod.rs`, `model/date.rs`, `error.rs`): Validated `CellIndex`, `Digit`, `NotesMask`, `Difficulty`, `GameKind`, `GameStatus`, `Puzzle`, `DomainError`, and pure Gregorian `DailyDate` arithmetic with deterministic weekly Difficulty scheduling (Mon `Easy`, Tue–Wed `Medium`, Thu–Fri `Hard`, Sat–Sun `Expert`).
- **Scoring Engine** (`scoring/mod.rs`): Exact **Earned Score**, completion-only **Time Bonus**, **Mistake** / **Hint** **Penalty**, and **Final Score** floored at zero.
- **Exact Solver, Human-Technique Grader & Smart Hint Engine** (`solver/exact.rs`, `solver/human.rs`, `generator/mod.rs`):
  - Bitmask MRV uniqueness verifier guaranteeing every Puzzle has **exactly 1 valid solution**.
  - Ordered human technique ladder (`NakedSingle`, `HiddenSingle` → `LockedCandidates`, `NakedPair`, `HiddenPair` → `NakedTriple`, `HiddenTriple`, `XWing` → `XYWing`, `Swordfish`, `ForcingChain`).
  - **Smart Hint** inspects the board for any existing **Mistake** first before deducing the next logical placement with explanation and supporting cell highlights.
  - Deterministic offline **Daily Challenge** generator seeded via version-stable FNV-1a + `SplitMix64` into `ChaCha8Rng`.
- **Game State Machine, Stats & Clean Architecture Ports** (`game/mod.rs`, `stats/mod.rs`, `ports/mod.rs`, `service/mod.rs`):
  - Enforces permanent **Mistakes** across Erase/Overwrite/Undo, single-award **Earned Score** cells (never on Hint fills), **Notes** peer auto-clear with Undo restoration, and max **1 active Standard Game + 1 active Daily Challenge** simultaneously.
  - Tracks per-Difficulty Stats, **Flawless Solves**, **Win Streak**, **Daily Streak**, Recent History, and monthly **Daily Challenge** calendar states (past catch-up playable, future locked, completed locked).

### 2. Thin Tauri v2 macOS Shell (`src-tauri`)
- `src-tauri/src/adapters/mod.rs`: Crash-safe `AtomicJsonRepository` (write → `sync_all` → atomic rename in macOS Application Support) + `SystemClock`.
- `src-tauri/src/commands/mod.rs` & `src-tauri/src/lib.rs`: 17 thin `#[tauri::command]` handlers + macOS `WindowEvent::Focused(false)` auto-pause listener emitting `game-auto-paused`.
- `src-tauri/tauri.conf.json`: macOS `Overlay` titlebar, `780×660` default (`680×580` min), minimal capabilities, and custom macOS icon set.

### 3. Refined macOS-Native Frontend (`src/`)
- **Design System & Zero-Asset Audio** (`src/index.css`, `src/context/PreferencesProvider.tsx`, `src/audio/soundEngine.ts`): Semantic Light/Dark/System tokens, SF Pro `tabular-nums`, synthesized Web Audio cues with mute toggle.
- **Play View** (`src/components/board/*`, `src/hooks/useKeyboardControls.ts`): Full keyboard (`Arrows`/`WASD`, `1–9`, `N` or `Shift+1–9`, `⌫`, `⌘Z`/`⌘⇧Z`, `H`, `P`) + mouse play, frosted Pause veil hiding digits, dual-slot switcher, localized micro-confetti + diagonal cell pulse when a digit completes all 9 placements, full-canvas confetti burst on Puzzle completion (honoring `prefers-reduced-motion`), and full completion score breakdown.
- **Daily Challenge Calendar & Stats** (`src/components/daily/DailyCalendarView.tsx`, `src/components/stats/StatsView.tsx`): Monthly calendar with catch-up play and **Daily Streak**; per-Difficulty tabs with **Win Streak**, **Daily Streak**, personal bests, and **Recent History**.

---

## Verification Results

| Check | Command | Result |
| :--- | :--- | :--- |
| Rust formatting | `cargo fmt --all -- --check` | Passed |
| Rust lints (pedantic, zero warnings) | `cargo clippy --workspace --all-targets --offline -- -D warnings` | Passed |
| Rust unit + property + integration tests | `cargo test --workspace --offline` | **49 / 49 passed** |
| TypeScript strict typecheck | `pnpm typecheck` | Passed |
| Frontend component tests | `pnpm test` | **7 / 7 passed** |
| Production frontend build | `pnpm build` | Passed (`84.85 kB` gzip JS, `5.77 kB` gzip CSS) |
| Release macOS `.app` bundle | `pnpm tauri build --bundles app -- --offline` | Passed — **`3.39 MiB`** (`Sudoku.app`) |

---

## Launch Locally on macOS

- **Open the built release app directly:**
  ```bash
  open target/release/bundle/macos/Sudoku.app
  ```
- **Or run in live hot-reload dev mode:**
  ```bash
  pnpm tauri dev
  ```
