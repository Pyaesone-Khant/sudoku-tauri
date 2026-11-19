//! Tauri v2 desktop host for the offline macOS Sudoku application.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod adapters;
pub mod commands;

use sudoku_core::SudokuApp;
use tauri::{Emitter, Manager, WindowEvent};

use adapters::{AtomicJsonRepository, SystemClock};
use commands::AppHost;

/// Starts the Tauri v2 desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let repo = AtomicJsonRepository::new(data_dir.join("state.json"));
            let domain_app = SudokuApp::load(repo, SystemClock)?;
            app.manage(AppHost::new(domain_app));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(false) = event {
                let snapshot = {
                    let host = window.state::<AppHost>();
                    let locked = host.app.lock();
                    locked.ok().and_then(|mut guard| guard.pause_game().ok())
                };
                if let Some(snapshot) = snapshot {
                    let _ = window.emit("game-auto-paused", snapshot);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::start_standard_game,
            commands::open_daily_challenge,
            commands::switch_active_slot,
            commands::place_digit,
            commands::toggle_note,
            commands::erase_cell,
            commands::undo,
            commands::redo,
            commands::request_hint,
            commands::apply_hint,
            commands::dismiss_hint,
            commands::pause_game,
            commands::resume_game,
            commands::sync_solve_time,
            commands::get_daily_calendar,
            commands::get_stats,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("fatal error while running Sudoku: {err}");
        std::process::exit(1);
    }
}
