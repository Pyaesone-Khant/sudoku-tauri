//! Typed domain errors for `sudoku-core`.

use thiserror::Error;

/// Canonical error type returned by all fallible domain operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    /// Cell index outside the valid `0..81` range.
    #[error("cell index {0} is out of range (expected 0..81)")]
    InvalidCellIndex(usize),

    /// Digit value outside the valid `1..=9` range.
    #[error("digit {0} is out of range (expected 1..=9)")]
    InvalidDigit(u8),

    /// Calendar date components do not form a valid Gregorian date.
    #[error("invalid calendar date: {0}")]
    InvalidDate(String),

    /// Attempted to modify or erase a pre-filled Given cell.
    #[error("cell {0} is a Given and cannot be modified")]
    CellIsGiven(u8),

    /// Attempted a board move when the Game is not in `Active` status.
    #[error("game is not active")]
    GameNotActive,

    /// Attempted to resume a Game that is not in `Paused` status.
    #[error("game is not paused")]
    GameNotPaused,

    /// No active Game exists in the requested slot.
    #[error("no active game in the requested slot")]
    NoActiveGame,

    /// Attempted to undo when the undo history is empty.
    #[error("nothing to undo")]
    NothingToUndo,

    /// Attempted to redo when the redo history is empty.
    #[error("nothing to redo")]
    NothingToRedo,

    /// Attempted to apply a Hint when no Hint suggestion is currently active.
    #[error("no active hint to apply")]
    NoActiveHint,

    /// Attempted to open a Daily Challenge dated in the future.
    #[error("daily challenge for {0} is locked until that date")]
    FutureDailyLocked(String),

    /// Attempted to start a Daily Challenge that has already been completed.
    #[error("daily challenge for {0} is already completed")]
    DailyAlreadyCompleted(String),

    /// Failure reported by a persistence adapter implementing `GameRepository`.
    #[error("storage error: {0}")]
    Storage(String),
}
