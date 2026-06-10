//! Atomic file writes used by migrate output.

use crate::error::Error;
use std::path::Path;
use tokio::fs;

/// Write `content` to `path`, creating parent directories as needed.
///
/// When the file already exists and `force` is false, returns
/// [`WriteOutcome::Skipped`] without modifying the file.
pub async fn write_file_atomic(
    path: &Path,
    content: &str,
    force: bool,
) -> Result<WriteOutcome, Error> {
    if path.exists() && !force {
        return Ok(WriteOutcome::Skipped);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content).await?;
    fs::rename(&temp_path, path).await?;
    Ok(WriteOutcome::Written)
}

/// Result of [`write_file_atomic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOutcome {
    /// File was written (or overwritten when `force` is true).
    Written,
    /// File already existed and `force` was false.
    Skipped,
}
