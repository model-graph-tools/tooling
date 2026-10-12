//! Rebuilds Neo4J model DB images with an updated REST API binary.

use super::resolve_targets;
use crate::constants::{REST_API_VERSION, latest_rest_api_version};
use crate::container::{pull_image, verify_container_command};
use crate::neo4j::Neo4JImage;
use crate::progress::{CommandStatus, Progress, done, summary};
use anyhow::bail;
use console::style;
use indicatif::MultiProgress;
use tokio::task::JoinSet;
use tokio::time::Instant;
use wildfly_meta::MetaItem;

/// Rebuilds Neo4J model DB images with an updated REST API binary.
///
/// When `all` is true, repacks all known model images from the registries.
/// When `api_version` is `None`, resolves the latest release from GitHub.
/// Data images are pulled from the remote registry if not available locally.
pub async fn repack(
    items: Option<&[MetaItem]>,
    all: bool,
    api_version: Option<&str>,
    chunk_size: u16,
) -> anyhow::Result<()> {
    verify_container_command()?;

    let api_version = match api_version {
        Some(v) => v.to_string(),
        None => resolve_api_version().await?,
    };

    let targets = resolve_targets(items, all)?;
    if targets.is_empty() {
        bail!("No images found to repack");
    }

    let count = targets.len();
    let noun = if count == 1 { "image" } else { "images" };
    println!(
        "\n{}",
        style(format!(
            "Repacking {} Neo4J model DB {} with REST API v{}",
            count, noun, api_version
        ))
        .bold()
    );

    let instant = Instant::now();
    if chunk_size > 0 {
        let mut all_status: Vec<CommandStatus> = Vec::with_capacity(count);
        for chunk in targets.chunks(chunk_size as usize) {
            let status = repack_batch(chunk, &api_version).await;
            all_status.extend(status);
        }
        summary(count, &all_status);
    } else {
        let status = repack_batch(&targets, &api_version).await;
        summary(count, &status);
    }
    done(instant);
    Ok(())
}

/// Repacks a batch of images in parallel.
async fn repack_batch(items: &[MetaItem], api_version: &str) -> Vec<CommandStatus> {
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for item in items {
        let image = Neo4JImage::new(item);
        let display = item.short_name();
        let progress = Progress::join(&multi_progress, &display);
        let version = api_version.to_string();

        tasks.spawn(async move {
            let result = repack_image(&image, &version, &progress).await;
            match &result {
                Ok(()) => progress.finish_success(Some(&format!("v{version}"))),
                Err(e) => progress.finish_error(&e.to_string()),
            }
            CommandStatus::from_result(&display, &result)
        });
    }

    tasks.join_all().await
}

/// Resolves the REST API version: latest from GitHub, falling back to the
/// compiled-in constant.
async fn resolve_api_version() -> anyhow::Result<String> {
    match latest_rest_api_version().await {
        Ok(v) => Ok(v),
        Err(e) => {
            eprintln!(
                "  {} Could not fetch latest REST API version ({}), using v{}",
                style("\u{26a0}").yellow(),
                e,
                REST_API_VERSION
            );
            Ok(REST_API_VERSION.to_string())
        }
    }
}

/// Pulls the data image and rebuilds the model image with the given REST API version.
async fn repack_image(
    image: &Neo4JImage,
    rest_api_version: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    let data_tag = image.data_image_tag();
    pull_image(&data_tag, progress).await?;
    image.build_image(rest_api_version, progress).await
}
