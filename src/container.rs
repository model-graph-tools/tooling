//! Container runtime abstraction (podman/docker) and common operations.

use crate::label::Label;
use crate::neo4j::{Neo4JContainer, Neo4JImage, RunningNeo4JContainer};
use crate::progress::Progress;
use crate::source::Source;
use anyhow::{Error, bail};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use which::which;

/// Verifies that `podman` or `docker` is available on PATH.
pub fn verify_container_command() -> Result<PathBuf, Error> {
    which("podman")
        .or_else(|_| which("docker"))
        .map_err(|_| anyhow::anyhow!("podman or docker not found"))
}

/// Returns a `Command` pre-configured with the container runtime (podman or docker).
pub fn container_command() -> anyhow::Result<Command> {
    if let Ok(podman_path) = which("podman") {
        Ok(Command::new(podman_path))
    } else if let Ok(docker_path) = which("docker") {
        Ok(Command::new(docker_path))
    } else {
        bail!("podman or docker not found")
    }
}

fn is_podman() -> bool {
    which("podman").is_ok()
}

/// Derives the container network name from a source identifier.
pub fn network_name(source: &Source) -> String {
    format!("mgt-network-{}", source.container_id())
}

/// Creates a container network, tolerating "already exists" errors.
pub async fn create_network(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("network").arg("create");
    if is_podman() {
        cmd.arg("--ignore");
    }
    cmd.arg(name).stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("already exists") {
            bail!("Failed to create network {}: {}", name, stderr);
        }
    }
    Ok(())
}

/// Runs a container subcommand, bailing with `error_context` on failure.
pub async fn run_container_cmd(args: &[&str], error_context: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    for arg in args {
        cmd.arg(arg);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "{}: {}",
            error_context,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

/// Removes a container network.
pub async fn remove_network(name: &str) -> anyhow::Result<()> {
    run_container_cmd(
        &["network", "rm", name],
        &format!("Failed to remove network {name}"),
    )
    .await
}

/// Stops a running container.
pub async fn stop_container(name: &str) -> anyhow::Result<()> {
    run_container_cmd(&["stop", name], &format!("Failed to stop {name}")).await
}

/// Removes a container volume.
pub async fn remove_volume(name: &str) -> anyhow::Result<()> {
    run_container_cmd(
        &["volume", "rm", name],
        &format!("Failed to remove volume {name}"),
    )
    .await
}

/// Removes a stopped container.
pub async fn remove_container(name: &str) -> anyhow::Result<()> {
    run_container_cmd(&["rm", name], &format!("Failed to remove container {name}")).await
}

/// Lists running Neo4J containers filtered by the `mgt` identifier label, sorted by port offset.
pub async fn running_neo4j_containers() -> anyhow::Result<Vec<RunningNeo4JContainer>> {
    let label = Label::Identifier;
    let mut cmd = container_command()?;
    cmd.arg("ps")
        .arg("--filter")
        .arg(label.filter())
        .arg("--format")
        .arg(format!(
            "{{{{.ID}}}}|{{{{.Names}}}}|{{{{.Status}}}}|{}",
            label.format_expr()
        ))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to list containers: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut containers: Vec<RunningNeo4JContainer> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                let identifier = label.parse_value(parts[3])?;
                let source = Source::parse(&identifier).ok()?;
                let image = Neo4JImage::new(&source);
                let container = Neo4JContainer::new(image);
                Some(RunningNeo4JContainer {
                    container,
                    id: parts[0].to_string(),
                    status: parts[2].to_string(),
                })
            } else {
                None
            }
        })
        .collect();
    containers.sort_by(|a, b| {
        a.container
            .image
            .source
            .port_offset()
            .cmp(&b.container.image.source.port_offset())
    });
    Ok(containers)
}

const MAX_HEALTHCHECK_ATTEMPTS: u32 = 30;

/// Polls a URL until it returns HTTP 200, retrying once per second.
pub async fn healthcheck(url: &str, progress: &Progress) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    for attempt in 1..=MAX_HEALTHCHECK_ATTEMPTS {
        progress.show_progress(&format!(
            "healthcheck {}/{}",
            attempt, MAX_HEALTHCHECK_ATTEMPTS
        ));
        if let Ok(response) = client.get(url).send().await
            && response.status().is_success()
        {
            return Ok(());
        }
        sleep(Duration::from_secs(1)).await;
    }
    bail!(
        "Healthcheck failed after {} attempts: {}",
        MAX_HEALTHCHECK_ATTEMPTS,
        url
    )
}
