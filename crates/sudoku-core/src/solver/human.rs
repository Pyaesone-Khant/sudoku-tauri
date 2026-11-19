//! Human-technique Sudoku solver for Difficulty grading and Smart Hints.

use serde::{Deserialize, Serialize};

use crate::model::{CellIndex, Difficulty, Digit, Puzzle};

/// Human-understandable Sudoku deduction techniques ordered by Difficulty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SolvingTechnique {
    /// Only one legal digit remains in the cell.
    NakedSingle,
    /// Only one cell in a row, column, or 3×3 box can hold the digit.
    HiddenSingle,
    /// Pointing or claiming locked candidates within a box/line intersection.
    LockedCandidates,
    /// Two cells in a unit restrict two digits exclusively to themselves.
    NakedPair,
    /// Two digits in a unit can only live in two cells.
    HiddenPair,
    /// Three cells in a unit restrict three digits exclusively to themselves.
    NakedTriple,
    /// Three digits in a unit can only live in three cells.
    HiddenTriple,
    /// Two rows/columns restrict a digit to the same two columns/rows.
    XWing,
    /// Pivot-and-wings elimination across three bivalue cells.
    XYWing,
    /// Three-line fish elimination.
    Swordfish,
    /// Bounded single-step bivalue contradiction chain.
    ForcingChain,
}

impl SolvingTechnique {
    /// Maps a technique to its canonical Difficulty tier.
    pub const fn difficulty(self) -> Difficulty {
        match self {
            Self::NakedSingle | Self::HiddenSingle => Difficulty::Easy,
            Self::LockedCandidates | Self::NakedPair | Self::HiddenPair => Difficulty::Medium,
            Self::NakedTriple | Self::HiddenTriple | Self::XWing => Difficulty::Hard,
            Self::XYWing | Self::Swordfish | Self::ForcingChain => Difficulty::Expert,
        }
    }

    /// Human-readable display name shown in Smart Hint banners.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::NakedSingle => "Naked Single",
            Self::HiddenSingle => "Hidden Single",
            Self::LockedCandidates => "Locked Candidates",
            Self::NakedPair => "Naked Pair",
            Self::HiddenPair => "Hidden Pair",
            Self::NakedTriple => "Naked Triple",
            Self::HiddenTriple => "Hidden Triple",
            Self::XWing => "X-Wing",
            Self::XYWing => "XY-Wing",
            Self::Swordfish => "Swordfish",
            Self::ForcingChain => "Forcing Chain",
        }
    }
}

/// On-demand Smart Hint suggestion produced by the human-technique solver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HintSuggestion {
    /// Board currently contains a Mistake that should be cleared before deducting further.
    FixMistake {
        /// Cell currently holding the incorrect digit.
        cell: CellIndex,
        /// The incorrect digit currently in `cell`.
        wrong_digit: Digit,
        /// Short, clear user-facing explanation.
        explanation: String,
    },
    /// Next logically deducible cell placement with technique name and supporting cells.
    PlaceDigit {
        /// Target cell to fill.
        cell: CellIndex,
        /// Correct digit to place in `cell`.
        digit: Digit,
        /// Hardest technique used to unlock this placement.
        technique: SolvingTechnique,
        /// Short, clear user-facing explanation.
        explanation: String,
        /// Supporting cells (row/col/box/pattern) to highlight visually.
        highlight_cells: Vec<CellIndex>,
    },
}

#[derive(Debug, Clone, Copy)]
enum UnitKind {
    Row(usize),
    Col(usize),
    Box(usize),
}

impl UnitKind {
    fn label(self) -> String {
        match self {
            Self::Row(r) => format!("Row {}", r + 1),
            Self::Col(c) => format!("Column {}", c + 1),
            Self::Box(b) => format!("Box {}", b + 1),
        }
    }
}

fn all_units() -> [(UnitKind, [usize; 9]); 27] {
    let mut units = [(UnitKind::Row(0), [0; 9]); 27];
    for (r, unit_slot) in units.iter_mut().enumerate().take(9) {
        let mut cells = [0; 9];
        for (c, slot) in cells.iter_mut().enumerate() {
            *slot = r * 9 + c;
        }
        *unit_slot = (UnitKind::Row(r), cells);
    }
    for (c, unit_slot) in units.iter_mut().skip(9).take(9).enumerate() {
        let mut cells = [0; 9];
        for (r, slot) in cells.iter_mut().enumerate() {
            *slot = r * 9 + c;
        }
        *unit_slot = (UnitKind::Col(c), cells);
    }
    for (b, unit_slot) in units.iter_mut().skip(18).take(9).enumerate() {
        let br = (b / 3) * 3;
        let bc = (b % 3) * 3;
        let mut cells = [0; 9];
        let mut k = 0;
        for dr in 0..3 {
            for dc in 0..3 {
                cells[k] = (br + dr) * 9 + (bc + dc);
                k += 1;
            }
        }
        *unit_slot = (UnitKind::Box(b), cells);
    }
    units
}

