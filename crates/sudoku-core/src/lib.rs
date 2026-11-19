//! Pure, Tauri-agnostic Sudoku domain engine.

#![deny(clippy::unwrap_used)]

/// Returns the crate version string for smoke verification.
pub const fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_version_is_non_empty() {
        assert!(!core_version().is_empty());
    }
}
