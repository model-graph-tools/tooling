use crate::container::{container_command, healthcheck, verify_container_command};
use crate::label::Label;
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::progress::{CommandStatus, Progress, done, summary};
use crate::source::Source;
use anyhow::bail;
use console::style;
use indicatif::MultiProgress;
use std::process::Stdio;
use tokio::task::JoinSet;
use tokio::time::Instant;

pub async fn start(sources: &[Source]) -> anyhow::Result<()> {
    verify_container_command()?;

    let count = sources.len();
    let noun = if count == 1 { "container" } else { "containers" };
    println!(
        "\n{}",
        style(format!("Starting {} Neo4J model DB {}", count, noun)).bold()
    );

    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for source in sources {
        let image = Neo4JImage::new(source);
        let neo4j = Neo4JContainer::new(image);
        let display = source.display_name();
        let progress = Progress::join(&multi_progress, &display);
        tasks.spawn(async move {
            let result = start_neo4j(&neo4j, &progress).await;
            match &result {
                Ok(()) => {
                    progress.finish_success(Some(&format!(
                        "http://localhost:{}",
                        neo4j.ports.http
                    )));
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

async fn start_neo4j(neo4j: &Neo4JContainer, progress: &Progress) -> anyhow::Result<()> {
    progress.show_progress("starting container...");
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
        .arg("--label")
        .arg(Label::Identifier.run_arg(&neo4j.image.source.container_id()))
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

    progress.show_progress("waiting for Neo4J...");
    healthcheck(
        &format!("http://localhost:{}/browser", neo4j.ports.http),
        progress,
    )
    .await?;

    Ok(())
}
