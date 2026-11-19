//! Per-Difficulty Statistics, Win Streak, Daily Streak, Recent History, and Daily Calendar.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::model::{DailyDate, Difficulty, GameKind};
use crate::scoring::ScoreBreakdown;

/// Immutable record persisted whenever a Game reaches `Completed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletedGameRecord {
    /// Completed Game ID.
    pub id: u64,
    /// Standard Game or Daily Challenge (`YYYY-MM-DD`).
    pub kind: GameKind,
    /// Puzzle Difficulty.
    pub difficulty: Difficulty,
    /// Total active Solve Time in seconds.
    pub solve_time_secs: u64,
    /// Total permanent Mistakes recorded.
    pub mistakes: u32,
    /// Total Hints used.
    pub hints_used: u32,
    /// Complete Score breakdown including Time Bonus and Final Score.
    pub score: ScoreBreakdown,
    /// True if completed with zero Mistakes and zero Hints.
    pub flawless: bool,
    /// Local calendar date on which the Game was completed.
    pub completed_on: DailyDate,
}

/// Aggregated statistics for a single Difficulty tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifficultyStats {
    /// Difficulty tier.
    pub difficulty: Difficulty,
    /// Total completed Games (Standard + Daily) in this Difficulty.
    pub games_completed: u32,
    /// Total abandoned Standard Games in this Difficulty.
    pub games_abandoned: u32,
    /// Highest Final Score achieved in this Difficulty.
    pub best_score: Option<u32>,
    /// Mean Final Score rounded to nearest integer.
    pub avg_score: Option<u32>,
    /// Fastest Solve Time in seconds.
    pub best_time_secs: Option<u64>,
    /// Mean Solve Time in seconds rounded to nearest integer.
    pub avg_time_secs: Option<u64>,
    /// Total Flawless Solves (0 Mistakes, 0 Hints).
    pub flawless_solves: u32,
}

/// Full Statistics dashboard projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatsSummary {
    /// Per-Difficulty statistics in canonical order (`Easy`, `Medium`, `Hard`, `Expert`).
    pub by_difficulty: Vec<DifficultyStats>,
    /// Current consecutive Standard Games completed without abandoning.
    pub win_streak_current: u32,
    /// All-time best Standard Game Win Streak.
    pub win_streak_best: u32,
    /// Current consecutive calendar days with a completed Daily Challenge leading to today/yesterday.
    pub daily_streak_current: u32,
    /// All-time best consecutive calendar-day Daily Streak.
    pub daily_streak_best: u32,
    /// Most recent completed Games (newest first, capped at 50).
    pub recent_history: Vec<CompletedGameRecord>,
}

/// Playability/completion status for a single date in the Daily Challenge Calendar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum DailyDayState {
    /// Date is in the future relative to the local clock.
    FutureLocked,
    /// Past or today date not yet started.
    Playable,
    /// Currently open in the active Daily Challenge slot.
    InProgress,
    /// Completed and locked as an immutable record.
    Completed {
        /// Recorded completion summary for this date.
        record: CompletedGameRecord,
    },
}

/// Single day cell inside a monthly Daily Challenge Calendar projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyCalendarDay {
    /// Local calendar date (`YYYY-MM-DD`).
    pub date: DailyDate,
    /// Deterministic weekly Difficulty for this date.
    pub difficulty: Difficulty,
    /// Current day state.
    pub state: DailyDayState,
}

/// Monthly Daily Challenge Calendar projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyCalendarMonth {
    /// Four-digit year.
    pub year: i32,
    /// Month (`1..=12`).
    pub month: u8,
    /// Current local date from the Clock.
    pub today: DailyDate,
    /// Current Daily Streak leading to today or yesterday.
    pub daily_streak_current: u32,
    /// Best all-time Daily Streak.
    pub daily_streak_best: u32,
    /// Ordered days `1..=days_in_month`.
    pub days: Vec<DailyCalendarDay>,
}

/// Computes `(daily_streak_current, daily_streak_best)` from completed Daily Challenge records.
pub fn compute_daily_streaks(records: &[CompletedGameRecord], today: DailyDate) -> (u32, u32) {
    let completed_dates: BTreeSet<DailyDate> = records
        .iter()
        .filter_map(|r| match r.kind {
            GameKind::Daily(date) => Some(date),
            GameKind::Standard => None,
        })
        .collect();

    if completed_dates.is_empty() {
        return (0, 0);
    }

    // Best streak across all recorded Daily Challenge dates.
    let mut best: u32 = 0;
    let mut run: u32 = 0;
    let mut prev: Option<DailyDate> = None;
    for &date in &completed_dates {
        if prev.is_some_and(|p| date.days_since(p) == 1) {
            run += 1;
        } else {
            run = 1;
        }
        if run > best {
            best = run;
        }
        prev = Some(date);
    }

    // Current streak leading up to today or yesterday.
    let anchor = if completed_dates.contains(&today) {
        Some(today)
    } else {
        let yesterday = today.prev_day();
        completed_dates.contains(&yesterday).then_some(yesterday)
    };

    let mut current: u32 = 0;
    if let Some(mut cursor) = anchor {
        while completed_dates.contains(&cursor) {
            current += 1;
            cursor = cursor.prev_day();
        }
    }

    (current, best)
}

