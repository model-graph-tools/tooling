//! Pushes Neo4J model DB images to the remote registry.

use crate::container::{container_command, local_image_names, verify_container_command};
use crate::neo4j::Neo4JImage;
use crate::progress::{CommandStatus, Progress, done, stderr_reader, summary};
use anyhow::bail;
use console::style;
use indicatif::MultiProgress;
use std::process::Stdio;
use tokio::task::JoinSet;
use tokio::time::Instant;
use wildfly_meta::MetaItem;

/// Pushes Neo4J model DB images for the given meta items to the remote registry.
pub async fn push(items: &[MetaItem], chunk_size: u16) -> anyhow::Result<()> {
    verify_container_command()?;

    let local = local_image_names().await?;
    let pushable: Vec<&MetaItem> = items
        .iter()
        .filter(|item| {
            let tag = Neo4JImage::new(item).image_tag();
            if local.contains(&tag) {
                true
            } else {
                eprintln!(
                    "  {} {} not found locally, skipping",
                    style("\u{26a0}").yellow(),
                    style(&tag).cyan()
                );
                false
            }
        })
        .collect();

    if pushable.is_empty() {
        bail!("No local images found for the given identifiers");
    }

    let count = pushable.len();
    let noun = if count == 1 { "image" } else { "images" };
    println!(
        "\n{}",
        style(format!("Pushing {} Neo4J model DB {}", count, noun)).bold()
    );

    let instant = Instant::now();
    if chunk_size > 0 {
        let mut all_status: Vec<CommandStatus> = Vec::with_capacity(count);
        for chunk in pushable.chunks(chunk_size as usize) {
            let status = push_batch(chunk).await?;
            all_status.extend(status);
        }
        summary(count, &all_status);
    } else {
        let status = push_batch(&pushable).await?;
        summary(count, &status);
    }
    done(instant);
    Ok(())
}

/// Pushes a batch of images in parallel.
async fn push_batch(items: &[&MetaItem]) -> anyhow::Result<Vec<CommandStatus>> {
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for item in items {
        let image = Neo4JImage::new(item);
        let image_tag = image.image_tag();
        let display = item.short_name();
        let progress = Progress::join(&multi_progress, &display);

        let mut cmd = container_command()?;
        cmd.arg("manifest")
            .arg("push")
            .arg(&image_tag)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let stderr = stderr_reader(&mut child)?;
        let progress_clone = progress.clone();

        tasks.spawn(async move {
            let output = child.wait_with_output().await;
            progress.finish_output(output, None)
        });
        tokio::spawn(async move {
            progress_clone.trace_progress(stderr).await;
        });
    }

    Ok(tasks.join_all().await)
}
