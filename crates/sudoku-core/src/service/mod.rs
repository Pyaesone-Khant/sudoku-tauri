//! Application orchestrator enforcing dual active slots, Daily locking, and auto-save.

use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::game::{Game, GameSnapshot, MoveOutcome};
use crate::generator::{generate_daily_puzzle, generate_standard_puzzle};
use crate::model::{CellIndex, DailyDate, Difficulty, Digit, GameKind, GameStatus};
use crate::ports::{ActiveSlot, Clock, GameRepository, PersistedState};
use crate::solver::HintSuggestion;
use crate::stats::{
    build_daily_calendar_month, build_stats_summary, CompletedGameRecord, DailyCalendarMonth,
    StatsSummary,
};

/// Top-level snapshot returned to the frontend so both active slots and stats stay synchronized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppStateSnapshot {
    /// Currently focused slot (`Standard` or `Daily`).
    pub active_slot: ActiveSlot,
    /// Snapshot of the currently focused Game, if present.
    pub current_game: Option<GameSnapshot>,
    /// True if an in-progress (`Active` or `Paused`) Standard Game exists.
    pub has_in_progress_standard: bool,
    /// Date of the currently in-progress Daily Challenge, if any.
    pub in_progress_daily_date: Option<DailyDate>,
    /// Today's local date from the Clock.
    pub today: DailyDate,
    /// Most recent move outcome, if triggered by a digit/hint action.
    pub last_move_outcome: Option<MoveOutcome>,
}

/// Clean Architecture application service owning all game orchestration.
#[derive(Debug)]
pub struct SudokuApp<R: GameRepository, C: Clock> {
    repo: R,
    clock: C,
    state: PersistedState,
}

impl<R: GameRepository, C: Clock> SudokuApp<R, C> {
    /// Loads persisted offline state and constructs the application service.
    pub fn load(repo: R, clock: C) -> Result<Self, DomainError> {
        let mut state = repo.load()?;
        // Any active Game left running across a crash/quit resumes cleanly in `Paused` state
        // so Solve Time doesn't silently tick before the player is ready.
        if let Some(game) = state.active_standard.as_mut() {
            if game.status() == GameStatus::Active {
                let _ = game.pause();
            }
        }
        if let Some(game) = state.active_daily.as_mut() {
            if game.status() == GameStatus::Active {
                let _ = game.pause();
            }
        }
        Ok(Self { repo, clock, state })
    }

    /// Returns the authoritative top-level app state snapshot.
    pub fn app_snapshot(&self, last_move_outcome: Option<MoveOutcome>) -> AppStateSnapshot {
        let current_game = self.focused_game().map(Game::snapshot);
        let has_in_progress_standard = self
            .state
            .active_standard
            .as_ref()
            .is_some_and(|g| matches!(g.status(), GameStatus::Active | GameStatus::Paused));
        let in_progress_daily_date = self.in_progress_daily_date();

        AppStateSnapshot {
            active_slot: self.state.active_slot,
            current_game,
            has_in_progress_standard,
            in_progress_daily_date,
            today: self.clock.today(),
            last_move_outcome,
        }
    }

    /// Starts a new Standard Game at `difficulty`.
    ///
    /// If another Standard Game is currently in progress, it is marked `Abandoned`,
    /// increments its Difficulty's abandoned count, and resets current `Win Streak`.
    /// Never modifies an in-progress Daily Challenge.
    pub fn start_standard_game(
        &mut self,
        difficulty: Difficulty,
    ) -> Result<AppStateSnapshot, DomainError> {
        if let Some(existing) = self.state.active_standard.as_mut() {
            if matches!(existing.status(), GameStatus::Active | GameStatus::Paused) {
                existing.abandon();
                let d_idx = difficulty_index(existing.difficulty());
                self.state.abandoned_by_difficulty[d_idx] =
                    self.state.abandoned_by_difficulty[d_idx].saturating_add(1);
                self.state.win_streak_current = 0;
            }
        }

        // Pause active Daily if user switches away from it.
        if let Some(daily) = self.state.active_daily.as_mut() {
            if daily.status() == GameStatus::Active {
                let _ = daily.pause();
            }
        }

        let mut rng = OsRng;
        let puzzle = generate_standard_puzzle(difficulty, &mut rng);
        let id = self.allocate_id();
        self.state.active_standard = Some(Game::new(id, GameKind::Standard, puzzle));
        self.state.active_slot = ActiveSlot::Standard;
        self.persist()?;
        Ok(self.app_snapshot(None))
    }

