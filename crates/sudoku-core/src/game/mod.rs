//! Game aggregate and authoritative UI snapshot projection.

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::model::{CellIndex, Difficulty, Digit, GameKind, GameStatus, NotesMask, Puzzle};
use crate::scoring::{compute_score_breakdown, ScoreBreakdown};
use crate::solver::{compute_smart_hint, HintSuggestion};

/// Outcome of a board digit placement, used by the UI for feedback and sound cues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveOutcome {
    /// Entry matched the cell's existing state; nothing changed.
    NoChange,
    /// Digit matched the unique solution.
    Correct,
    /// Digit disagreed with the unique solution; permanent Mistake recorded.
    Mistake,
    /// Final cell solved; Game transitioned to `Completed`.
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PeerNoteChange {
    cell: CellIndex,
    before: NotesMask,
    after: NotesMask,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BoardDelta {
    cell: CellIndex,
    prev_value: Option<Digit>,
    next_value: Option<Digit>,
    prev_notes: NotesMask,
    next_notes: NotesMask,
    peer_note_changes: Vec<PeerNoteChange>,
}

/// Authoritative snapshot of a single board cell for the thin frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellSnapshot {
    /// Row-major cell index (`0..81`).
    pub index: u8,
    /// Current digit placed in the cell (`1..=9`), or `None` if empty.
    pub value: Option<u8>,
    /// True if pre-filled as a Puzzle Given.
    pub is_given: bool,
    /// True if the currently displayed digit disagrees with the unique solution.
    pub is_mistake: bool,
    /// True if this cell was revealed/locked by a Smart Hint.
    pub is_hinted: bool,
    /// Candidate digits (`1..=9`) currently marked in Notes.
    pub notes: Vec<u8>,
}

/// Authoritative snapshot of a Game sent to the frontend over Tauri IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSnapshot {
    /// Unique Game identifier.
    pub id: u64,
    /// Standard Game or Daily Challenge (`YYYY-MM-DD`).
    pub kind: GameKind,
    /// Assigned Difficulty.
    pub difficulty: Difficulty,
    /// Current lifecycle status.
    pub status: GameStatus,
    /// All 81 board cells in row-major order.
    pub cells: Vec<CellSnapshot>,
    /// Total permanent Mistakes recorded in this Game.
    pub mistakes: u32,
    /// Total Smart Hints requested in this Game.
    pub hints_used: u32,
    /// Active elapsed Solve Time in seconds.
    pub solve_time_secs: u64,
    /// Live or completed Score breakdown.
    pub score: ScoreBreakdown,
    /// True when zero Mistakes and zero Hints have been recorded.
    pub flawless: bool,
    /// Active Smart Hint suggestion, if one is currently shown.
    pub active_hint: Option<HintSuggestion>,
    /// Whether Undo is available.
    pub can_undo: bool,
    /// Whether Redo is available.
    pub can_redo: bool,
    /// Count of correctly placed occurrences for each digit `1..=9` (index 0 = digit 1).
    pub correct_digit_counts: [u8; 9],
}

/// A player's single play session on a Puzzle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    id: u64,
    kind: GameKind,
    puzzle: Puzzle,
    values: Vec<Option<Digit>>,
    notes: Vec<NotesMask>,
    awarded_cells: Vec<bool>,
    hinted_cells: Vec<bool>,
    mistakes: u32,
    hints_used: u32,
    solve_time_secs: u64,
    status: GameStatus,
    active_hint: Option<HintSuggestion>,
    undo_stack: Vec<BoardDelta>,
    redo_stack: Vec<BoardDelta>,
}

