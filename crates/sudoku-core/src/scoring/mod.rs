//! Pure scoring math: Earned Score, Time Bonus, Penalty, and Final Score.

use serde::{Deserialize, Serialize};

use crate::model::Difficulty;

/// Complete score breakdown for a Game (live or completed).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Points accumulated from first-time correct editable cell placements.
    pub earned_score: u32,
    /// Bonus awarded upon completion faster than the Difficulty target time.
    pub time_bonus: u32,
    /// Total points deducted for Mistakes.
    pub mistake_penalty: u32,
    /// Total points deducted for applied Hints.
    pub hint_penalty: u32,
    /// Sum of Mistake and Hint penalties.
    pub total_penalty: u32,
    /// `Earned Score + Time Bonus − Penalty`, floored at zero.
    pub final_score: u32,
}

/// Points awarded for each first-time correct editable cell placement.
pub const fn points_per_cell(difficulty: Difficulty) -> u32 {
    match difficulty {
        Difficulty::Easy => 50,
        Difficulty::Medium => 75,
        Difficulty::Hard => 100,
        Difficulty::Expert => 150,
    }
}

/// Penalty points deducted per recorded Mistake.
pub const fn mistake_penalty_points(difficulty: Difficulty) -> u32 {
    match difficulty {
        Difficulty::Easy => 100,
        Difficulty::Medium => 150,
        Difficulty::Hard => 200,
        Difficulty::Expert => 300,
    }
}

/// Penalty points deducted per applied Hint.
pub const fn hint_penalty_points(difficulty: Difficulty) -> u32 {
    match difficulty {
        Difficulty::Easy => 150,
        Difficulty::Medium => 225,
        Difficulty::Hard => 300,
        Difficulty::Expert => 450,
    }
}

/// Target Solve Time in seconds below which a Time Bonus is awarded on completion.
pub const fn target_solve_time_secs(difficulty: Difficulty) -> u64 {
    match difficulty {
        Difficulty::Easy => 300,
        Difficulty::Medium => 600,
        Difficulty::Hard => 1_080,
        Difficulty::Expert => 1_800,
    }
}

/// Maximum possible Time Bonus awarded for an instant completion.
pub const fn max_time_bonus(difficulty: Difficulty) -> u32 {
    match difficulty {
        Difficulty::Easy => 1_000,
        Difficulty::Medium => 1_500,
        Difficulty::Hard => 2_000,
        Difficulty::Expert => 3_000,
    }
}

/// Computes Time Bonus for a completed Game using deterministic integer rounding.
#[allow(clippy::cast_possible_truncation)]
pub fn compute_time_bonus(difficulty: Difficulty, solve_time_secs: u64) -> u32 {
    let target = target_solve_time_secs(difficulty);
    if solve_time_secs >= target {
        return 0;
    }
    let remaining = target - solve_time_secs;
    let max_bonus = u64::from(max_time_bonus(difficulty));
    ((max_bonus * remaining + (target / 2)) / target) as u32
}

/// Computes the complete `ScoreBreakdown` for a Game.
///
/// `time_bonus` is `0` while the Game is not yet `Completed`.
pub fn compute_score_breakdown(
    difficulty: Difficulty,
    awarded_cell_count: u32,
    mistakes: u32,
    hints_used: u32,
    solve_time_secs: u64,
    completed: bool,
) -> ScoreBreakdown {
    let earned_score = awarded_cell_count.saturating_mul(points_per_cell(difficulty));
    let time_bonus = if completed {
        compute_time_bonus(difficulty, solve_time_secs)
    } else {
        0
    };
    let mistake_penalty = mistakes.saturating_mul(mistake_penalty_points(difficulty));
    let hint_penalty = hints_used.saturating_mul(hint_penalty_points(difficulty));
    let total_penalty = mistake_penalty.saturating_add(hint_penalty);
    let final_score = earned_score
        .saturating_add(time_bonus)
        .saturating_sub(total_penalty);

    ScoreBreakdown {
        earned_score,
        time_bonus,
        mistake_penalty,
        hint_penalty,
        total_penalty,
        final_score,
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Difficulty::Easy, 0, 1_000)]
    #[case(Difficulty::Easy, 150, 500)]
    #[case(Difficulty::Easy, 300, 0)]
    #[case(Difficulty::Easy, 600, 0)]
    #[case(Difficulty::Medium, 300, 750)]
    #[case(Difficulty::Hard, 540, 1_000)]
    #[case(Difficulty::Expert, 900, 1_500)]
    fn time_bonus_scales_linearly_to_zero(
        #[case] difficulty: Difficulty,
        #[case] solve_time_secs: u64,
        #[case] expected: u32,
    ) {
        assert_eq!(compute_time_bonus(difficulty, solve_time_secs), expected);
    }

    #[test]
    fn final_score_floors_at_zero_when_penalties_exceed_earned() {
        let breakdown = compute_score_breakdown(Difficulty::Easy, 2, 10, 2, 120, true);
        assert_eq!(breakdown.earned_score, 100);
        assert_eq!(breakdown.mistake_penalty, 1_000);
        assert_eq!(breakdown.hint_penalty, 300);
        assert_eq!(breakdown.total_penalty, 1_300);
        assert_eq!(breakdown.final_score, 0);
    }

    #[test]
    fn time_bonus_is_zero_while_in_progress() {
        let breakdown = compute_score_breakdown(Difficulty::Hard, 10, 1, 0, 30, false);
        assert_eq!(breakdown.earned_score, 1_000);
        assert_eq!(breakdown.time_bonus, 0);
        assert_eq!(breakdown.total_penalty, 200);
        assert_eq!(breakdown.final_score, 800);
    }

    proptest! {
        #[test]
        fn final_score_never_exceeds_earned_plus_time_bonus(
            cells in 0_u32..81,
            mistakes in 0_u32..100,
            hints in 0_u32..81,
            secs in 0_u64..5000,
            completed in any::<bool>(),
        ) {
            for difficulty in Difficulty::ALL {
                let s = compute_score_breakdown(difficulty, cells, mistakes, hints, secs, completed);
                assert_eq!(s.total_penalty, s.mistake_penalty + s.hint_penalty);
                assert_eq!(s.final_score, (s.earned_score + s.time_bonus).saturating_sub(s.total_penalty));
            }
        }
    }
}
