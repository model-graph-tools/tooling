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
            let image = Neo4JImage::new(item);
            let model_tag = image.image_tag();
            let data_tag = image.data_image_tag();
            let model_ok = local.contains(&model_tag);
            let data_ok = local.contains(&data_tag);
            if !model_ok {
                eprintln!(
                    "  {} {} not found locally, skipping",
                    style("\u{26a0}").yellow(),
                    style(&model_tag).cyan()
                );
            }
            if !data_ok {
                eprintln!(
                    "  {} {} not found locally, skipping",
                    style("\u{26a0}").yellow(),
                    style(&data_tag).cyan()
                );
            }
            model_ok && data_ok
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
        let mut all_status: Vec<CommandStatus> = Vec::with_capacity(count * 2);
        for chunk in pushable.chunks(chunk_size as usize) {
            let status = push_batch(chunk).await?;
            all_status.extend(status);
        }
        summary(all_status.len(), &all_status);
    } else {
        let status = push_batch(&pushable).await?;
        summary(status.len(), &status);
    }
    done(instant);
    Ok(())
}

/// Pushes a batch of images in parallel (both data and model images).
async fn push_batch(items: &[&MetaItem]) -> anyhow::Result<Vec<CommandStatus>> {
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for item in items {
        let image = Neo4JImage::new(item);
        let display = item.short_name();

        // Push data image (regular push, single-arch)
        let data_tag = image.data_image_tag();
        let data_display = format!("{} (data)", display);
        let data_progress = Progress::join(&multi_progress, &data_display);
        let mut data_cmd = container_command()?;
        data_cmd
            .arg("push")
            .arg(&data_tag)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut data_child = data_cmd.spawn()?;
        let data_stderr = stderr_reader(&mut data_child)?;
        let data_progress_clone = data_progress.clone();
        tasks.spawn(async move {
            let output = data_child.wait_with_output().await;
            data_progress.finish_output(output, None)
        });
        tokio::spawn(async move {
            data_progress_clone.trace_progress(data_stderr).await;
        });

        // Push model image (manifest push, multi-arch)
        let model_tag = image.image_tag();
        let model_display = format!("{} (model)", display);
        let model_progress = Progress::join(&multi_progress, &model_display);
        let mut model_cmd = container_command()?;
        model_cmd
            .arg("manifest")
            .arg("push")
            .arg(&model_tag)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut model_child = model_cmd.spawn()?;
        let model_stderr = stderr_reader(&mut model_child)?;
        let model_progress_clone = model_progress.clone();
        tasks.spawn(async move {
            let output = model_child.wait_with_output().await;
            model_progress.finish_output(output, None)
        });
        tokio::spawn(async move {
            model_progress_clone.trace_progress(model_stderr).await;
        });
    }

    Ok(tasks.join_all().await)
}
