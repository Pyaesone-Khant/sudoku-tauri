//! Tauri v2 desktop host for the offline macOS Sudoku application.

#![deny(clippy::unwrap_used)]

#[tauri::command]
fn get_core_version() -> &'static str {
    sudoku_core::core_version()
}

/// Starts the Tauri v2 desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_core_version])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("fatal error while running Sudoku: {err}");
        std::process::exit(1);
    }
}
