//! Stops running Neo4J model DB containers.

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

    let container_names: Vec<String> = if all {
        running_neo4j_containers()
            .await?
            .iter()
            .map(|r| r.container.container_name())
            .collect()
    } else {
        items
            .expect("Argument <identifier> expected!")
            .iter()
            .map(|item| Neo4JContainer::new(Neo4JImage::new(item)).container_name())
            .collect()
    };

    if container_names.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("\nNo running Neo4J model DB containers found.");
        }
        return Ok(());
    }

    let count = container_names.len();
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

    for name in container_names {
        let progress = match &multi_progress {
            Some(mp) => Progress::join(mp, &name),
            None => Progress::hidden(&name),
        };
        tasks.spawn(async move {
            let result = stop_container(&name).await;
            if !json {
                match &result {
                    Ok(()) => progress.finish_success(Some("Stopped")),
                    Err(e) => progress.finish_error(&e.to_string()),
                }
            }
            (name, result)
        });
    }

    let results = tasks.join_all().await;

    if json {
        let command_results: Vec<CommandResult> = results
            .into_iter()
            .map(|(name, result)| match result {
                Ok(()) => CommandResult {
                    identifier: name,
                    success: true,
                    bolt: None,
                    http: None,
                    error: None,
                },
                Err(e) => CommandResult {
                    identifier: name,
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
            .map(|(name, result)| CommandStatus::from_result(name, result))
            .collect();
        summary(count, &status);
        done(instant);
    }
    Ok(())
}