/// Builds the full `StatsSummary` across all completed games and streaks.
pub fn build_stats_summary(
    completed_games: &[CompletedGameRecord],
    abandoned_by_difficulty: [u32; 4],
    win_streak_current: u32,
    win_streak_best: u32,
    today: DailyDate,
) -> StatsSummary {
    let by_difficulty = Difficulty::ALL
        .into_iter()
        .enumerate()
        .map(|(idx, difficulty)| {
            let matching: Vec<&CompletedGameRecord> = completed_games
                .iter()
                .filter(|r| r.difficulty == difficulty)
                .collect();
            #[allow(clippy::cast_possible_truncation)]
            let games_completed = matching.len() as u32;
            let best_score = matching.iter().map(|r| r.score.final_score).max();
            let avg_score = if games_completed == 0 {
                None
            } else {
                let sum: u64 = matching
                    .iter()
                    .map(|r| u64::from(r.score.final_score))
                    .sum();
                let count = u64::from(games_completed);
                #[allow(clippy::cast_possible_truncation)]
                Some(((sum + count / 2) / count) as u32)
            };
            let best_time_secs = matching.iter().map(|r| r.solve_time_secs).min();
            let avg_time_secs = if games_completed == 0 {
                None
            } else {
                let sum: u64 = matching.iter().map(|r| r.solve_time_secs).sum();
                let count = u64::from(games_completed);
                Some((sum + count / 2) / count)
            };
            #[allow(clippy::cast_possible_truncation)]
            let flawless_solves = matching.iter().filter(|r| r.flawless).count() as u32;

            DifficultyStats {
                difficulty,
                games_completed,
                games_abandoned: abandoned_by_difficulty[idx],
                best_score,
                avg_score,
                best_time_secs,
                avg_time_secs,
                flawless_solves,
            }
        })
        .collect();

    let (daily_streak_current, daily_streak_best) = compute_daily_streaks(completed_games, today);
    let recent_history = completed_games.iter().rev().take(50).cloned().collect();

    StatsSummary {
        by_difficulty,
        win_streak_current,
        win_streak_best,
        daily_streak_current,
        daily_streak_best,
        recent_history,
    }
}

/// Builds the monthly Daily Challenge calendar projection for `(year, month)`.
pub fn build_daily_calendar_month(
    year: i32,
    month: u8,
    today: DailyDate,
    active_daily_date: Option<DailyDate>,
    completed_games: &[CompletedGameRecord],
) -> DailyCalendarMonth {
    let completed_by_date: BTreeMap<DailyDate, &CompletedGameRecord> = completed_games
        .iter()
        .filter_map(|r| match r.kind {
            GameKind::Daily(d) => Some((d, r)),
            GameKind::Standard => None,
        })
        .collect();

    let total_days = DailyDate::days_in_month(year, month);
    let mut days = Vec::with_capacity(usize::from(total_days));

    for day in 1..=total_days {
        if let Ok(date) = DailyDate::new(year, month, day) {
            let state = if let Some(&record) = completed_by_date.get(&date) {
                DailyDayState::Completed {
                    record: record.clone(),
                }
            } else if date > today {
                DailyDayState::FutureLocked
            } else if active_daily_date == Some(date) {
                DailyDayState::InProgress
            } else {
                DailyDayState::Playable
            };
            days.push(DailyCalendarDay {
                date,
                difficulty: date.daily_difficulty(),
                state,
            });
        }
    }

    let (daily_streak_current, daily_streak_best) = compute_daily_streaks(completed_games, today);

    DailyCalendarMonth {
        year,
        month,
        today,
        daily_streak_current,
        daily_streak_best,
        days,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn daily_record(id: u64, date: &str, final_score: u32) -> CompletedGameRecord {
        let d: DailyDate = date.parse().unwrap();
        CompletedGameRecord {
            id,
            kind: GameKind::Daily(d),
            difficulty: d.daily_difficulty(),
            solve_time_secs: 200,
            mistakes: 0,
            hints_used: 0,
            score: ScoreBreakdown {
                earned_score: final_score,
                time_bonus: 0,
                mistake_penalty: 0,
                hint_penalty: 0,
                total_penalty: 0,
                final_score,
            },
            flawless: true,
            completed_on: d,
        }
    }

    #[test]
    fn daily_streak_handles_today_yesterday_and_gaps() {
        let today: DailyDate = "2026-11-19".parse().unwrap();
        let records = vec![
            daily_record(1, "2026-11-10", 2000),
            daily_record(2, "2026-11-11", 2000),
            daily_record(3, "2026-11-12", 2000),
            daily_record(4, "2026-11-13", 2000),
            // Gap on Nov 14-16
            daily_record(5, "2026-11-17", 2000),
            daily_record(6, "2026-11-18", 2000),
        ];

        // Anchored at yesterday (Nov 18): current = 2, best = 4.
        assert_eq!(compute_daily_streaks(&records, today), (2, 4));

        // Completing today extends current to 3.
        let mut with_today = records;
        with_today.push(daily_record(7, "2026-11-19", 2100));
        assert_eq!(compute_daily_streaks(&with_today, today), (3, 4));
    }

    #[test]
    fn daily_streak_breaks_when_missed_yesterday_and_today() {
        let today: DailyDate = "2026-11-19".parse().unwrap();
        let records = vec![
            daily_record(1, "2026-11-16", 1500),
            daily_record(2, "2026-11-17", 1500),
        ];
        assert_eq!(compute_daily_streaks(&records, today), (0, 2));
    }
}
