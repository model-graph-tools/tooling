//! Stops running Neo4J model DB containers.

use anyhow::Context;

use crate::container::{running_neo4j_containers, stop_container, verify_container_command};
use crate::json::CommandResult;
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::progress::{CommandStatus, Progress, done, summary};
use console::style;
use indicatif::MultiProgress;
use tokio::task::JoinSet;
use tokio::time::Instant;
use wildfly_meta::MetaItem;

/// Stops Neo4J containers by meta item identifier, or all if `--all` is passed.
pub async fn stop(items: Option<&[MetaItem]>, all: bool, json: bool) -> anyhow::Result<()> {
    verify_container_command()?;

    let containers: Vec<(String, String)> = if all {
        running_neo4j_containers()
            .await?
            .iter()
            .map(|r| {
                (
                    r.container.container_name(),
                    r.container.image.item.expression(),
                )
            })
            .collect()
    } else {
        items
            .ok_or_else(|| anyhow::anyhow!("Argument <identifier> expected"))?
            .iter()
            .map(|item| {
                let container_name = Neo4JContainer::new(Neo4JImage::new(item))?.container_name();
                Ok((container_name, item.expression()))
            })
            .collect::<anyhow::Result<Vec<_>>>()?
    };

    if containers.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("\nNo running Neo4J model DB containers found.");
        }
        return Ok(());
    }

    let count = containers.len();
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
            style(format!("Stopping {} Neo4J model DB {}", count, noun)).bold()
        );
        Some(MultiProgress::new())
    };

    let instant = Instant::now();
    let mut tasks = JoinSet::new();

    for (container_name, expression) in containers {
        let progress = match &multi_progress {
            Some(mp) => Progress::join(mp, &container_name),
            None => Progress::hidden(&container_name),
        };
        tasks.spawn(async move {
            let result = stop_container(&container_name).await;
            if !json {
                match &result {
                    Ok(()) => progress.finish_success(Some("Stopped")),
                    Err(e) => progress.finish_error(&e.to_string()),
                }
            }
            (container_name, expression, result)
        });
    }

    let results = tasks.join_all().await;

    if json {
        let command_results: Vec<CommandResult> = results
            .into_iter()
            .map(|(_, identifier, result)| match result {
                Ok(()) => CommandResult::success(identifier, None, None),
                Err(e) => CommandResult::error(identifier, &e),
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string(&command_results).context("Failed to serialize JSON output")?
        );
    } else {
        let status: Vec<CommandStatus> = results
            .iter()
            .map(|(name, _, result)| CommandStatus::from_result(name, result))
            .collect();
        summary(count, &status);
        done(instant);
    }
    Ok(())
}
