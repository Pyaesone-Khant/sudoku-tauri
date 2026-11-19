//! Pure, Tauri-agnostic Sudoku domain engine.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod error;
pub mod model;
pub mod scoring;

pub use error::DomainError;
pub use model::{
    CellIndex, DailyDate, Difficulty, Digit, GameKind, GameStatus, NotesMask, Puzzle, Weekday,
};
pub use scoring::{compute_score_breakdown, compute_time_bonus, ScoreBreakdown};

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