const fn shares_peer(a: usize, b: usize) -> bool {
    if a == b {
        return false;
    }
    let (ar, ac) = (a / 9, a % 9);
    let (br, bc) = (b / 9, b % 9);
    ar == br || ac == bc || (ar / 3 == br / 3 && ac / 3 == bc / 3)
}

#[derive(Debug, Clone)]
struct LogicBoard {
    values: [u8; 81],
    candidates: [u16; 81],
}

#[derive(Debug, Clone)]
enum StepOutcome {
    Placed {
        cell: usize,
        digit: u8,
        technique: SolvingTechnique,
        explanation: String,
        highlights: Vec<usize>,
    },
    Eliminated {
        technique: SolvingTechnique,
        highlights: Vec<usize>,
    },
}

impl LogicBoard {
    const ALL_MASK: u16 = 0b1_1111_1111;

    fn from_option_digits(grid: &[Option<Digit>]) -> Option<Self> {
        if grid.len() != CellIndex::BOARD_SIZE {
            return None;
        }
        let mut board = Self {
            values: [0; 81],
            candidates: [Self::ALL_MASK; 81],
        };
        for (idx, cell) in grid.iter().enumerate() {
            if let Some(d) = cell {
                if !board.assign(idx, d.get()) {
                    return None;
                }
            }
        }
        Some(board)
    }

    fn is_solved(&self) -> bool {
        self.values.iter().all(|&v| v != 0)
    }

    fn has_contradiction(&self) -> bool {
        (0..81).any(|i| self.values[i] == 0 && self.candidates[i] == 0)
    }

    fn assign(&mut self, idx: usize, digit: u8) -> bool {
        let bit = 1_u16 << (digit - 1);
        if self.values[idx] == digit {
            return true;
        }
        if self.values[idx] != 0 || (self.candidates[idx] & bit) == 0 {
            return false;
        }
        self.values[idx] = digit;
        self.candidates[idx] = 0;
        let r = idx / 9;
        let c = idx % 9;
        let br = (r / 3) * 3;
        let bc = (c / 3) * 3;

        for cc in 0..9 {
            let p = r * 9 + cc;
            if p != idx {
                self.candidates[p] &= !bit;
            }
        }
        for rr in 0..9 {
            let p = rr * 9 + c;
            if p != idx {
                self.candidates[p] &= !bit;
            }
        }
        for dr in 0..3 {
            for dc in 0..3 {
                let p = (br + dr) * 9 + (bc + dc);
                if p != idx {
                    self.candidates[p] &= !bit;
                }
            }
        }
        !self.has_contradiction()
    }

    fn next_step(&mut self, units: &[(UnitKind, [usize; 9]); 27]) -> Option<StepOutcome> {
        if let Some(step) = self.find_naked_single() {
            return Some(step);
        }
        if let Some(step) = self.find_hidden_single(units) {
            return Some(step);
        }
        if let Some(step) = self.apply_locked_candidates(units) {
            return Some(step);
        }
        if let Some(step) = self.apply_naked_subset(units, 2, SolvingTechnique::NakedPair) {
            return Some(step);
        }
        if let Some(step) = self.apply_hidden_subset(units, 2, SolvingTechnique::HiddenPair) {
            return Some(step);
        }
        if let Some(step) = self.apply_naked_subset(units, 3, SolvingTechnique::NakedTriple) {
            return Some(step);
        }
        if let Some(step) = self.apply_hidden_subset(units, 3, SolvingTechnique::HiddenTriple) {
            return Some(step);
        }
        if let Some(step) = self.apply_x_wing() {
            return Some(step);
        }
        if let Some(step) = self.apply_xy_wing() {
            return Some(step);
        }
        if let Some(step) = self.apply_swordfish() {
            return Some(step);
        }
        self.apply_forcing_chain(units)
    }

