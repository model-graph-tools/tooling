//! File download utility with progress reporting and local caching.

use crate::progress::Progress;
use anyhow::{Context, anyhow};
use std::env::temp_dir;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// Downloads a file from `url` to a temporary directory using `filename`.
///
/// Returns the local path immediately if a non-empty cached copy exists.
/// Writes to a `.tmp` file first and renames on success to avoid leaving
/// corrupt zero-byte files after partial downloads.
pub async fn download_file(
    url: &str,
    filename: &str,
    progress: &Progress,
) -> anyhow::Result<PathBuf> {
    let path = temp_dir().join(filename);
    if path.exists() && fs::metadata(&path).map_or(false, |m| m.len() > 0) {
        return Ok(path);
    }

    progress.show_progress(&format!("Downloading {filename}..."));
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let content = response
            .bytes()
            .await
            .context(format!("Failed to download content from {url}"))?;
        if content.is_empty() {
            return Err(anyhow!("Downloaded file is empty: {url}"));
        }

        let tmp_path = temp_dir().join(format!("{filename}.tmp"));
        let mut file =
            File::create(&tmp_path).context(format!("Failed to create {}", tmp_path.display()))?;
        file.write_all(&content)
            .context(format!("Failed to write to {}", tmp_path.display()))?;
        fs::rename(&tmp_path, &path).context(format!(
            "Failed to rename {} to {}",
            tmp_path.display(),
            path.display()
        ))?;
        Ok(path)
    } else {
        Err(anyhow!("Failed to download {}: {}", url, response.status()))
    }
}