    /// Opens or resumes the offline Daily Challenge for `date`.
    ///
    /// - Rejects future dates (`FutureDailyLocked`).
    /// - Rejects already-completed dates (`DailyAlreadyCompleted`).
    /// - Preserves any in-progress Standard Game untouched (pausing it while playing Daily).
    pub fn open_daily_challenge(
        &mut self,
        date: DailyDate,
    ) -> Result<AppStateSnapshot, DomainError> {
        let today = self.clock.today();
        if date > today {
            return Err(DomainError::FutureDailyLocked(date.to_iso_string()));
        }
        if self.is_daily_completed(date) {
            return Err(DomainError::DailyAlreadyCompleted(date.to_iso_string()));
        }

        // Pause Standard Game if active so its Solve Time stays honest.
        if let Some(std_game) = self.state.active_standard.as_mut() {
            if std_game.status() == GameStatus::Active {
                let _ = std_game.pause();
            }
        }

        let already_open_same_date = self.state.active_daily.as_ref().is_some_and(|g| {
            g.kind() == GameKind::Daily(date)
                && matches!(g.status(), GameStatus::Active | GameStatus::Paused)
        });

        if already_open_same_date {
            if let Some(daily) = self.state.active_daily.as_mut() {
                if daily.status() == GameStatus::Paused {
                    daily.resume()?;
                }
            }
        } else {
            let puzzle = generate_daily_puzzle(date);
            let id = self.allocate_id();
            self.state.active_daily = Some(Game::new(id, GameKind::Daily(date), puzzle));
        }

        self.state.active_slot = ActiveSlot::Daily;
        self.persist()?;
        Ok(self.app_snapshot(None))
    }

    /// Switches focus between the `Standard` and `Daily` active slots, pausing the previous one.
    pub fn switch_active_slot(
        &mut self,
        slot: ActiveSlot,
    ) -> Result<AppStateSnapshot, DomainError> {
        if self.state.active_slot != slot {
            if let Some(prev) = self.focused_game_mut() {
                if prev.status() == GameStatus::Active {
                    let _ = prev.pause();
                }
            }
            self.state.active_slot = slot;
            self.persist()?;
        }
        Ok(self.app_snapshot(None))
    }

    /// Places `digit` in `cell` on the currently focused Game and persists immediately.
    pub fn place_digit(
        &mut self,
        cell: CellIndex,
        digit: Digit,
    ) -> Result<AppStateSnapshot, DomainError> {
        let outcome = self.focused_game_mut_or_err()?.place_digit(cell, digit)?;
        if outcome == MoveOutcome::Completed {
            self.record_focused_completion();
        }
        self.persist()?;
        Ok(self.app_snapshot(Some(outcome)))
    }

    /// Toggles `digit` in the Notes of `cell` on the currently focused Game.
    pub fn toggle_note(
        &mut self,
        cell: CellIndex,
        digit: Digit,
    ) -> Result<AppStateSnapshot, DomainError> {
        let _ = self.focused_game_mut_or_err()?.toggle_note(cell, digit)?;
        self.persist()?;
        Ok(self.app_snapshot(None))
    }

    /// Erases digit and Notes from `cell` on the currently focused Game.
    pub fn erase_cell(&mut self, cell: CellIndex) -> Result<AppStateSnapshot, DomainError> {
        let _ = self.focused_game_mut_or_err()?.erase_cell(cell)?;
        self.persist()?;
        Ok(self.app_snapshot(None))
    }

    /// Undoes the last board/Notes change on the currently focused Game.
    pub fn undo(&mut self) -> Result<AppStateSnapshot, DomainError> {
        self.focused_game_mut_or_err()?.undo()?;
        self.persist()?;
        Ok(self.app_snapshot(None))
    }

    /// Redoes the last undone board/Notes change on the currently focused Game.
    pub fn redo(&mut self) -> Result<AppStateSnapshot, DomainError> {
        self.focused_game_mut_or_err()?.redo()?;
        if self
            .focused_game()
            .is_some_and(|g| g.status() == GameStatus::Completed)
        {
            self.record_focused_completion();
        }
        self.persist()?;
        Ok(self.app_snapshot(None))
    }

    /// Requests a Smart Hint on the currently focused Game.
    pub fn request_hint(&mut self) -> Result<(AppStateSnapshot, HintSuggestion), DomainError> {
        let hint = self.focused_game_mut_or_err()?.request_hint()?;
        self.persist()?;
        Ok((self.app_snapshot(None), hint))
    }

    /// Applies the active Smart Hint on the currently focused Game.
    pub fn apply_hint(&mut self) -> Result<AppStateSnapshot, DomainError> {
        let outcome = self.focused_game_mut_or_err()?.apply_hint()?;
        if outcome == MoveOutcome::Completed {
            self.record_focused_completion();
        }
        self.persist()?;
        Ok(self.app_snapshot(Some(outcome)))
    }

    /// Dismisses the active Smart Hint banner on the currently focused Game.
    pub fn dismiss_hint(&mut self) -> Result<AppStateSnapshot, DomainError> {
        self.focused_game_mut_or_err()?.dismiss_hint();
        self.persist()?;
        Ok(self.app_snapshot(None))
    }

