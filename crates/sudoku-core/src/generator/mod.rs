//! Unique-solution, technique-graded Sudoku Puzzle and deterministic Daily Challenge generator.

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::model::{CellIndex, DailyDate, Difficulty, Digit, Puzzle};
use crate::solver::exact::{generate_complete_grid, solve_unique, SolutionCount};
use crate::solver::human::grade_puzzle;

/// Version-stable 64-bit FNV-1a + `SplitMix64` seed derivation for offline Daily Challenges.
fn daily_seed(date: DailyDate) -> u64 {
    let key = format!("sudoku-daily-v1:{}", date.to_iso_string());
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    // Final SplitMix64 avalanche step for crisp bit dispersion.
    hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    hash = (hash ^ (hash >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    hash ^ (hash >> 31)
}

/// Generates a Standard Game Puzzle guaranteed to have a unique solution and exact `difficulty`.
pub fn generate_standard_puzzle<R: Rng>(difficulty: Difficulty, rng: &mut R) -> Puzzle {
    // Keep a bounded search budget so generation stays snappy even in debug builds.
    let max_attempts = match difficulty {
        Difficulty::Easy => 60,
        Difficulty::Medium => 120,
        Difficulty::Hard => 280,
        Difficulty::Expert => 360,
    };

    let mut closest: Option<(Difficulty, Puzzle)> = None;

    for _ in 0..max_attempts {
        let solution = generate_complete_grid(rng);
        let givens = carve_clues(&solution, difficulty, rng);

        if let Some(graded) = grade_puzzle(&givens) {
            if graded == difficulty {
                return Puzzle {
                    givens,
                    solution,
                    difficulty,
                };
            }
            // Track closest non-exact match as a safe deterministic fallback (never fails).
            let replace = match &closest {
                None => true,
                Some((prev_diff, _)) => {
                    difficulty_distance(graded, difficulty)
                        < difficulty_distance(*prev_diff, difficulty)
                }
            };
            if replace {
                closest = Some((
                    graded,
                    Puzzle {
                        givens,
                        solution,
                        difficulty,
                    },
                ));
            }
        }
    }

    if let Some((_, puzzle)) = closest {
        return puzzle;
    }

    // Ultra-rare fallback: return a uniquely solvable carved board.
    let solution = generate_complete_grid(rng);
    let givens = carve_clues(&solution, Difficulty::Easy, rng);
    Puzzle {
        givens,
        solution,
        difficulty,
    }
}

/// Generates the deterministic offline Daily Challenge Puzzle for `date`.
pub fn generate_daily_puzzle(date: DailyDate) -> Puzzle {
    let difficulty = date.daily_difficulty();
    let mut rng = ChaCha8Rng::seed_from_u64(daily_seed(date));
    generate_standard_puzzle(difficulty, &mut rng)
}

const fn difficulty_rank(d: Difficulty) -> u8 {
    match d {
        Difficulty::Easy => 0,
        Difficulty::Medium => 1,
        Difficulty::Hard => 2,
        Difficulty::Expert => 3,
    }
}

const fn difficulty_distance(a: Difficulty, b: Difficulty) -> u8 {
    difficulty_rank(a).abs_diff(difficulty_rank(b))
}

/// Carves clues from a solved board while preserving `SolutionCount::Unique`.
fn carve_clues<R: Rng>(solution: &[Digit], target: Difficulty, rng: &mut R) -> Vec<Option<Digit>> {
    let mut givens: Vec<Option<Digit>> = solution.iter().copied().map(Some).collect();
    let mut order: Vec<usize> = (0..CellIndex::BOARD_SIZE).collect();
    order.shuffle(rng);

    // Target clue floors calibrated per Difficulty to steer the solver toward the
    // desired technique tier quickly without over-carving Easy puzzles.
    let min_givens = match target {
        Difficulty::Easy => rng.gen_range(36..=43),
        Difficulty::Medium => rng.gen_range(28..=34),
        Difficulty::Hard => rng.gen_range(23..=29),
        Difficulty::Expert => 21,
    };

    let mut remaining = CellIndex::BOARD_SIZE;
    for idx in order {
        if remaining <= min_givens {
            break;
        }
        let backup = givens[idx];
        givens[idx] = None;
        if matches!(solve_unique(&givens), SolutionCount::Unique(_)) {
            remaining -= 1;
        } else {
            givens[idx] = backup;
        }
    }

    givens
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Difficulty::Easy, 101)]
    #[case(Difficulty::Medium, 202)]
    #[case(Difficulty::Hard, 303)]
    #[case(Difficulty::Expert, 404)]
    fn standard_puzzle_has_unique_solution_and_valid_givens(
        #[case] difficulty: Difficulty,
        #[case] seed: u64,
    ) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let puzzle = generate_standard_puzzle(difficulty, &mut rng);

        assert_eq!(puzzle.difficulty, difficulty);
        assert_eq!(puzzle.givens.len(), 81);
        assert_eq!(puzzle.solution.len(), 81);
        assert!(puzzle.given_count() >= 17 && puzzle.given_count() <= 50);

        for (given, &sol) in puzzle.givens.iter().zip(puzzle.solution.iter()) {
            if let Some(g) = given {
                assert_eq!(*g, sol);
            }
        }
        assert_eq!(
            solve_unique(&puzzle.givens),
            SolutionCount::Unique(puzzle.solution.clone())
        );
    }

    #[test]
    fn daily_challenge_is_100_percent_deterministic() {
        let date = DailyDate::new(2026, 11, 19).unwrap();
        let first = generate_daily_puzzle(date);
        let second = generate_daily_puzzle(date);
        assert_eq!(first, second);
        assert_eq!(first.difficulty, Difficulty::Hard); // 2026-11-19 is Thursday
        assert_eq!(
            solve_unique(&first.givens),
            SolutionCount::Unique(first.solution)
        );
    }

    #[test]
    fn distinct_dates_generate_distinct_daily_puzzles() {
        let d1 = DailyDate::new(2026, 11, 18).unwrap();
        let d2 = DailyDate::new(2026, 11, 19).unwrap();
        assert_ne!(generate_daily_puzzle(d1), generate_daily_puzzle(d2));
    }
}
