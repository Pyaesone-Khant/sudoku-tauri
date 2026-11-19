//! Clean Architecture boundary traits for persistence and local system date.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::game::Game;
use crate::model::DailyDate;
use crate::stats::CompletedGameRecord;

/// Identifies which active Game slot is currently focused on the Play screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveSlot {
    #[default]
    Standard,
    Daily,
}

/// Complete offline persistence envelope stored by a `GameRepository`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedState {
    /// Schema version for forward-compatible migrations.
    pub version: u32,
    /// Monotonically increasing Game ID counter.
    pub next_game_id: u64,
    /// In-progress (or most recently completed) Standard Game slot.
    pub active_standard: Option<Game>,
    /// In-progress (or most recently completed) Daily Challenge slot.
    pub active_daily: Option<Game>,
    /// Currently focused slot on the Play screen.
    pub active_slot: ActiveSlot,
    /// Full history of completed Games (Standard + Daily).
    pub completed_games: Vec<CompletedGameRecord>,
    /// Abandoned Standard Game counts indexed by Difficulty (`Easy`, `Medium`, `Hard`, `Expert`).
    pub abandoned_by_difficulty: [u32; 4],
    /// Current Standard Game Win Streak.
    pub win_streak_current: u32,
    /// Best all-time Standard Game Win Streak.
    pub win_streak_best: u32,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            version: 1,
            next_game_id: 1,
            active_standard: None,
            active_daily: None,
            active_slot: ActiveSlot::Standard,
            completed_games: Vec::new(),
            abandoned_by_difficulty: [0; 4],
            win_streak_current: 0,
            win_streak_best: 0,
        }
    }
}

/// Persistence port for loading and atomically saving `PersistedState`.
pub trait GameRepository: Send + Sync {
    /// Loads saved state, or returns `PersistedState::default()` if none exists yet.
    fn load(&self) -> Result<PersistedState, DomainError>;

    /// Atomically persists the full state snapshot.
    fn save(&self, state: &PersistedState) -> Result<(), DomainError>;
}

/// Clock port providing the current local calendar date.
pub trait Clock: Send + Sync {
    /// Returns today's local Gregorian calendar date.
    fn today(&self) -> DailyDate;
}

/// Deterministic in-memory `GameRepository` for fast unit and integration tests.
#[derive(Debug, Default)]
pub struct InMemoryRepository {
    inner: Mutex<PersistedState>,
}

impl InMemoryRepository {
    /// Creates an empty in-memory repository.
    pub fn new() -> Self {
        Self::default()
    }
}

impl GameRepository for InMemoryRepository {
    fn load(&self) -> Result<PersistedState, DomainError> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| DomainError::Storage("in-memory lock poisoned".to_owned()))?;
        Ok(guard.clone())
    }

    fn save(&self, state: &PersistedState) -> Result<(), DomainError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| DomainError::Storage("in-memory lock poisoned".to_owned()))?;
        *guard = state.clone();
        Ok(())
    }
}

/// Fixed-date `Clock` for deterministic tests.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock {
    today: DailyDate,
}

impl FixedClock {
    /// Creates a clock always returning `today`.
    pub const fn new(today: DailyDate) -> Self {
        Self { today }
    }
}

impl Clock for FixedClock {
    fn today(&self) -> DailyDate {
        self.today
    }
}