    /// Pauses the currently focused Game (if active) and persists state.
    pub fn pause_game(&mut self) -> Result<AppStateSnapshot, DomainError> {
        if let Some(game) = self.focused_game_mut() {
            if game.status() == GameStatus::Active {
                game.pause()?;
                self.persist()?;
            }
        }
        Ok(self.app_snapshot(None))
    }

    /// Resumes the currently focused Game (if paused) and persists state.
    pub fn resume_game(&mut self) -> Result<AppStateSnapshot, DomainError> {
        self.focused_game_mut_or_err()?.resume()?;
        self.persist()?;
        Ok(self.app_snapshot(None))
    }

    /// Adds `delta_secs` to the active Game's Solve Time and persists.
    pub fn sync_solve_time(&mut self, delta_secs: u64) -> Result<AppStateSnapshot, DomainError> {
        if delta_secs > 0 {
            if let Some(game) = self.focused_game_mut() {
                if game.status() == GameStatus::Active {
                    game.add_solve_time(delta_secs);
                    self.persist()?;
                }
            }
        }
        Ok(self.app_snapshot(None))
    }

    /// Returns the monthly Daily Challenge Calendar projection for `(year, month)`.
    pub fn daily_calendar(&self, year: i32, month: u8) -> Result<DailyCalendarMonth, DomainError> {
        if !(1..=12).contains(&month) {
            return Err(DomainError::InvalidDate(format!("{year:04}-{month:02}")));
        }
        Ok(build_daily_calendar_month(
            year,
            month,
            self.clock.today(),
            self.in_progress_daily_date(),
            &self.state.completed_games,
        ))
    }

    /// Returns the full Statistics summary across all Difficulties and streaks.
    pub fn stats_summary(&self) -> StatsSummary {
        build_stats_summary(
            &self.state.completed_games,
            self.state.abandoned_by_difficulty,
            self.state.win_streak_current,
            self.state.win_streak_best,
            self.clock.today(),
        )
    }

    /// Exposes the currently focused Game for tests/diagnostics.
    pub fn focused_game(&self) -> Option<&Game> {
        match self.state.active_slot {
            ActiveSlot::Standard => self.state.active_standard.as_ref(),
            ActiveSlot::Daily => self.state.active_daily.as_ref(),
        }
    }

    fn focused_game_mut(&mut self) -> Option<&mut Game> {
        match self.state.active_slot {
            ActiveSlot::Standard => self.state.active_standard.as_mut(),
            ActiveSlot::Daily => self.state.active_daily.as_mut(),
        }
    }

    fn focused_game_mut_or_err(&mut self) -> Result<&mut Game, DomainError> {
        self.focused_game_mut().ok_or(DomainError::NoActiveGame)
    }

    fn allocate_id(&mut self) -> u64 {
        let id = self.state.next_game_id;
        self.state.next_game_id = self.state.next_game_id.saturating_add(1);
        id
    }

    fn is_daily_completed(&self, date: DailyDate) -> bool {
        self.state
            .completed_games
            .iter()
            .any(|r| r.kind == GameKind::Daily(date))
    }

    fn in_progress_daily_date(&self) -> Option<DailyDate> {
        self.state.active_daily.as_ref().and_then(|g| {
            if matches!(g.status(), GameStatus::Active | GameStatus::Paused) {
                match g.kind() {
                    GameKind::Daily(d) => Some(d),
                    GameKind::Standard => None,
                }
            } else {
                None
            }
        })
    }

    fn record_focused_completion(&mut self) {
        let Some(game) = self.focused_game() else {
            return;
        };
        if self.state.completed_games.iter().any(|r| r.id == game.id()) {
            return;
        }
        let record = CompletedGameRecord {
            id: game.id(),
            kind: game.kind(),
            difficulty: game.difficulty(),
            solve_time_secs: game.solve_time_secs(),
            mistakes: game.mistakes(),
            hints_used: game.hints_used(),
            score: game.score_breakdown(),
            flawless: game.is_flawless(),
            completed_on: self.clock.today(),
        };
        if matches!(record.kind, GameKind::Standard) {
            self.state.win_streak_current = self.state.win_streak_current.saturating_add(1);
            if self.state.win_streak_current > self.state.win_streak_best {
                self.state.win_streak_best = self.state.win_streak_current;
            }
        }
        self.state.completed_games.push(record);
    }

    fn persist(&self) -> Result<(), DomainError> {
        self.repo.save(&self.state)
    }
}

const fn difficulty_index(difficulty: Difficulty) -> usize {
    match difficulty {
        Difficulty::Easy => 0,
        Difficulty::Medium => 1,
        Difficulty::Hard => 2,
        Difficulty::Expert => 3,
    }
}
