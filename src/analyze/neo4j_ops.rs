//! Neo4J container lifecycle operations shared across analysis pipelines.

use crate::container::{container_command, healthcheck};
use crate::label::Label;
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::progress::Progress;
use anyhow::bail;
use std::process::Stdio;

/// Creates a Neo4J volume and starts the container on the given network.
///
/// Waits for Neo4J to become healthy before returning.
pub(super) async fn start_neo4j(
    neo4j: &Neo4JContainer,
    network: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    progress.show_progress("creating volume...");
    let mut volume_cmd = container_command()?;
    volume_cmd
        .arg("volume")
        .arg("create")
        .arg("--ignore")
        .arg(neo4j.volume_name())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = volume_cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to create volume: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    progress.show_progress("starting container...");
    let mut run_cmd = container_command()?;
    run_cmd
        .arg("run")
        .arg("--detach")
        .arg("--name")
        .arg(neo4j.container_name())
        .arg("--network")
        .arg(network)
        .arg("--publish")
        .arg(format!("{}:7687", neo4j.ports.bolt))
        .arg("--publish")
        .arg(format!("{}:7474", neo4j.ports.http))
        .arg("--env")
        .arg("NEO4J_AUTH=none")
        .arg("--volume")
        .arg(format!("{}:/data", neo4j.volume_name()))
        .arg("--label")
        .arg(Label::Identifier.run_arg(&neo4j.image.source.container_id()))
        .arg(Neo4JImage::base_image_name())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = run_cmd.output().await?;
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
