use crate::container::{container_command, healthcheck, verify_container_command};
use crate::neo4j::Neo4J;
use crate::progress::{CommandStatus, Progress, done, summary};
use anyhow::bail;
use console::style;
use indicatif::MultiProgress;
use std::process::Stdio;
use tokio::task::JoinSet;
use tokio::time::Instant;
use wildfly_container_versions::WildFlyContainer;

pub async fn start(wildfly_containers: &[WildFlyContainer]) -> anyhow::Result<()> {
    verify_container_command()?;

    let count = wildfly_containers.len();
    let noun = if count == 1 { "container" } else { "containers" };
    println!(
        "\n{}",
        style(format!("Starting {} Neo4J model DB {}", count, noun)).bold()
    );

    let instant = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for wc in wildfly_containers {
        let neo4j = Neo4J::new(wc);
        let progress = Progress::join(&multi_progress, &wc.display_version());
        tasks.spawn(async move {
            let result = start_neo4j(&neo4j, &progress).await;
            match &result {
                Ok(()) => {
                    progress.finish_success(Some(&format!(
                        "http://localhost:{}",
                        neo4j.http_port
                    )));
                }
                Err(e) => progress.finish_error(&e.to_string()),
            }
            CommandStatus::from_result(&neo4j.wildfly_container.display_version(), &result)
        });
    }

    let status = tasks.join_all().await;
    summary(count, &status);
    done(instant);
    Ok(())
}

async fn start_neo4j(neo4j: &Neo4J, progress: &Progress) -> anyhow::Result<()> {
    progress.show_progress("starting container...");
    let mut cmd = container_command()?;
    cmd.arg("run")
        .arg("--rm")
        .arg("--detach")
        .arg("--name")
        .arg(neo4j.container_name())
        .arg("--publish")
        .arg(format!("{}:7474", neo4j.http_port))
        .arg("--publish")
        .arg(format!("{}:7687", neo4j.bolt_port))
        .arg("--env")
        .arg("NEO4J_AUTH=none")
        .arg(neo4j.image_tag())
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
        &format!("http://localhost:{}/browser", neo4j.http_port),
        progress,
    )
    .await?;

    Ok(())
}
