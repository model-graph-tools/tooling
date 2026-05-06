//! Container runtime abstraction (podman/docker) and common operations.

use crate::label::Label;
use crate::neo4j::{Neo4JContainer, Neo4JImage, RunningNeo4JContainer};
use crate::progress::Progress;
use crate::registry::{images_registry, packs_registry};
use crate::error::MgtError;
use anyhow::Error;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use which::which;
use wildfly_meta::{MetaItem, parse_meta_item};

/// Verifies that `podman` or `docker` is available on PATH.
pub fn verify_container_command() -> Result<PathBuf, Error> {
    which("podman")
        .or_else(|_| which("docker"))
        .map_err(|_| MgtError::container_runtime_not_found().into())
}

/// Returns a `Command` pre-configured with the container runtime (podman or docker).
pub fn container_command() -> anyhow::Result<Command> {
    if let Ok(podman_path) = which("podman") {
        Ok(Command::new(podman_path))
    } else if let Ok(docker_path) = which("docker") {
        Ok(Command::new(docker_path))
    } else {
        Err(MgtError::container_runtime_not_found().into())
    }
}

/// Returns `true` if `podman` is available (used to enable podman-specific flags).
fn is_podman() -> bool {
    which("podman").is_ok()
}

/// Derives the container network name from a meta item identifier.
pub fn network_name(item: &MetaItem) -> String {
    format!("mgt-network-{}", item.container_name())
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
            return Err(MgtError::network_create_failed(name, stderr.trim_end()).into());
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
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MgtError::container_command_failed(error_context, stderr.trim_end()).into());
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
    let source_name_label = Label::SourceName;
    let mut cmd = container_command()?;
    cmd.arg("ps")
        .arg("--filter")
        .arg(Label::Identifier.filter())
        .arg("--format")
        .arg(format!(
            "{{{{.ID}}}}|{{{{.Names}}}}|{{{{.Status}}}}|{}",
            source_name_label.format_expr()
        ))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MgtError::container_list_failed(stderr.trim_end()).into());
    }
    let mut containers: Vec<RunningNeo4JContainer> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                let name = source_name_label.parse_value(parts[3])?;
                let item = parse_meta_item(&name, images_registry(), packs_registry()).ok()?;
                let image = Neo4JImage::new(&item);
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
            .item
            .port_offset()
            .cmp(&b.container.image.item.port_offset())
    });
    Ok(containers)
}

/// Returns the names of all locally available container images.
pub async fn local_image_names() -> anyhow::Result<HashSet<String>> {
    let mut cmd = container_command()?;
    cmd.arg("images")
        .arg("--format")
        .arg("{{.Repository}}:{{.Tag}}")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MgtError::image_list_failed(stderr.trim_end()).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(String::from)
        .collect())
}

/// Pulls a container image only if it is not already available locally.
pub async fn pull_image(image: &str, progress: &Progress) -> anyhow::Result<()> {
    progress.show_progress(&format!("Checking {}...", image));
    if local_image_names().await?.contains(image) {
        progress.show_progress(&format!("{} already available", image));
        return Ok(());
    }
    progress.show_progress(&format!("Pulling {}...", image));
    let mut cmd = container_command()?;
    cmd.arg("pull")
        .arg(image)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MgtError::image_pull_failed(image, stderr.trim_end()).into());
    }
    Ok(())
}

/// Maximum number of health check polling attempts before giving up.
const MAX_HEALTHCHECK_ATTEMPTS: u32 = 120;

/// Polls a URL until it returns HTTP 200, retrying once per second.
pub async fn healthcheck(url: &str, progress: &Progress) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    for attempt in 1..=MAX_HEALTHCHECK_ATTEMPTS {
        progress.show_progress(&format!(
            "Healthcheck {}/{}",
            attempt, MAX_HEALTHCHECK_ATTEMPTS
        ));
        if let Ok(response) = client.get(url).send().await
            && response.status().is_success()
        {
            return Ok(());
        }
        sleep(Duration::from_secs(1)).await;
    }
    Err(MgtError::healthcheck_failed(url, MAX_HEALTHCHECK_ATTEMPTS).into())
}