impl Game {
    /// Creates a new `Active` Game from `puzzle`.
    pub fn new(id: u64, kind: GameKind, puzzle: Puzzle) -> Self {
        let values = puzzle.givens.clone();
        Self {
            id,
            kind,
            puzzle,
            values,
            notes: vec![NotesMask::EMPTY; CellIndex::BOARD_SIZE],
            awarded_cells: vec![false; CellIndex::BOARD_SIZE],
            hinted_cells: vec![false; CellIndex::BOARD_SIZE],
            mistakes: 0,
            hints_used: 0,
            solve_time_secs: 0,
            status: GameStatus::Active,
            active_hint: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Returns the Game's unique ID.
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns whether this Game is a Standard Game or a Daily Challenge.
    pub const fn kind(&self) -> GameKind {
        self.kind
    }

    /// Returns the Puzzle's assigned Difficulty.
    pub const fn difficulty(&self) -> Difficulty {
        self.puzzle.difficulty
    }

    /// Returns the current lifecycle status.
    pub const fn status(&self) -> GameStatus {
        self.status
    }

    /// Returns the underlying Puzzle.
    pub const fn puzzle(&self) -> &Puzzle {
        &self.puzzle
    }

    /// Returns total permanent Mistakes recorded.
    pub const fn mistakes(&self) -> u32 {
        self.mistakes
    }

    /// Returns total Hints recorded.
    pub const fn hints_used(&self) -> u32 {
        self.hints_used
    }

    /// Returns active Solve Time in seconds.
    pub const fn solve_time_secs(&self) -> u64 {
        self.solve_time_secs
    }

    /// Returns true if the Game has zero Mistakes and zero Hints used.
    pub const fn is_flawless(&self) -> bool {
        self.mistakes == 0 && self.hints_used == 0
    }

    /// Computes the current or final `ScoreBreakdown`.
    pub fn score_breakdown(&self) -> ScoreBreakdown {
        #[allow(clippy::cast_possible_truncation)]
        let awarded = self.awarded_cells.iter().filter(|&&a| a).count() as u32;
        compute_score_breakdown(
            self.puzzle.difficulty,
            awarded,
            self.mistakes,
            self.hints_used,
            self.solve_time_secs,
            self.status == GameStatus::Completed,
        )
    }

    /// Places `digit` in an editable `cell`.
    ///
    /// - Validates against the Puzzle's unique solution immediately.
    /// - Correct placements auto-clear peer Notes and award Earned Score at most once
    ///   (never on Hint-locked cells).
    /// - Incorrect placements increment permanent `mistakes`.
    pub fn place_digit(
        &mut self,
        cell: CellIndex,
        digit: Digit,
    ) -> Result<MoveOutcome, DomainError> {
        self.ensure_active()?;
        if self.puzzle.is_given(cell) {
            return Err(DomainError::CellIsGiven(cell.as_u8()));
        }

        let idx = cell.as_usize();
        if self.values[idx] == Some(digit) {
            return Ok(MoveOutcome::NoChange);
        }

        let prev_value = self.values[idx];
        let prev_notes = self.notes[idx];
        self.values[idx] = Some(digit);
        self.notes[idx].clear();
        self.active_hint = None;

        let is_correct = self.puzzle.solution_digit(cell) == Some(digit);
        let mut peer_note_changes = Vec::new();

        if is_correct {
            for peer in cell.peers() {
                let p_idx = peer.as_usize();
                if self.notes[p_idx].contains(digit) {
                    let before = self.notes[p_idx];
                    self.notes[p_idx].remove(digit);
                    peer_note_changes.push(PeerNoteChange {
                        cell: peer,
                        before,
                        after: self.notes[p_idx],
                    });
                }
            }
            if !self.awarded_cells[idx] && !self.hinted_cells[idx] {
                self.awarded_cells[idx] = true;
            }
        } else {
            self.mistakes = self.mistakes.saturating_add(1);
        }

        self.undo_stack.push(BoardDelta {
            cell,
            prev_value,
            next_value: Some(digit),
            prev_notes,
            next_notes: NotesMask::EMPTY,
            peer_note_changes,
        });
        self.redo_stack.clear();

        if is_correct && self.is_board_complete() {
            self.status = GameStatus::Completed;
            return Ok(MoveOutcome::Completed);
        }

        Ok(if is_correct {
            MoveOutcome::Correct
        } else {
            MoveOutcome::Mistake
        })
    }

    /// Toggles `digit` in the Notes of an empty editable `cell`.
    pub fn toggle_note(&mut self, cell: CellIndex, digit: Digit) -> Result<bool, DomainError> {
        self.ensure_active()?;
        if self.puzzle.is_given(cell) {
            return Err(DomainError::CellIsGiven(cell.as_u8()));
        }

        let idx = cell.as_usize();
        if self.values[idx].is_some() {
            return Ok(false);
        }

        let prev_notes = self.notes[idx];
        self.notes[idx].toggle(digit);
        let next_notes = self.notes[idx];

        self.undo_stack.push(BoardDelta {
            cell,
            prev_value: None,
            next_value: None,
            prev_notes,
            next_notes,
            peer_note_changes: Vec::new(),
        });
        self.redo_stack.clear();
        Ok(true)
    }

    /// Erases any entered digit and Notes from an editable `cell`.
    ///
    /// Never decrements Mistakes or clears single-award Earned Score tracking.
    pub fn erase_cell(&mut self, cell: CellIndex) -> Result<bool, DomainError> {
        self.ensure_active()?;
        if self.puzzle.is_given(cell) {
            return Err(DomainError::CellIsGiven(cell.as_u8()));
        }

        let idx = cell.as_usize();
        if self.values[idx].is_none() && self.notes[idx].is_empty() {
            return Ok(false);
        }

        let prev_value = self.values[idx];
        let prev_notes = self.notes[idx];
        self.values[idx] = None;
        self.notes[idx].clear();
        self.active_hint = None;

        self.undo_stack.push(BoardDelta {
            cell,
            prev_value,
            next_value: None,
            prev_notes,
            next_notes: NotesMask::EMPTY,
            peer_note_changes: Vec::new(),
        });
        self.redo_stack.clear();
        Ok(true)
    }

    /// Reverts the last board/Notes change without altering Mistakes, Hints, or Earned Score tracking.
    pub fn undo(&mut self) -> Result<(), DomainError> {
        self.ensure_active()?;
        let delta = self.undo_stack.pop().ok_or(DomainError::NothingToUndo)?;
        let idx = delta.cell.as_usize();
        self.values[idx] = delta.prev_value;
        self.notes[idx] = delta.prev_notes;
        for peer_change in &delta.peer_note_changes {
            self.notes[peer_change.cell.as_usize()] = peer_change.before;
        }
        self.active_hint = None;
        self.redo_stack.push(delta);
        Ok(())
    }

    /// Reapplies the last undone board/Notes change.
    pub fn redo(&mut self) -> Result<(), DomainError> {
        self.ensure_active()?;
        let delta = self.redo_stack.pop().ok_or(DomainError::NothingToRedo)?;
        let idx = delta.cell.as_usize();
        self.values[idx] = delta.next_value;
        self.notes[idx] = delta.next_notes;
        for peer_change in &delta.peer_note_changes {
            self.notes[peer_change.cell.as_usize()] = peer_change.after;
        }
        self.active_hint = None;
        self.undo_stack.push(delta);
        if self.is_board_complete() {
            self.status = GameStatus::Completed;
        }
        Ok(())
    }

    /// Requests a Smart Hint for the current board state.
    ///
    /// Charges the Hint penalty once when a new suggestion is generated and marks any
    /// suggested cell as Hint-locked so it cannot award Earned Score even if entered manually.
    pub fn request_hint(&mut self) -> Result<HintSuggestion, DomainError> {
        self.ensure_active()?;
        if let Some(existing) = &self.active_hint {
            return Ok(existing.clone());
        }

        let suggestion =
            compute_smart_hint(&self.puzzle, &self.values).ok_or(DomainError::NoActiveHint)?;
        self.hints_used = self.hints_used.saturating_add(1);
        if let HintSuggestion::PlaceDigit { cell, .. } = &suggestion {
            self.hinted_cells[cell.as_usize()] = true;
        }
        self.active_hint = Some(suggestion.clone());
        Ok(suggestion)
    }

    /// Applies the currently active Smart Hint suggestion without charging a second penalty.
    pub fn apply_hint(&mut self) -> Result<MoveOutcome, DomainError> {
        self.ensure_active()?;
        let suggestion = self.active_hint.take().ok_or(DomainError::NoActiveHint)?;
        match suggestion {
            HintSuggestion::FixMistake { cell, .. } => {
                let _ = self.erase_cell(cell)?;
                Ok(MoveOutcome::Correct)
            }
            HintSuggestion::PlaceDigit { cell, digit, .. } => {
                self.hinted_cells[cell.as_usize()] = true;
                self.place_digit(cell, digit)
            }
        }
    }

    /// Dismisses the active Smart Hint banner (the recorded Hint penalty remains).
    pub fn dismiss_hint(&mut self) {
        self.active_hint = None;
    }

    /// Pauses an `Active` Game so Solve Time stops and the UI obscures the board.
    pub fn pause(&mut self) -> Result<(), DomainError> {
        match self.status {
            GameStatus::Active => {
                self.status = GameStatus::Paused;
                Ok(())
            }
            GameStatus::Paused => Ok(()),
            GameStatus::Completed | GameStatus::Abandoned => Err(DomainError::GameNotActive),
        }
    }

    /// Resumes a `Paused` Game.
    pub fn resume(&mut self) -> Result<(), DomainError> {
        match self.status {
            GameStatus::Paused => {
                self.status = GameStatus::Active;
                Ok(())
            }
            GameStatus::Active => Ok(()),
            GameStatus::Completed | GameStatus::Abandoned => Err(DomainError::GameNotPaused),
        }
    }

    /// Accumulates active elapsed Solve Time seconds while the Game is `Active`.
    pub fn add_solve_time(&mut self, delta_secs: u64) {
        if self.status == GameStatus::Active {
            self.solve_time_secs = self.solve_time_secs.saturating_add(delta_secs);
        }
    }

    /// Marks an in-progress Game as `Abandoned`.
    pub fn abandon(&mut self) {
        if matches!(self.status, GameStatus::Active | GameStatus::Paused) {
            self.status = GameStatus::Abandoned;
            self.active_hint = None;
        }
    }

    /// Builds an authoritative `GameSnapshot` for the thin UI layer.
    pub fn snapshot(&self) -> GameSnapshot {
        let mut correct_digit_counts = [0_u8; 9];
        let mut cells = Vec::with_capacity(CellIndex::BOARD_SIZE);

        for idx in 0..CellIndex::BOARD_SIZE {
            let value = self.values[idx];
            let is_given = self.puzzle.givens[idx].is_some();
            let sol = self.puzzle.solution[idx];
            let is_mistake = value.is_some_and(|d| d != sol);
            if let Some(d) = value {
                if !is_mistake {
                    correct_digit_counts[(d.get() - 1) as usize] += 1;
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            cells.push(CellSnapshot {
                index: idx as u8,
                value: value.map(Digit::get),
                is_given,
                is_mistake,
                is_hinted: self.hinted_cells[idx],
                notes: self.notes[idx]
                    .to_digits()
                    .into_iter()
                    .map(Digit::get)
                    .collect(),
            });
        }

        GameSnapshot {
            id: self.id,
            kind: self.kind,
            difficulty: self.puzzle.difficulty,
            status: self.status,
            cells,
            mistakes: self.mistakes,
            hints_used: self.hints_used,
            solve_time_secs: self.solve_time_secs,
            score: self.score_breakdown(),
            flawless: self.is_flawless(),
            active_hint: self.active_hint.clone(),
            can_undo: !self.undo_stack.is_empty(),
            can_redo: !self.redo_stack.is_empty(),
            correct_digit_counts,
        }
    }

    const fn ensure_active(&self) -> Result<(), DomainError> {
        if matches!(self.status, GameStatus::Active) {
            Ok(())
        } else {
            Err(DomainError::GameNotActive)
        }
    }

    fn is_board_complete(&self) -> bool {
        self.values
            .iter()
            .zip(self.puzzle.solution.iter())
            .all(|(val, &sol)| *val == Some(sol))
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::generator::generate_standard_puzzle;

    fn sample_game() -> Game {
        let mut rng = ChaCha8Rng::seed_from_u64(77);
        let puzzle = generate_standard_puzzle(Difficulty::Easy, &mut rng);
        Game::new(1, GameKind::Standard, puzzle)
    }

    fn first_editable_cell(game: &Game) -> CellIndex {
        let idx = game
            .puzzle()
            .givens
            .iter()
            .position(Option::is_none)
            .unwrap();
        CellIndex::new(idx).unwrap()
    }

    #[test]
    fn mistakes_are_permanent_across_erase_overwrite_and_undo() {
        let mut game = sample_game();
        let cell = first_editable_cell(&game);
        let correct = game.puzzle().solution_digit(cell).unwrap();
        let wrong1 = Digit::new(if correct.get() == 9 {
            1
        } else {
            correct.get() + 1
        })
        .unwrap();
        let wrong2 = Digit::new(if wrong1.get() == 9 {
            1
        } else {
            wrong1.get() + 1
        })
        .unwrap();
        let wrong2 = if wrong2 == correct {
            Digit::new(if wrong2.get() == 9 {
                1
            } else {
                wrong2.get() + 1
            })
            .unwrap()
        } else {
            wrong2
        };

        assert_eq!(
            game.place_digit(cell, wrong1).unwrap(),
            MoveOutcome::Mistake
        );
        assert_eq!(game.mistakes(), 1);

        // Overwrite with another wrong digit -> 2 Mistakes.
        assert_eq!(
            game.place_digit(cell, wrong2).unwrap(),
            MoveOutcome::Mistake
        );
        assert_eq!(game.mistakes(), 2);

        // Erase does not remove Mistakes.
        assert!(game.erase_cell(cell).unwrap());
        assert_eq!(game.mistakes(), 2);

        // Undo reverts board cell without removing Mistakes.
        game.undo().unwrap();
        assert_eq!(game.mistakes(), 2);
        assert!(!game.is_flawless());
    }

    #[test]
    fn cell_earned_score_is_awarded_at_most_once_and_never_on_hint() {
        let mut game = sample_game();
        let cell = first_editable_cell(&game);
        let correct = game.puzzle().solution_digit(cell).unwrap();

        assert_eq!(
            game.place_digit(cell, correct).unwrap(),
            MoveOutcome::Correct
        );
        assert_eq!(game.score_breakdown().earned_score, 50);

        // Erase and re-enter must not double-award Earned Score.
        game.erase_cell(cell).unwrap();
        assert_eq!(
            game.place_digit(cell, correct).unwrap(),
            MoveOutcome::Correct
        );
        assert_eq!(game.score_breakdown().earned_score, 50);

        // Request and apply Smart Hint -> increments hints_used, awards 0 Earned Score.
        let hint = game.request_hint().unwrap();
        assert!(matches!(hint, HintSuggestion::PlaceDigit { .. }));
        assert_eq!(game.hints_used(), 1);
        game.apply_hint().unwrap();
        assert_eq!(game.hints_used(), 1);
        assert_eq!(game.score_breakdown().earned_score, 50);
        assert_eq!(game.score_breakdown().hint_penalty, 150);
    }

    #[test]
    fn correct_placement_auto_clears_peer_notes_and_undo_restores_them() {
        let mut game = sample_game();
        let cell = first_editable_cell(&game);
        let correct = game.puzzle().solution_digit(cell).unwrap();
        let editable_peer = cell
            .peers()
            .into_iter()
            .find(|&p| !game.puzzle().is_given(p))
            .unwrap();

        assert!(game.toggle_note(editable_peer, correct).unwrap());
        assert!(game.snapshot().cells[editable_peer.as_usize()]
            .notes
            .contains(&correct.get()));

        assert_eq!(
            game.place_digit(cell, correct).unwrap(),
            MoveOutcome::Correct
        );
        assert!(!game.snapshot().cells[editable_peer.as_usize()]
            .notes
            .contains(&correct.get()));

        game.undo().unwrap();
        assert!(game.snapshot().cells[editable_peer.as_usize()]
            .notes
            .contains(&correct.get()));
    }

    #[test]
    fn cannot_modify_given_cell_or_paused_game() {
        let mut game = sample_game();
        let given_idx = game
            .puzzle()
            .givens
            .iter()
            .position(Option::is_some)
            .unwrap();
        let given_cell = CellIndex::new(given_idx).unwrap();
        assert!(matches!(
            game.place_digit(given_cell, Digit::new(1).unwrap()),
            Err(DomainError::CellIsGiven(_))
        ));

        let editable = first_editable_cell(&game);
        game.pause().unwrap();
        assert!(matches!(
            game.place_digit(editable, Digit::new(1).unwrap()),
            Err(DomainError::GameNotActive)
        ));
        game.add_solve_time(30);
        assert_eq!(game.solve_time_secs(), 0);
        game.resume().unwrap();
        game.add_solve_time(12);
        assert_eq!(game.solve_time_secs(), 12);
    }
}
