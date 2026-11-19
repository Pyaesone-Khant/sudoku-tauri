//! Thin Tauri v2 IPC command handlers delegating all rules to `sudoku-core`.

#![expect(
    clippy::needless_pass_by_value,
    reason = "tauri::command macro requires State<'_, T> parameters by value"
)]

use std::sync::Mutex;

use serde::Serialize;
use sudoku_core::{
    ActiveSlot, AppStateSnapshot, CellIndex, DailyCalendarMonth, DailyDate, Difficulty, Digit,
    HintSuggestion, StatsSummary, SudokuApp,
};
use tauri::State;

use crate::adapters::{AtomicJsonRepository, SystemClock};

/// Shared application state managed by Tauri.
#[derive(Debug)]
pub struct AppHost {
    /// Synchronized domain service handle.
    pub app: Mutex<SudokuApp<AtomicJsonRepository, SystemClock>>,
}

impl AppHost {
    /// Wraps an initialized `SudokuApp` in a mutex-guarded Tauri state container.
    pub const fn new(app: SudokuApp<AtomicJsonRepository, SystemClock>) -> Self {
        Self {
            app: Mutex::new(app),
        }
    }
}

/// Combined payload returned when a Smart Hint is requested.
#[derive(Debug, Serialize)]
pub struct HintResponse {
    /// Updated authoritative application state snapshot.
    pub state: AppStateSnapshot,
    /// Generated Smart Hint suggestion.
    pub hint: HintSuggestion,
}

fn with_app<T>(
    host: &State<'_, AppHost>,
    f: impl FnOnce(&mut SudokuApp<AtomicJsonRepository, SystemClock>) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = host
        .app
        .lock()
        .map_err(|_| "internal application lock error".to_owned())?;
    f(&mut guard)
}

#[tauri::command]
pub fn get_app_state(host: State<'_, AppHost>) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| Ok(app.app_snapshot(None)))
}

#[tauri::command]
pub fn start_standard_game(
    host: State<'_, AppHost>,
    difficulty: Difficulty,
) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| {
        app.start_standard_game(difficulty)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn open_daily_challenge(
    host: State<'_, AppHost>,
    date: &str,
) -> Result<AppStateSnapshot, String> {
    let parsed: DailyDate = date
        .parse()
        .map_err(|e: sudoku_core::DomainError| e.to_string())?;
    with_app(&host, |app| {
        app.open_daily_challenge(parsed).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn switch_active_slot(
    host: State<'_, AppHost>,
    slot: ActiveSlot,
) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| {
        app.switch_active_slot(slot).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn place_digit(
    host: State<'_, AppHost>,
    cell: usize,
    digit: u8,
) -> Result<AppStateSnapshot, String> {
    let cell_idx = CellIndex::new(cell).map_err(|e| e.to_string())?;
    let digit_val = Digit::new(digit).map_err(|e| e.to_string())?;
    with_app(&host, |app| {
        app.place_digit(cell_idx, digit_val)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn toggle_note(
    host: State<'_, AppHost>,
    cell: usize,
    digit: u8,
) -> Result<AppStateSnapshot, String> {
    let cell_idx = CellIndex::new(cell).map_err(|e| e.to_string())?;
    let digit_val = Digit::new(digit).map_err(|e| e.to_string())?;
    with_app(&host, |app| {
        app.toggle_note(cell_idx, digit_val)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn erase_cell(host: State<'_, AppHost>, cell: usize) -> Result<AppStateSnapshot, String> {
    let cell_idx = CellIndex::new(cell).map_err(|e| e.to_string())?;
    with_app(&host, |app| {
        app.erase_cell(cell_idx).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn undo(host: State<'_, AppHost>) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| app.undo().map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn redo(host: State<'_, AppHost>) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| app.redo().map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn request_hint(host: State<'_, AppHost>) -> Result<HintResponse, String> {
    with_app(&host, |app| {
        let (state, hint) = app.request_hint().map_err(|e| e.to_string())?;
        Ok(HintResponse { state, hint })
    })
}

#[tauri::command]
pub fn apply_hint(host: State<'_, AppHost>) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| app.apply_hint().map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn dismiss_hint(host: State<'_, AppHost>) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| app.dismiss_hint().map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn pause_game(host: State<'_, AppHost>) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| app.pause_game().map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn resume_game(host: State<'_, AppHost>) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| app.resume_game().map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn sync_solve_time(
    host: State<'_, AppHost>,
    delta_secs: u64,
) -> Result<AppStateSnapshot, String> {
    with_app(&host, |app| {
        app.sync_solve_time(delta_secs).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_daily_calendar(
    host: State<'_, AppHost>,
    year: i32,
    month: u8,
) -> Result<DailyCalendarMonth, String> {
    with_app(&host, |app| {
        app.daily_calendar(year, month).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_stats(host: State<'_, AppHost>) -> Result<StatsSummary, String> {
    with_app(&host, |app| Ok(app.stats_summary()))
}
