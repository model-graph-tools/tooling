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

/// Pushes a batch of images in parallel across items, but sequential per item
/// (data image first, then model manifest) to avoid concurrent podman operations
/// on related images.
async fn push_batch(items: &[&MetaItem]) -> anyhow::Result<Vec<CommandStatus>> {
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for item in items {
        let image = Neo4JImage::new(item);
        let display = item.short_name();
        let data_tag = image.data_image_tag();
        let model_tag = image.image_tag();
        let data_progress = Progress::join(&multi_progress, &format!("{} (data)", display));
        let model_progress = Progress::join(&multi_progress, &format!("{} (model)", display));

        tasks.spawn(async move {
            let data_status = push_one_image(&["push", &data_tag], &data_progress).await;
            let model_status =
                push_one_image(&["manifest", "push", &model_tag], &model_progress).await;
            vec![data_status, model_status]
        });
    }

    Ok(tasks.join_all().await.into_iter().flatten().collect())
}

/// Pushes a single image or manifest, tracking progress via stderr.
async fn push_one_image(args: &[&str], progress: &Progress) -> CommandStatus {
    let mut cmd = match container_command() {
        Ok(cmd) => cmd,
        Err(e) => return progress.finish_status(false, &e.to_string()),
    };
    for arg in args {
        cmd.arg(arg);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return progress.finish_status(false, &e.to_string()),
    };

    let stderr_lines = match stderr_reader(&mut child) {
        Ok(lines) => lines,
        Err(e) => return progress.finish_status(false, &e.to_string()),
    };

    let progress_clone = progress.clone();
    let stderr_handle = tokio::spawn(async move {
        let mut lines = stderr_lines;
        let mut collected = Vec::new();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    progress_clone.show_progress(&line);
                    collected.push(line);
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }
        collected
    });

    let exit = child.wait().await;
    let stderr_output = stderr_handle.await.unwrap_or_default();
    let error_msg = stderr_output.join(" ");

    match exit {
        Ok(status) if status.success() => progress.finish_status(true, ""),
        Ok(_) => progress.finish_status(false, &error_msg),
        Err(e) => progress.finish_status(false, &e.to_string()),
    }
}
