//! Bitmask-accelerated MRV exact Sudoku solver and complete-grid generator.

use rand::seq::SliceRandom;
use rand::Rng;

use crate::model::{CellIndex, Digit};

/// Result of counting valid solutions for a 9×9 Sudoku grid (capped at 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolutionCount {
    /// The grid is contradictory or unsolvable.
    None,
    /// The grid has exactly one unique valid solution.
    Unique(Vec<Digit>),
    /// The grid has two or more valid solutions.
    Multiple,
}

#[derive(Debug, Clone)]
struct BitGrid {
    cells: [u8; 81],
    rows: [u16; 9],
    cols: [u16; 9],
    boxes: [u16; 9],
}

impl BitGrid {
    const ALL_DIGITS: u16 = 0b1_1111_1111;

    fn from_option_digits(grid: &[Option<Digit>]) -> Option<Self> {
        if grid.len() != CellIndex::BOARD_SIZE {
            return None;
        }
        let mut state = Self {
            cells: [0; 81],
            rows: [0; 9],
            cols: [0; 9],
            boxes: [0; 9],
        };
        for (idx, cell) in grid.iter().enumerate() {
            if let Some(digit) = cell {
                if !state.place(idx, digit.get()) {
                    return None;
                }
            }
        }
        Some(state)
    }

    const fn empty() -> Self {
        Self {
            cells: [0; 81],
            rows: [0; 9],
            cols: [0; 9],
            boxes: [0; 9],
        }
    }

    const fn box_idx(idx: usize) -> usize {
        ((idx / 9) / 3) * 3 + ((idx % 9) / 3)
    }

    const fn candidates(&self, idx: usize) -> u16 {
        let r = idx / 9;
        let c = idx % 9;
        let b = Self::box_idx(idx);
        Self::ALL_DIGITS & !(self.rows[r] | self.cols[c] | self.boxes[b])
    }

    fn place(&mut self, idx: usize, digit: u8) -> bool {
        let bit = 1_u16 << (digit - 1);
        let r = idx / 9;
        let c = idx % 9;
        let b = Self::box_idx(idx);
        if (self.rows[r] | self.cols[c] | self.boxes[b]) & bit != 0 {
            return false;
        }
        self.cells[idx] = digit;
        self.rows[r] |= bit;
        self.cols[c] |= bit;
        self.boxes[b] |= bit;
        true
    }

    fn remove(&mut self, idx: usize, digit: u8) {
        let bit = !(1_u16 << (digit - 1));
        let r = idx / 9;
        let c = idx % 9;
        let b = Self::box_idx(idx);
        self.cells[idx] = 0;
        self.rows[r] &= bit;
        self.cols[c] &= bit;
        self.boxes[b] &= bit;
    }

    /// Selects the empty cell with the fewest legal candidates (Minimum Remaining Values).
    /// Returns `Ok(None)` when solved, `Ok(Some((idx, mask)))` when solvable next step exists,
    /// or `Err(())` on dead end (an empty cell has zero candidates).
    fn select_mrv_cell(&self) -> Result<Option<(usize, u16)>, ()> {
        let mut best: Option<(usize, u16, u32)> = None;
        for idx in 0..81 {
            if self.cells[idx] != 0 {
                continue;
            }
            let mask = self.candidates(idx);
            let count = mask.count_ones();
            if count == 0 {
                return Err(());
            }
            if count == 1 {
                return Ok(Some((idx, mask)));
            }
            if best.is_none_or(|(_, _, best_count)| count < best_count) {
                best = Some((idx, mask, count));
            }
        }
        Ok(best.map(|(idx, mask, _)| (idx, mask)))
    }

    fn to_digits(&self) -> Option<Vec<Digit>> {
        self.cells.iter().map(|&d| Digit::new(d).ok()).collect()
    }
}

