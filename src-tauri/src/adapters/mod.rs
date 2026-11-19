//! Infrastructure adapters implementing `sudoku-core` boundary traits.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use chrono::{Datelike, Local};
use sudoku_core::{Clock, DailyDate, DomainError, GameRepository, PersistedState};

/// Crash-safe JSON persistence adapter using write-sync-rename atomic swaps.
#[derive(Debug, Clone)]
pub struct AtomicJsonRepository {
    file_path: PathBuf,
}

impl AtomicJsonRepository {
    /// Creates a repository persisting state at `file_path`.
    pub const fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl GameRepository for AtomicJsonRepository {
    fn load(&self) -> Result<PersistedState, DomainError> {
        if !self.file_path.exists() {
            return Ok(PersistedState::default());
        }
        let raw = fs::read_to_string(&self.file_path)
            .map_err(|e| DomainError::Storage(format!("failed to read state: {e}")))?;
        serde_json::from_str(&raw)
            .map_err(|e| DomainError::Storage(format!("failed to parse saved state: {e}")))
    }

    fn save(&self, state: &PersistedState) -> Result<(), DomainError> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| DomainError::Storage(format!("failed to create data dir: {e}")))?;
        }
        let payload = serde_json::to_vec_pretty(state)
            .map_err(|e| DomainError::Storage(format!("failed to serialize state: {e}")))?;

        let tmp_path = self.file_path.with_extension("json.tmp");
        let mut file = File::create(&tmp_path)
            .map_err(|e| DomainError::Storage(format!("failed to create temp state file: {e}")))?;
        file.write_all(&payload)
            .map_err(|e| DomainError::Storage(format!("failed to write temp state file: {e}")))?;
        file.sync_all()
            .map_err(|e| DomainError::Storage(format!("failed to sync temp state file: {e}")))?;

        fs::rename(&tmp_path, &self.file_path)
            .map_err(|e| DomainError::Storage(format!("failed to replace state file: {e}")))?;
        Ok(())
    }
}

/// System clock returning the user's current local calendar date.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn today(&self) -> DailyDate {
        let now = Local::now().date_naive();
        #[allow(clippy::cast_possible_truncation)]
        DailyDate::new(now.year(), now.month() as u8, now.day() as u8)
            .unwrap_or_else(|_| DailyDate::from_days_since_epoch(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_json_repository_roundtrips_and_defaults_when_missing() {
        let dir = std::env::temp_dir().join(format!(
            "sudoku-repo-test-{}-{}",
            std::process::id(),
            Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let path = dir.join("state.json");
        let repo = AtomicJsonRepository::new(path.clone());

        assert_eq!(
            repo.load().expect("default load"),
            PersistedState::default()
        );

        let state = PersistedState {
            next_game_id: 42,
            win_streak_current: 3,
            win_streak_best: 5,
            ..PersistedState::default()
        };
        repo.save(&state).expect("atomic save");
        assert_eq!(repo.load().expect("reload"), state);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn system_clock_returns_valid_date() {
        let today = SystemClock.today();
        assert!(today.year() >= 2024);
        assert!((1..=12).contains(&today.month()));
    }
}
