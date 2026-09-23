//! Rebuilds Neo4J model DB images with an updated REST API binary.

use crate::constants::{REST_API_VERSION, latest_rest_api_version};
use crate::container::{
    container_command, pull_image, remove_container, stop_container, verify_container_command,
};
use crate::error::MgtError;
use crate::neo4j::Neo4JImage;
use crate::progress::{CommandStatus, Progress, done, summary};
use crate::registry::{images_registry, packs_registry};
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
/// Images not available locally are pulled from the remote registry.
pub async fn repack(
    items: Option<&[MetaItem]>,
    all: bool,
    api_version: Option<&str>,
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
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for item in targets {
        let image = Neo4JImage::new(&item);
        let display = item.short_name();
        let progress = Progress::join(&multi_progress, &display);
        let version = api_version.clone();

        tasks.spawn(async move {
            let result = repack_image(&image, &version, &progress).await;
            match &result {
                Ok(()) => progress.finish_success(Some(&format!("v{version}"))),
                Err(e) => progress.finish_error(&e.to_string()),
            }
            CommandStatus::from_result(&display, &result)
        });
    }

    let status: Vec<CommandStatus> = tasks.join_all().await;
    summary(count, &status);
    done(instant);
    Ok(())
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

/// Resolves the list of `MetaItem`s to repack.
///
/// When `--all` is passed, returns ALL known versions and feature packs from
/// the registries (images will be pulled from remote if not available locally).
/// For explicit identifiers, all specified items are included.
fn resolve_targets(items: Option<&[MetaItem]>, all: bool) -> anyhow::Result<Vec<MetaItem>> {
    if all {
        let images = images_registry()?;
        let packs = packs_registry()?;

        let mut targets: Vec<MetaItem> = Vec::new();
        for img in images.all() {
            targets.push(MetaItem::Image(img.clone()));
        }
        for fp in packs.all() {
            targets.push(MetaItem::FeaturePack(fp.clone()));
        }
        Ok(targets)
    } else {
        let provided = items.ok_or_else(|| anyhow::anyhow!("Argument <identifier> expected"))?;
        Ok(provided.to_vec())
    }
}

/// Repacks a single image: ensures the image is available locally (pulling
/// from the remote registry if needed), creates a temporary container to
/// extract the Neo4J data, removes the container, and rebuilds the image
/// with the new REST API version.
async fn repack_image(
    image: &Neo4JImage,
    rest_api_version: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    let image_tag = image.image_tag();
    let temp_container = format!("mgt-repack-{}", image.item.container_name());

    pull_image(&image_tag, progress).await?;

    progress.show_progress("Starting temporary container...");
    let mut cmd = container_command()?;
    cmd.arg("run")
        .arg("--detach")
        .arg("--name")
        .arg(&temp_container)
        .arg(&image_tag)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MgtError::repack_failed(&image_tag, stderr.trim_end()).into());
    }

    let rebuild_result = image.build_image(rest_api_version, progress).await;

    progress.show_progress("Cleaning up temporary container...");
    if let Err(e) = stop_container(&temp_container).await {
        eprintln!(
            "  {} Failed to stop temporary container {}: {}",
            style("\u{26a0}").yellow(),
            temp_container,
            e
        );
    }
    if let Err(e) = remove_container(&temp_container).await {
        eprintln!(
            "  {} Failed to remove temporary container {}: {}",
            style("\u{26a0}").yellow(),
            temp_container,
            e
        );
    }

    rebuild_result
}