/// Counts solutions of `grid`, stopping early once 2 solutions are found.
pub fn solve_unique(grid: &[Option<Digit>]) -> SolutionCount {
    let Some(mut state) = BitGrid::from_option_digits(grid) else {
        return SolutionCount::None;
    };

    let mut solutions: Vec<[u8; 81]> = Vec::with_capacity(2);
    search_solutions(&mut state, &mut solutions);

    match solutions.len() {
        0 => SolutionCount::None,
        1 => {
            let solved = solutions[0]
                .iter()
                .map(|&d| Digit::new(d).ok())
                .collect::<Option<Vec<Digit>>>();
            solved.map_or(SolutionCount::None, SolutionCount::Unique)
        }
        _ => SolutionCount::Multiple,
    }
}

fn search_solutions(state: &mut BitGrid, solutions: &mut Vec<[u8; 81]>) {
    if solutions.len() >= 2 {
        return;
    }
    let Ok(next) = state.select_mrv_cell() else {
        return;
    };
    let Some((idx, mut mask)) = next else {
        solutions.push(state.cells);
        return;
    };

    while mask != 0 {
        #[allow(clippy::cast_possible_truncation)]
        let digit = (mask.trailing_zeros() as u8) + 1;
        mask &= mask - 1;
        if state.place(idx, digit) {
            search_solutions(state, solutions);
            state.remove(idx, digit);
            if solutions.len() >= 2 {
                return;
            }
        }
    }
}

/// Generates a uniformly valid complete 9×9 Sudoku solution grid using `rng`.
pub fn generate_complete_grid<R: Rng>(rng: &mut R) -> Vec<Digit> {
    let mut state = BitGrid::empty();
    let _ = fill_random(&mut state, rng);
    state.to_digits().unwrap_or_default()
}

fn fill_random<R: Rng>(state: &mut BitGrid, rng: &mut R) -> bool {
    let Ok(next) = state.select_mrv_cell() else {
        return false;
    };
    let Some((idx, mask)) = next else {
        return true;
    };

    let mut digits = [0_u8; 9];
    let mut len = 0;
    let mut m = mask;
    while m != 0 {
        #[allow(clippy::cast_possible_truncation)]
        let digit = (m.trailing_zeros() as u8) + 1;
        m &= m - 1;
        digits[len] = digit;
        len += 1;
    }
    digits[..len].shuffle(rng);

    for &digit in &digits[..len] {
        if state.place(idx, digit) {
            if fill_random(state, rng) {
                return true;
            }
            state.remove(idx, digit);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;

    fn parse_grid(s: &str) -> Vec<Option<Digit>> {
        s.chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| {
                if c == '.' || c == '0' {
                    None
                } else {
                    Some(Digit::new(c as u8 - b'0').unwrap())
                }
            })
            .collect()
    }

    #[test]
    fn solves_classic_unique_puzzle() {
        let grid = parse_grid(
            "53..7....\
             6..195...\
             .98....6.\
             8...6...3\
             4..8.3..1\
             7...2...6\
             .6....28.\
             ...419..5\
             ....8..79",
        );
        let result = solve_unique(&grid);
        assert!(matches!(result, SolutionCount::Unique(_)));
    }

    #[test]
    fn detects_multiple_solutions_on_underconstrained_grid() {
        let grid = vec![None; 81];
        assert_eq!(solve_unique(&grid), SolutionCount::Multiple);
    }

    #[test]
    fn detects_contradictory_grid_as_none() {
        let mut grid = vec![None; 81];
        grid[0] = Some(Digit::new(5).unwrap());
        grid[1] = Some(Digit::new(5).unwrap());
        assert_eq!(solve_unique(&grid), SolutionCount::None);
    }

    #[test]
    fn generates_valid_complete_grid() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let solved = generate_complete_grid(&mut rng);
        assert_eq!(solved.len(), 81);
        let as_opts: Vec<Option<Digit>> = solved.iter().copied().map(Some).collect();
        assert_eq!(solve_unique(&as_opts), SolutionCount::Unique(solved));
    }
}
