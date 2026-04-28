//! File download utility with progress reporting and local caching.

use crate::progress::Progress;
use anyhow::anyhow;
use std::env::temp_dir;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Downloads a file from `url` to a temporary directory using `filename`.
///
/// Returns the local path immediately if the file already exists.
pub async fn download_file(
    url: &str,
    filename: &str,
    progress: &Progress,
) -> anyhow::Result<PathBuf> {
    let path = temp_dir().join(filename);
    if path.exists() {
        return Ok(path);
    }

    progress.show_progress(&format!("Downloading {filename}..."));
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let mut file = File::create(&path)?;
        let content = response.bytes().await?;
        file.write_all(&content)?;
        Ok(path)
    } else {
        Err(anyhow!("Failed to download {}: {}", url, response.status()))
    }
}
