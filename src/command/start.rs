//! Starts Neo4J containers from pre-built images.

use crate::container::{container_command, healthcheck, pull_image, verify_container_command};
use crate::label::Label;
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::progress::{CommandStatus, Progress, done, summary};
use anyhow::bail;
use console::style;
use indicatif::MultiProgress;
use std::process::Stdio;
use tokio::task::JoinSet;
use tokio::time::Instant;
use wildfly_meta::MetaItem;

/// Starts Neo4J containers for the given meta items from their pre-built images.
pub async fn start(items: &[MetaItem]) -> anyhow::Result<()> {
    verify_container_command()?;

    let count = items.len();
    let noun = if count == 1 {
        "container"
    } else {
        "containers"
    };
    println!(
        "\n{}",
        style(format!("Starting {} Neo4J model DB {}", count, noun)).bold()
    );

    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for item in items {
        let image = Neo4JImage::new(item);
        let neo4j = Neo4JContainer::new(image);
        let display = item.short_name();
        let item = item.clone();
        let progress = Progress::join(&multi_progress, &display);
        tasks.spawn(async move {
            let result = start_neo4j(&neo4j, &item, &progress).await;
            match &result {
                Ok(()) => {
                    progress
                        .finish_success(Some(&format!("http://localhost:{}", neo4j.ports.http)));
                }
                Err(e) => progress.finish_error(&e.to_string()),
            }
            CommandStatus::from_result(&display, &result)
        });
    }

    let status = tasks.join_all().await;
    summary(count, &status);
    done(instant);
    Ok(())
}

/// Starts a single Neo4J container from a pre-built image and waits for it to become healthy.
async fn start_neo4j(
    neo4j: &Neo4JContainer,
    item: &MetaItem,
    progress: &Progress,
) -> anyhow::Result<()> {
    let _ = pull_image(&neo4j.image.image_tag(), progress).await;

    progress.show_progress("Starting container...");
    let mut cmd = container_command()?;
    cmd.arg("run")
        .arg("--rm")
        .arg("--detach")
        .arg("--name")
        .arg(neo4j.container_name())
        .arg("--publish")
        .arg(format!("{}:7474", neo4j.ports.http))
        .arg("--publish")
        .arg(format!("{}:7687", neo4j.ports.bolt))
        .arg("--env")
        .arg("NEO4J_AUTH=none")
        .arg("--env")
        .arg(format!(
            "NEO4J_browser_post__connect__cmd=play http://localhost:{}/welcome.html",
            neo4j.ports.http
        ))
        .arg("--label")
        .arg(Label::Identifier.run_arg(&item.container_name()))
        .arg("--label")
        .arg(Label::SourceType.run_arg(item.kind()))
        .arg("--label")
        .arg(Label::SourceName.run_arg(&item.expression()))
        .arg(neo4j.image.image_tag())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to start Neo4J: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    progress.show_progress("Waiting for Neo4J...");
    healthcheck(
        &format!("http://localhost:{}/browser", neo4j.ports.http),
        progress,
    )
    .await?;

    Ok(())
}
