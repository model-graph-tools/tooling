use crate::container::{running_neo4j_containers, stop_container, verify_container_command};
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::progress::{CommandStatus, Progress, done, summary};
use crate::source::Source;
use console::style;
use indicatif::MultiProgress;
use tokio::task::JoinSet;
use tokio::time::Instant;

pub async fn stop(sources: Option<&[Source]>, all: bool) -> anyhow::Result<()> {
    verify_container_command()?;

    let container_names: Vec<String> = if all {
        running_neo4j_containers()
            .await?
            .iter()
            .map(|r| r.container.container_name())
            .collect()
    } else {
        sources
            .expect("Argument <identifier> expected!")
            .iter()
            .map(|s| Neo4JContainer::new(Neo4JImage::new(s)).container_name())
            .collect()
    };

    if container_names.is_empty() {
        println!("\nNo running Neo4J model DB containers found.");
        return Ok(());
    }

    let count = container_names.len();
    let noun = if count == 1 { "container" } else { "containers" };
    println!(
        "\n{}",
        style(format!("Stopping {} Neo4J model DB {}", count, noun)).bold()
    );

    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for name in container_names {
        let progress = Progress::join(&multi_progress, &name);
        tasks.spawn(async move {
            let result = stop_container(&name).await;
            match &result {
                Ok(()) => progress.finish_success(Some("stopped")),
                Err(e) => progress.finish_error(&e.to_string()),
            }
            CommandStatus::from_result(&name, &result)
        });
    }

    let status = tasks.join_all().await;
    summary(count, &status);
    done(instant);
    Ok(())
}