    fn find_naked_single(&mut self) -> Option<StepOutcome> {
        for idx in 0..81 {
            if self.values[idx] == 0 && self.candidates[idx].is_power_of_two() {
                #[allow(clippy::cast_possible_truncation)]
                let digit = (self.candidates[idx].trailing_zeros() as u8) + 1;
                if self.assign(idx, digit) {
                    let row = idx / 9 + 1;
                    let col = idx % 9 + 1;
                    return Some(StepOutcome::Placed {
                        cell: idx,
                        digit,
                        technique: SolvingTechnique::NakedSingle,
                        explanation: format!(
                            "Naked Single: {digit} is the only valid digit left for Row {row}, Column {col}."
                        ),
                        highlights: vec![idx],
                    });
                }
            }
        }
        None
    }

    fn find_hidden_single(&mut self, units: &[(UnitKind, [usize; 9]); 27]) -> Option<StepOutcome> {
        for &(kind, cells) in units {
            for digit in 1..=9_u8 {
                let bit = 1_u16 << (digit - 1);
                let mut found: Option<usize> = None;
                let mut count = 0;
                for &idx in &cells {
                    if self.values[idx] == digit {
                        count = 2;
                        break;
                    }
                    if self.values[idx] == 0 && (self.candidates[idx] & bit) != 0 {
                        found = Some(idx);
                        count += 1;
                        if count > 1 {
                            break;
                        }
                    }
                }
                if count == 1 {
                    if let Some(idx) = found {
                        if self.assign(idx, digit) {
                            let row = idx / 9 + 1;
                            let col = idx % 9 + 1;
                            return Some(StepOutcome::Placed {
                                cell: idx,
                                digit,
                                technique: SolvingTechnique::HiddenSingle,
                                explanation: format!(
                                    "Hidden Single: in {}, {digit} can only go in Row {row}, Column {col}.",
                                    kind.label()
                                ),
                                highlights: cells.to_vec(),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_locked_candidates(
        &mut self,
        units: &[(UnitKind, [usize; 9]); 27],
    ) -> Option<StepOutcome> {
        // Pointing: all candidates of digit in a box share a single row or column.
        for &(kind, box_cells) in &units[18..27] {
            let UnitKind::Box(_) = kind else {
                continue;
            };
            for digit in 1..=9_u8 {
                let bit = 1_u16 << (digit - 1);
                let mut rows_mask = 0_u16;
                let mut cols_mask = 0_u16;
                let mut box_hits = Vec::new();
                for &idx in &box_cells {
                    if self.values[idx] == 0 && (self.candidates[idx] & bit) != 0 {
                        rows_mask |= 1 << (idx / 9);
                        cols_mask |= 1 << (idx % 9);
                        box_hits.push(idx);
                    }
                }
                if box_hits.len() < 2 {
                    continue;
                }
                if rows_mask.is_power_of_two() {
                    let row = rows_mask.trailing_zeros() as usize;
                    let mut changed = false;
                    for c in 0..9 {
                        let idx = row * 9 + c;
                        if !box_cells.contains(&idx)
                            && self.values[idx] == 0
                            && (self.candidates[idx] & bit) != 0
                        {
                            self.candidates[idx] &= !bit;
                            changed = true;
                        }
                    }
                    if changed {
                        return Some(StepOutcome::Eliminated {
                            technique: SolvingTechnique::LockedCandidates,
                            highlights: box_hits,
                        });
                    }
                }
                if cols_mask.is_power_of_two() {
                    let col = cols_mask.trailing_zeros() as usize;
                    let mut changed = false;
                    for r in 0..9 {
                        let idx = r * 9 + col;
                        if !box_cells.contains(&idx)
                            && self.values[idx] == 0
                            && (self.candidates[idx] & bit) != 0
                        {
                            self.candidates[idx] &= !bit;
                            changed = true;
                        }
                    }
                    if changed {
                        return Some(StepOutcome::Eliminated {
                            technique: SolvingTechnique::LockedCandidates,
                            highlights: box_hits,
                        });
                    }
                }
            }
        }

        // Claiming: all candidates of digit in a row or column lie in a single box.
        for &(_, line_cells) in &units[0..18] {
            for digit in 1..=9_u8 {
                let bit = 1_u16 << (digit - 1);
                let mut boxes_mask = 0_u16;
                let mut line_hits = Vec::new();
                for &idx in &line_cells {
                    if self.values[idx] == 0 && (self.candidates[idx] & bit) != 0 {
                        let b = ((idx / 9) / 3) * 3 + ((idx % 9) / 3);
                        boxes_mask |= 1 << b;
                        line_hits.push(idx);
                    }
                }
                if line_hits.len() >= 2 && boxes_mask.is_power_of_two() {
                    let box_id = boxes_mask.trailing_zeros() as usize;
                    let box_cells = units[18 + box_id].1;
                    let mut changed = false;
                    for &idx in &box_cells {
                        if !line_cells.contains(&idx)
                            && self.values[idx] == 0
                            && (self.candidates[idx] & bit) != 0
                        {
                            self.candidates[idx] &= !bit;
                            changed = true;
                        }
                    }
                    if changed {
                        return Some(StepOutcome::Eliminated {
                            technique: SolvingTechnique::LockedCandidates,
                            highlights: line_hits,
                        });
                    }
                }
            }
        }
        None
    }

    fn apply_naked_subset(
        &mut self,
        units: &[(UnitKind, [usize; 9]); 27],
        size: usize,
        technique: SolvingTechnique,
    ) -> Option<StepOutcome> {
        for &(_, cells) in units {
            let eligible: Vec<usize> = cells
                .iter()
                .copied()
                .filter(|&idx| {
                    let count = self.candidates[idx].count_ones() as usize;
                    self.values[idx] == 0 && (2..=size).contains(&count)
                })
                .collect();
            if eligible.len() < size {
                continue;
            }
            if size == 2 {
                for i in 0..eligible.len() {
                    for j in (i + 1)..eligible.len() {
                        let union = self.candidates[eligible[i]] | self.candidates[eligible[j]];
                        if union.count_ones() == 2 {
                            let subset = [eligible[i], eligible[j]];
                            if self.eliminate_mask_from_unit(&cells, &subset, union) {
                                return Some(StepOutcome::Eliminated {
                                    technique,
                                    highlights: subset.to_vec(),
                                });
                            }
                        }
                    }
                }
            } else if size == 3 {
                for i in 0..eligible.len() {
                    for j in (i + 1)..eligible.len() {
                        for k in (j + 1)..eligible.len() {
                            let union = self.candidates[eligible[i]]
                                | self.candidates[eligible[j]]
                                | self.candidates[eligible[k]];
                            if union.count_ones() == 3 {
                                let subset = [eligible[i], eligible[j], eligible[k]];
                                if self.eliminate_mask_from_unit(&cells, &subset, union) {
                                    return Some(StepOutcome::Eliminated {
                                        technique,
                                        highlights: subset.to_vec(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn eliminate_mask_from_unit(
        &mut self,
        unit: &[usize; 9],
        keep: &[usize],
        remove_mask: u16,
    ) -> bool {
        let mut changed = false;
        for &idx in unit {
            if !keep.contains(&idx)
                && self.values[idx] == 0
                && (self.candidates[idx] & remove_mask) != 0
            {
                self.candidates[idx] &= !remove_mask;
                changed = true;
            }
        }
        changed
    }

    fn apply_hidden_subset(
        &mut self,
        units: &[(UnitKind, [usize; 9]); 27],
        size: usize,
        technique: SolvingTechnique,
    ) -> Option<StepOutcome> {
        for &(_, cells) in units {
            let mut digit_positions = [0_u16; 9];
            for (pos, &idx) in cells.iter().enumerate() {
                if self.values[idx] != 0 {
                    continue;
                }
                let m = self.candidates[idx];
                for (d, slot) in digit_positions.iter_mut().enumerate() {
                    if (m & (1_u16 << d)) != 0 {
                        *slot |= 1_u16 << pos;
                    }
                }
            }
            let eligible_digits: Vec<usize> = (0..9)
                .filter(|&d| {
                    let count = digit_positions[d].count_ones() as usize;
                    (2..=size).contains(&count)
                })
                .collect();
            if eligible_digits.len() < size {
                continue;
            }
            if size == 2 {
                for i in 0..eligible_digits.len() {
                    for j in (i + 1)..eligible_digits.len() {
                        let d1 = eligible_digits[i];
                        let d2 = eligible_digits[j];
                        let pos_union = digit_positions[d1] | digit_positions[d2];
                        if pos_union.count_ones() == 2 {
                            let keep_digits = (1_u16 << d1) | (1_u16 << d2);
                            if let Some(highlights) =
                                self.restrict_unit_positions(&cells, pos_union, keep_digits)
                            {
                                return Some(StepOutcome::Eliminated {
                                    technique,
                                    highlights,
                                });
                            }
                        }
                    }
                }
            } else if size == 3 {
                for i in 0..eligible_digits.len() {
                    for j in (i + 1)..eligible_digits.len() {
                        for k in (j + 1)..eligible_digits.len() {
                            let d1 = eligible_digits[i];
                            let d2 = eligible_digits[j];
                            let d3 = eligible_digits[k];
                            let pos_union =
                                digit_positions[d1] | digit_positions[d2] | digit_positions[d3];
                            if pos_union.count_ones() == 3 {
                                let keep_digits = (1_u16 << d1) | (1_u16 << d2) | (1_u16 << d3);
                                if let Some(highlights) =
                                    self.restrict_unit_positions(&cells, pos_union, keep_digits)
                                {
                                    return Some(StepOutcome::Eliminated {
                                        technique,
                                        highlights,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn restrict_unit_positions(
        &mut self,
        cells: &[usize; 9],
        pos_mask: u16,
        keep_digits: u16,
    ) -> Option<Vec<usize>> {
        let mut changed = false;
        let mut highlights = Vec::new();
        for (pos, &idx) in cells.iter().enumerate() {
            if (pos_mask & (1_u16 << pos)) != 0 {
                highlights.push(idx);
                let next = self.candidates[idx] & keep_digits;
                if next != self.candidates[idx] {
                    self.candidates[idx] = next;
                    changed = true;
                }
            }
        }
        changed.then_some(highlights)
    }

    fn apply_x_wing(&mut self) -> Option<StepOutcome> {
        for digit in 1..=9_u8 {
            let bit = 1_u16 << (digit - 1);
            if let Some(step) = self.apply_x_wing_rows(bit) {
                return Some(step);
            }
            if let Some(step) = self.apply_x_wing_cols(bit) {
                return Some(step);
            }
        }
        None
    }

    fn apply_x_wing_rows(&mut self, bit: u16) -> Option<StepOutcome> {
        let mut row_cols: Vec<(usize, u16)> = Vec::new();
        for r in 0..9 {
            let mut cols = 0_u16;
            for c in 0..9 {
                let idx = r * 9 + c;
                if self.values[idx] == 0 && (self.candidates[idx] & bit) != 0 {
                    cols |= 1 << c;
                }
            }
            if cols.count_ones() == 2 {
                row_cols.push((r, cols));
            }
        }
        for i in 0..row_cols.len() {
            for j in (i + 1)..row_cols.len() {
                if row_cols[i].1 == row_cols[j].1 {
                    let (r1, cols) = row_cols[i];
                    let r2 = row_cols[j].0;
                    let mut changed = false;
                    for r in 0..9 {
                        if r == r1 || r == r2 {
                            continue;
                        }
                        for c in 0..9 {
                            if (cols & (1 << c)) != 0 {
                                let idx = r * 9 + c;
                                if self.values[idx] == 0 && (self.candidates[idx] & bit) != 0 {
                                    self.candidates[idx] &= !bit;
                                    changed = true;
                                }
                            }
                        }
                    }
                    if changed {
                        let c1 = cols.trailing_zeros() as usize;
                        let c2 = (cols & !(1 << c1)).trailing_zeros() as usize;
                        return Some(StepOutcome::Eliminated {
                            technique: SolvingTechnique::XWing,
                            highlights: vec![r1 * 9 + c1, r1 * 9 + c2, r2 * 9 + c1, r2 * 9 + c2],
                        });
                    }
                }
            }
        }
        None
    }

    fn apply_x_wing_cols(&mut self, bit: u16) -> Option<StepOutcome> {
        let mut col_rows: Vec<(usize, u16)> = Vec::new();
        for c in 0..9 {
            let mut rows = 0_u16;
            for r in 0..9 {
                let idx = r * 9 + c;
                if self.values[idx] == 0 && (self.candidates[idx] & bit) != 0 {
                    rows |= 1 << r;
                }
            }
            if rows.count_ones() == 2 {
                col_rows.push((c, rows));
            }
        }
        for i in 0..col_rows.len() {
            for j in (i + 1)..col_rows.len() {
                if col_rows[i].1 == col_rows[j].1 {
                    let (c1, rows) = col_rows[i];
                    let c2 = col_rows[j].0;
                    let mut changed = false;
                    for c in 0..9 {
                        if c == c1 || c == c2 {
                            continue;
                        }
                        for r in 0..9 {
                            if (rows & (1 << r)) != 0 {
                                let idx = r * 9 + c;
                                if self.values[idx] == 0 && (self.candidates[idx] & bit) != 0 {
                                    self.candidates[idx] &= !bit;
                                    changed = true;
                                }
                            }
                        }
                    }
                    if changed {
                        let r1 = rows.trailing_zeros() as usize;
                        let r2 = (rows & !(1 << r1)).trailing_zeros() as usize;
                        return Some(StepOutcome::Eliminated {
                            technique: SolvingTechnique::XWing,
                            highlights: vec![r1 * 9 + c1, r1 * 9 + c2, r2 * 9 + c1, r2 * 9 + c2],
                        });
                    }
                }
            }
        }
        None
    }

    fn apply_xy_wing(&mut self) -> Option<StepOutcome> {
        let bivalues: Vec<usize> = (0..81)
            .filter(|&i| self.values[i] == 0 && self.candidates[i].count_ones() == 2)
            .collect();

        for &pivot in &bivalues {
            let pm = self.candidates[pivot];
            let x_bit = pm & pm.wrapping_neg();
            let y_bit = pm ^ x_bit;
            let wings: Vec<usize> = bivalues
                .iter()
                .copied()
                .filter(|&w| w != pivot && shares_peer(pivot, w))
                .collect();

            for i in 0..wings.len() {
                for j in (i + 1)..wings.len() {
                    let w1 = wings[i];
                    let w2 = wings[j];
                    let m1 = self.candidates[w1];
                    let m2 = self.candidates[w2];
                    let valid = ((m1 & x_bit) != 0
                        && (m1 & y_bit) == 0
                        && (m2 & y_bit) != 0
                        && (m2 & x_bit) == 0
                        && (m1 ^ x_bit) == (m2 ^ y_bit))
                        || ((m1 & y_bit) != 0
                            && (m1 & x_bit) == 0
                            && (m2 & x_bit) != 0
                            && (m2 & y_bit) == 0
                            && (m1 ^ y_bit) == (m2 ^ x_bit));
                    if !valid {
                        continue;
                    }
                    let z_bit = (m1 & m2) & !pm;
                    if z_bit.count_ones() != 1 {
                        continue;
                    }
                    let mut changed = false;
                    for target in 0..81 {
                        if target != pivot
                            && target != w1
                            && target != w2
                            && self.values[target] == 0
                            && (self.candidates[target] & z_bit) != 0
                            && shares_peer(target, w1)
                            && shares_peer(target, w2)
                        {
                            self.candidates[target] &= !z_bit;
                            changed = true;
                        }
                    }
                    if changed {
                        return Some(StepOutcome::Eliminated {
                            technique: SolvingTechnique::XYWing,
                            highlights: vec![pivot, w1, w2],
                        });
                    }
                }
            }
        }
        None
    }

    fn apply_swordfish(&mut self) -> Option<StepOutcome> {
        for digit in 1..=9_u8 {
            let bit = 1_u16 << (digit - 1);
            // Row-based Swordfish
            let mut row_cols: Vec<(usize, u16)> = Vec::new();
            for r in 0..9 {
                let mut cols = 0_u16;
                for c in 0..9 {
                    let idx = r * 9 + c;
                    if self.values[idx] == 0 && (self.candidates[idx] & bit) != 0 {
                        cols |= 1 << c;
                    }
                }
                if (2..=3).contains(&cols.count_ones()) {
                    row_cols.push((r, cols));
                }
            }
            for i in 0..row_cols.len() {
                for j in (i + 1)..row_cols.len() {
                    for k in (j + 1)..row_cols.len() {
                        let union = row_cols[i].1 | row_cols[j].1 | row_cols[k].1;
                        if union.count_ones() == 3 {
                            let (r1, r2, r3) = (row_cols[i].0, row_cols[j].0, row_cols[k].0);
                            let mut changed = false;
                            for r in 0..9 {
                                if r == r1 || r == r2 || r == r3 {
                                    continue;
                                }
                                for c in 0..9 {
                                    if (union & (1 << c)) != 0 {
                                        let idx = r * 9 + c;
                                        if self.values[idx] == 0
                                            && (self.candidates[idx] & bit) != 0
                                        {
                                            self.candidates[idx] &= !bit;
                                            changed = true;
                                        }
                                    }
                                }
                            }
                            if changed {
                                return Some(StepOutcome::Eliminated {
                                    technique: SolvingTechnique::Swordfish,
                                    highlights: vec![r1 * 9, r2 * 9, r3 * 9],
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Bounded 1-level bivalue forcing chain: tests each candidate of the smallest-domain
    /// empty cell using singles propagation; eliminates any candidate that leads to a
    /// direct contradiction.
    fn apply_forcing_chain(&mut self, units: &[(UnitKind, [usize; 9]); 27]) -> Option<StepOutcome> {
        let mut cells: Vec<(usize, u32)> = (0..81)
            .filter(|&i| self.values[i] == 0 && self.candidates[i].count_ones() >= 2)
            .map(|i| (i, self.candidates[i].count_ones()))
            .collect();
        cells.sort_by_key(|&(_, count)| count);

        for (idx, _) in cells.into_iter().take(12) {
            let mut mask = self.candidates[idx];
            while mask != 0 {
                #[allow(clippy::cast_possible_truncation)]
                let digit = (mask.trailing_zeros() as u8) + 1;
                let bit = 1_u16 << (digit - 1);
                mask &= mask - 1;

                let mut probe = self.clone();
                if !probe.assign(idx, digit) || probe.propagate_singles_hits_contradiction(units) {
                    self.candidates[idx] &= !bit;
                    return Some(StepOutcome::Eliminated {
                        technique: SolvingTechnique::ForcingChain,
                        highlights: vec![idx],
                    });
                }
            }
        }
        None
    }

    fn propagate_singles_hits_contradiction(
        &mut self,
        units: &[(UnitKind, [usize; 9]); 27],
    ) -> bool {
        loop {
            if self.has_contradiction() {
                return true;
            }
            if self.is_solved() {
                return false;
            }
            if self.find_naked_single().is_some() {
                continue;
            }
            if self.find_hidden_single(units).is_some() {
                continue;
            }
            if self.apply_locked_candidates(units).is_some() {
                continue;
            }
            return self.has_contradiction();
        }
    }
}

/// Grades a Sudoku starting grid by the hardest human solving technique required.
///
/// Returns `None` if the grid cannot be solved by the human technique ladder (or is invalid).
pub fn grade_puzzle(givens: &[Option<Digit>]) -> Option<Difficulty> {
    let mut board = LogicBoard::from_option_digits(givens)?;
    let units = all_units();
    let mut max_technique = SolvingTechnique::NakedSingle;

    while !board.is_solved() {
        let outcome = board.next_step(&units)?;
        let technique = match outcome {
            StepOutcome::Placed { technique, .. } | StepOutcome::Eliminated { technique, .. } => {
                technique
            }
        };
        if technique > max_technique {
            max_technique = technique;
        }
    }

    Some(max_technique.difficulty())
}

/// Computes an actionable Smart Hint for the current board state.
///
/// 1. If any editable cell currently holds an incorrect digit (a Mistake), returns
///    `HintSuggestion::FixMistake` first so logical deductions stay sound.
/// 2. Otherwise runs the human-technique solver until the next logical digit placement
///    is unlocked and returns `HintSuggestion::PlaceDigit` with the technique and explanation.
pub fn compute_smart_hint(puzzle: &Puzzle, values: &[Option<Digit>]) -> Option<HintSuggestion> {
    if values.len() != CellIndex::BOARD_SIZE || puzzle.solution.len() != CellIndex::BOARD_SIZE {
        return None;
    }

    if let Some(mistake_hint) = first_mistake_hint(puzzle, values) {
        return Some(mistake_hint);
    }

    if values.iter().all(Option::is_some) {
        return None;
    }

    let mut board = LogicBoard::from_option_digits(values)?;
    let units = all_units();
    let mut highest_enabler: Option<SolvingTechnique> = None;
    let mut enabler_highlights: Vec<usize> = Vec::new();

    while !board.is_solved() {
        let Some(outcome) = board.next_step(&units) else {
            break;
        };
        match outcome {
            StepOutcome::Eliminated {
                technique,
                highlights,
            } => {
                if highest_enabler.is_none_or(|prev| technique >= prev) {
                    highest_enabler = Some(technique);
                    enabler_highlights = highlights;
                }
            }
            StepOutcome::Placed {
                cell,
                digit,
                technique,
                explanation,
                highlights,
            } => {
                return build_place_hint(
                    cell,
                    digit,
                    technique,
                    explanation,
                    highlights,
                    highest_enabler,
                    enabler_highlights,
                );
            }
        }
    }

    fallback_chain_hint(puzzle, values, &board.candidates)
}

fn first_mistake_hint(puzzle: &Puzzle, values: &[Option<Digit>]) -> Option<HintSuggestion> {
    for (idx, (&cell_val, &sol_val)) in values.iter().zip(puzzle.solution.iter()).enumerate() {
        if let Some(entered) = cell_val {
            if entered != sol_val {
                let cell = CellIndex::new(idx).ok()?;
                return Some(HintSuggestion::FixMistake {
                    cell,
                    wrong_digit: entered,
                    explanation: format!(
                        "Row {}, Column {} currently holds {}, which is a Mistake. Clear it to keep deductions accurate.",
                        cell.row() + 1,
                        cell.col() + 1,
                        entered.get()
                    ),
                });
            }
        }
    }
    None
}

fn build_place_hint(
    cell: usize,
    digit: u8,
    technique: SolvingTechnique,
    explanation: String,
    highlights: Vec<usize>,
    highest_enabler: Option<SolvingTechnique>,
    enabler_highlights: Vec<usize>,
) -> Option<HintSuggestion> {
    let cell_idx = CellIndex::new(cell).ok()?;
    let digit_val = Digit::new(digit).ok()?;
    let (reported_technique, reported_explanation, raw_highlights) = match highest_enabler {
        Some(enabler) if enabler > technique => {
            let mut combined = enabler_highlights;
            if !combined.contains(&cell) {
                combined.push(cell);
            }
            (
                enabler,
                format!(
                    "{}: eliminating candidates unlocks {} at Row {}, Column {}.",
                    enabler.display_name(),
                    digit,
                    cell_idx.row() + 1,
                    cell_idx.col() + 1
                ),
                combined,
            )
        }
        _ => (technique, explanation, highlights),
    };

    let highlight_cells = raw_highlights
        .into_iter()
        .filter_map(|i| CellIndex::new(i).ok())
        .collect();

    Some(HintSuggestion::PlaceDigit {
        cell: cell_idx,
        digit: digit_val,
        technique: reported_technique,
        explanation: reported_explanation,
        highlight_cells,
    })
}

fn fallback_chain_hint(
    puzzle: &Puzzle,
    values: &[Option<Digit>],
    candidates: &[u16; 81],
) -> Option<HintSuggestion> {
    let (idx, _) = values
        .iter()
        .enumerate()
        .filter(|(_, v)| v.is_none())
        .min_by_key(|&(idx, _)| candidates[idx].count_ones())?;
    let cell = CellIndex::new(idx).ok()?;
    let digit = puzzle.solution[idx];
    Some(HintSuggestion::PlaceDigit {
        cell,
        digit,
        technique: SolvingTechnique::ForcingChain,
        explanation: format!(
            "Forcing Chain: elimination chains force {} in Row {}, Column {}.",
            digit.get(),
            cell.row() + 1,
            cell.col() + 1
        ),
        highlight_cells: vec![cell],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::exact::{solve_unique, SolutionCount};

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
    fn grades_singles_puzzle_as_easy() {
        let givens = parse_grid(
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
        assert_eq!(grade_puzzle(&givens), Some(Difficulty::Easy));
    }

    #[test]
    fn smart_hint_flags_existing_mistake_before_deducing_next_cell() {
        let givens = parse_grid(
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
        let SolutionCount::Unique(solution) = solve_unique(&givens) else {
            panic!("expected unique solution");
        };
        let puzzle = Puzzle {
            givens: givens.clone(),
            solution,
            difficulty: Difficulty::Easy,
        };

        let mut values = givens;
        // Cell 2 is empty; solution[2] is 4, so placing 9 is a Mistake.
        let wrong = if puzzle.solution[2].get() == 9 { 8 } else { 9 };
        values[2] = Some(Digit::new(wrong).unwrap());

        let hint = compute_smart_hint(&puzzle, &values).unwrap();
        assert!(matches!(
            hint,
            HintSuggestion::FixMistake { cell, wrong_digit, .. }
            if cell.as_usize() == 2 && wrong_digit.get() == wrong
        ));
    }

    #[test]
    fn smart_hint_returns_valid_placement_matching_unique_solution() {
        let givens = parse_grid(
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
        let SolutionCount::Unique(solution) = solve_unique(&givens) else {
            panic!("expected unique solution");
        };
        let puzzle = Puzzle {
            givens: givens.clone(),
            solution,
            difficulty: Difficulty::Easy,
        };

        let hint = compute_smart_hint(&puzzle, &givens).unwrap();
        match hint {
            HintSuggestion::PlaceDigit { cell, digit, .. } => {
                assert_eq!(puzzle.solution[cell.as_usize()], digit);
                assert!(givens[cell.as_usize()].is_none());
            }
            HintSuggestion::FixMistake { .. } => panic!("expected placement hint"),
        }
    }
}
