//! Pure, Tauri-agnostic Sudoku domain engine.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod error;
pub mod game;
pub mod generator;
pub mod model;
pub mod ports;
pub mod scoring;
pub mod service;
pub mod solver;
pub mod stats;

pub use error::DomainError;
pub use game::{CellSnapshot, Game, GameSnapshot, MoveOutcome};
pub use generator::{generate_daily_puzzle, generate_standard_puzzle};
pub use model::{
    CellIndex, DailyDate, Difficulty, Digit, GameKind, GameStatus, NotesMask, Puzzle, Weekday,
};
pub use ports::{
    ActiveSlot, Clock, FixedClock, GameRepository, InMemoryRepository, PersistedState,
};
pub use scoring::{compute_score_breakdown, compute_time_bonus, ScoreBreakdown};
pub use service::{AppStateSnapshot, SudokuApp};
pub use solver::{
    compute_smart_hint, generate_complete_grid, grade_puzzle, solve_unique, HintSuggestion,
    SolutionCount, SolvingTechnique,
};
pub use stats::{
    CompletedGameRecord, DailyCalendarDay, DailyCalendarMonth, DailyDayState, DifficultyStats,
    StatsSummary,
};

/// Returns the crate version string for diagnostics.
pub const fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_version_is_non_empty() {
        assert!(!core_version().is_empty());
    }
}
