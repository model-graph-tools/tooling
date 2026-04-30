//! Starts Neo4J containers from pre-built images.

use crate::container::{container_command, healthcheck, pull_image, verify_container_command};
use crate::json::CommandResult;
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
pub async fn start(items: &[MetaItem], json: bool) -> anyhow::Result<()> {
    verify_container_command()?;

    let count = items.len();
    let multi_progress = if json {
        None
    } else {
        let noun = if count == 1 {
            "container"
        } else {
            "containers"
        };
        println!(
            "\n{}",
            style(format!("Starting {} Neo4J model DB {}", count, noun)).bold()
        );
        Some(MultiProgress::new())
    };

    let instant = Instant::now();
    let mut tasks = JoinSet::new();

    for item in items {
        let image = Neo4JImage::new(item);
        let neo4j = Neo4JContainer::new(image);
        let display = item.short_name();
        let item = item.clone();
        let progress = match &multi_progress {
            Some(mp) => Progress::join(mp, &display),
            None => Progress::hidden(&display),
        };
        let bolt = neo4j.ports.bolt;
        let http = neo4j.ports.http;
        tasks.spawn(async move {
            let result = start_neo4j(&neo4j, &item, &progress).await;
            if !json {
                match &result {
                    Ok(()) => {
                        progress.finish_success(Some(&format!(
                            "http://localhost:{}",
                            neo4j.ports.http
                        )));
                    }
                    Err(e) => progress.finish_error(&e.to_string()),
                }
            }
            (display, result, bolt, http)
        });
    }

    let results = tasks.join_all().await;

    if json {
        let command_results: Vec<CommandResult> = results
            .into_iter()
            .map(|(identifier, result, bolt, http)| match result {
                Ok(()) => CommandResult {
                    identifier,
                    success: true,
                    bolt: Some(bolt),
                    http: Some(http),
                    error: None,
                },
                Err(e) => CommandResult {
                    identifier,
                    success: false,
                    bolt: None,
                    http: None,
                    error: Some(e.to_string()),
                },
            })
            .collect();
        println!("{}", serde_json::to_string(&command_results).unwrap());
    } else {
        let status: Vec<CommandStatus> = results
            .iter()
            .map(|(display, result, _, _)| CommandStatus::from_result(display, result))
            .collect();
        summary(count, &status);
        done(instant);
    }
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
