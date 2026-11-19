//! Exact backtracking uniqueness solver, human-technique grader, and Smart Hint engine.

pub mod exact;
pub mod human;

pub use exact::{generate_complete_grid, solve_unique, SolutionCount};
pub use human::{compute_smart_hint, grade_puzzle, HintSuggestion, SolvingTechnique};
