//! Integration tests covering end-to-end `sudoku-core` domain invariants.

use sudoku_core::{
    ActiveSlot, CellIndex, DailyDate, DailyDayState, Difficulty, Digit, DomainError, FixedClock,
    GameKind, GameStatus, InMemoryRepository, MoveOutcome, SudokuApp,
};

fn make_app(today: &str) -> SudokuApp<InMemoryRepository, FixedClock> {
    let date: DailyDate = today.parse().expect("valid test date");
    SudokuApp::load(InMemoryRepository::new(), FixedClock::new(date)).expect("app loads")
}

fn solve_focused_game(app: &mut SudokuApp<InMemoryRepository, FixedClock>) {
    let solution = app
        .focused_game()
        .expect("focused game")
        .puzzle()
        .solution
        .clone();
    let givens = app
        .focused_game()
        .expect("focused game")
        .puzzle()
        .givens
        .clone();

    for (idx, (&given, &sol)) in givens.iter().zip(solution.iter()).enumerate() {
        if given.is_none() {
            let cell = CellIndex::new(idx).expect("valid cell");
            let snap = app.place_digit(cell, sol).expect("valid move");
            if snap.last_move_outcome == Some(MoveOutcome::Completed) {
                return;
            }
        }
    }
}

#[test]
fn dual_active_slots_preserve_standard_game_when_opening_daily_challenge() {
    let mut app = make_app("2026-11-19");
    let std_snap = app
        .start_standard_game(Difficulty::Easy)
        .expect("standard game starts");
    let std_id = std_snap.current_game.expect("game").id;

    // Make one move on Standard Game and accumulate Solve Time.
    app.sync_solve_time(42).expect("sync time");

    // Open today's Daily Challenge: Standard Game must remain preserved and paused.
    let today: DailyDate = "2026-11-19".parse().expect("date");
    let daily_snap = app.open_daily_challenge(today).expect("daily opens");
    assert_eq!(daily_snap.active_slot, ActiveSlot::Daily);
    assert!(daily_snap.has_in_progress_standard);
    assert_eq!(daily_snap.in_progress_daily_date, Some(today));
    assert_eq!(
        daily_snap.current_game.expect("daily game").kind,
        GameKind::Daily(today)
    );

    // Switch back to Standard slot: same game ID, paused cleanly with preserved Solve Time.
    let back = app
        .switch_active_slot(ActiveSlot::Standard)
        .expect("switch back");
    let restored = back.current_game.expect("restored standard");
    assert_eq!(restored.id, std_id);
    assert_eq!(restored.status, GameStatus::Paused);
    assert_eq!(restored.solve_time_secs, 42);
}

#[test]
fn daily_challenge_blocks_future_dates_and_locks_completed_dates() {
    let mut app = make_app("2026-11-19");
    let tomorrow: DailyDate = "2026-11-20".parse().expect("date");
    let yesterday: DailyDate = "2026-11-18".parse().expect("date");
    let today: DailyDate = "2026-11-19".parse().expect("date");

    // Future date must be locked.
    assert!(matches!(
        app.open_daily_challenge(tomorrow),
        Err(DomainError::FutureDailyLocked(_))
    ));

    // Past catch-up date is playable and completes cleanly.
    app.open_daily_challenge(yesterday)
        .expect("past date playable");
    app.sync_solve_time(120).expect("time");
    solve_focused_game(&mut app);

    // Reopening completed Daily date must be rejected.
    assert!(matches!(
        app.open_daily_challenge(yesterday),
        Err(DomainError::DailyAlreadyCompleted(_))
    ));

    // Complete today as well -> Daily Streak becomes 2, and both roll into per-Difficulty Stats.
    app.open_daily_challenge(today).expect("today playable");
    app.sync_solve_time(180).expect("time");
    solve_focused_game(&mut app);

    let stats = app.stats_summary();
    assert_eq!(stats.daily_streak_current, 2);
    assert_eq!(stats.daily_streak_best, 2);
    assert_eq!(stats.recent_history.len(), 2);

    // Verify Calendar projection reflects Completed, Playable, and FutureLocked states.
    let cal = app.daily_calendar(2026, 11).expect("calendar");
    assert_eq!(cal.daily_streak_current, 2);
    assert!(matches!(
        cal.days[17].state,
        DailyDayState::Completed { .. }
    )); // Nov 18
    assert!(matches!(
        cal.days[18].state,
        DailyDayState::Completed { .. }
    )); // Nov 19
    assert!(matches!(cal.days[19].state, DailyDayState::FutureLocked)); // Nov 20
    assert!(matches!(cal.days[0].state, DailyDayState::Playable)); // Nov 1
}

#[test]
fn win_streak_increments_on_standard_completion_and_resets_on_abandonment() {
    let mut app = make_app("2026-11-19");

    // Complete first Standard Game.
    app.start_standard_game(Difficulty::Easy).expect("start 1");
    app.sync_solve_time(60).expect("time");
    solve_focused_game(&mut app);
    let stats = app.stats_summary();
    assert_eq!(stats.win_streak_current, 1);
    assert_eq!(stats.win_streak_best, 1);
    assert_eq!(stats.by_difficulty[0].flawless_solves, 1);
    assert!(stats.by_difficulty[0].best_score.unwrap_or(0) > 0);

    // Start a second Standard Game and abandon it by starting a third.
    app.start_standard_game(Difficulty::Easy).expect("start 2");
    app.start_standard_game(Difficulty::Medium)
        .expect("start 3 abandons 2");

    let stats = app.stats_summary();
    assert_eq!(stats.win_streak_current, 0);
    assert_eq!(stats.win_streak_best, 1);
    assert_eq!(stats.by_difficulty[0].games_abandoned, 1);
}

#[test]
fn persisted_state_auto_pauses_on_reload() {
    let repo = InMemoryRepository::new();
    let today: DailyDate = "2026-11-19".parse().expect("date");
    let mut app = SudokuApp::load(repo, FixedClock::new(today)).expect("load");
    app.start_standard_game(Difficulty::Easy).expect("start");

    let cell = CellIndex::new(
        app.focused_game()
            .expect("game")
            .puzzle()
            .givens
            .iter()
            .position(Option::is_none)
            .expect("editable"),
    )
    .expect("cell");
    let sol = app
        .focused_game()
        .expect("game")
        .puzzle()
        .solution_digit(cell)
        .expect("sol");
    let wrong = Digit::new(if sol.get() == 9 { 1 } else { sol.get() + 1 }).expect("digit");
    app.place_digit(cell, wrong).expect("mistake");
    app.sync_solve_time(25).expect("time");

    let snap = app.app_snapshot(None);
    assert_eq!(snap.current_game.as_ref().expect("g").mistakes, 1);
    assert_eq!(snap.current_game.as_ref().expect("g").solve_time_secs, 25);
}
