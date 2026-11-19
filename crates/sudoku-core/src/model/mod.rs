//! Core Sudoku domain value types and entities.

pub mod date;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

pub use date::{DailyDate, Weekday};

/// Validated board cell index in `0..81` (row-major order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CellIndex(u8);

impl CellIndex {
    /// Total number of cells on a standard 9×9 Sudoku board.
    pub const BOARD_SIZE: usize = 81;

    /// Constructs a `CellIndex` from `0..81`.
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(index: usize) -> Result<Self, DomainError> {
        if index < Self::BOARD_SIZE {
            Ok(Self(index as u8))
        } else {
            Err(DomainError::InvalidCellIndex(index))
        }
    }

    /// Constructs a `CellIndex` from zero-based `(row, col)` coordinates (`0..9`).
    pub fn from_row_col(row: usize, col: usize) -> Result<Self, DomainError> {
        if row < 9 && col < 9 {
            Self::new(row * 9 + col)
        } else {
            Err(DomainError::InvalidCellIndex(row * 9 + col))
        }
    }

    /// Returns the index as `usize`.
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Returns the raw `u8` index (`0..81`).
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    /// Zero-based row (`0..9`).
    pub const fn row(self) -> usize {
        (self.0 / 9) as usize
    }

    /// Zero-based column (`0..9`).
    pub const fn col(self) -> usize {
        (self.0 % 9) as usize
    }

    /// Zero-based 3×3 box index (`0..9`).
    pub const fn box_index(self) -> usize {
        (self.row() / 3) * 3 + (self.col() / 3)
    }

    /// Returns the 20 peer cells sharing the same row, column, or 3×3 box.
    pub fn peers(self) -> [Self; 20] {
        let mut peers = [Self(0); 20];
        let mut write = 0;
        let r = self.row();
        let c = self.col();
        let b = self.box_index();
        let mut i: u8 = 0;
        while i < 81 {
            let candidate = Self(i);
            if candidate != self
                && (candidate.row() == r || candidate.col() == c || candidate.box_index() == b)
            {
                peers[write] = candidate;
                write += 1;
            }
            i += 1;
        }
        peers
    }
}

/// Validated Sudoku digit in `1..=9`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Digit(u8);

impl Digit {
    /// All nine valid Sudoku digits in ascending order.
    pub const ALL: [Self; 9] = [
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self(5),
        Self(6),
        Self(7),
        Self(8),
        Self(9),
    ];

    /// Constructs a `Digit` from `1..=9`.
    pub fn new(value: u8) -> Result<Self, DomainError> {
        if (1..=9).contains(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::InvalidDigit(value))
        }
    }

    /// Returns the raw digit (`1..=9`).
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Returns the single-bit mask `1 << (digit - 1)` in `0..9`.
    pub const fn bit(self) -> u16 {
        1_u16 << (self.0 - 1)
    }
}

/// Compact 9-bit bitmask storing player Notes (candidate digits 1–9) for a cell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NotesMask(u16);

impl NotesMask {
    /// Empty Notes mask.
    pub const EMPTY: Self = Self(0);

    /// Full candidate mask containing digits `1..=9`.
    pub const ALL: Self = Self(0b1_1111_1111);

    /// Returns true if `digit` is marked in this mask.
    pub const fn contains(self, digit: Digit) -> bool {
        (self.0 & digit.bit()) != 0
    }

    /// Inserts `digit` into the mask.
    pub fn insert(&mut self, digit: Digit) {
        self.0 |= digit.bit();
    }

    /// Removes `digit` from the mask. Returns `true` if the bit was present.
    pub fn remove(&mut self, digit: Digit) -> bool {
        let was_present = self.contains(digit);
        self.0 &= !digit.bit();
        was_present
    }

    /// Toggles `digit` in the mask.
    pub fn toggle(&mut self, digit: Digit) {
        self.0 ^= digit.bit();
    }

    /// Clears all candidate bits.
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    /// Returns the number of candidate digits currently marked.
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns `true` when no candidate digits are marked.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the raw bitmask (bits 0..9 correspond to digits 1..=9).
    pub const fn bits(self) -> u16 {
        self.0
    }

    /// Returns marked digits in ascending order (`1..=9`).
    pub fn to_digits(self) -> Vec<Digit> {
        Digit::ALL
            .into_iter()
            .filter(|&d| self.contains(d))
            .collect()
    }

    /// Returns the single marked digit if and only if `count() == 1`.
    pub fn singleton(self) -> Option<Digit> {
        if self.count() == 1 {
            #[allow(clippy::cast_possible_truncation)]
            let value = (self.0.trailing_zeros() as u8) + 1;
            Digit::new(value).ok()
        } else {
            None
        }
    }
}

/// Skill rating of a Puzzle, determined by the hardest human solving technique required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    /// All four canonical Difficulty tiers in ascending order.
    pub const ALL: [Self; 4] = [Self::Easy, Self::Medium, Self::Hard, Self::Expert];
}

/// Distinguishes a Standard Game from a date-seeded Daily Challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "date")]
pub enum GameKind {
    Standard,
    Daily(DailyDate),
}

/// Lifecycle state of a Game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameStatus {
    Active,
    Paused,
    Completed,
    Abandoned,
}

/// A 9×9 Sudoku starting grid with a guaranteed unique solution and assigned Difficulty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Puzzle {
    /// Pre-filled Given cells (`None` for editable empty cells).
    pub givens: Vec<Option<Digit>>,
    /// The Puzzle's unique valid solution (length 81).
    pub solution: Vec<Digit>,
    /// Assigned human-technique Difficulty.
    pub difficulty: Difficulty,
}

impl Puzzle {
    /// Returns true if `cell` is a pre-filled Given.
    pub fn is_given(&self, cell: CellIndex) -> bool {
        self.givens.get(cell.as_usize()).and_then(|v| *v).is_some()
    }

    /// Returns the unique solution digit for `cell`.
    pub fn solution_digit(&self, cell: CellIndex) -> Option<Digit> {
        self.solution.get(cell.as_usize()).copied()
    }

    /// Counts the number of pre-filled Givens in this Puzzle.
    pub fn given_count(&self) -> usize {
        self.givens.iter().filter(|cell| cell.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_index_has_twenty_unique_peers() {
        let cell = CellIndex::from_row_col(4, 5).unwrap();
        let peers = cell.peers();
        assert_eq!(peers.len(), 20);
        for (i, &p) in peers.iter().enumerate() {
            assert_ne!(p, cell);
            assert!(
                p.row() == cell.row() || p.col() == cell.col() || p.box_index() == cell.box_index()
            );
            for &other in &peers[i + 1..] {
                assert_ne!(p, other);
            }
        }
    }

    #[test]
    fn cell_index_and_digit_reject_out_of_range() {
        assert!(CellIndex::new(81).is_err());
        assert!(CellIndex::from_row_col(9, 0).is_err());
        assert!(Digit::new(0).is_err());
        assert!(Digit::new(10).is_err());
    }

    #[test]
    fn notes_mask_operations_are_exact() {
        let mut mask = NotesMask::EMPTY;
        let d3 = Digit::new(3).unwrap();
        let d7 = Digit::new(7).unwrap();

        assert!(mask.is_empty());
        mask.toggle(d3);
        assert_eq!(mask.singleton(), Some(d3));
        mask.insert(d7);
        assert_eq!(mask.singleton(), None);
        assert_eq!(mask.to_digits(), vec![d3, d7]);
        assert!(mask.remove(d3));
        assert!(!mask.remove(d3));
        assert_eq!(mask.singleton(), Some(d7));
        mask.clear();
        assert!(mask.is_empty());
    }
}
