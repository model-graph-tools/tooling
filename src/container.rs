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

pub fn verify_container_command() -> Result<PathBuf, Error> {
    which("podman")
        .or_else(|_| which("docker"))
        .map_err(|_| anyhow::anyhow!("podman or docker not found"))
}

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

pub fn network_name(source: &Source) -> String {
    format!("mgt-network-{}", source.container_id())
}

pub async fn create_network(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("network").arg("create");
    if is_podman() {
        cmd.arg("--ignore");
    }
    cmd.arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("already exists") {
            bail!("Failed to create network {}: {}", name, stderr);
        }
    }
    Ok(())
}

pub async fn remove_network(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("network")
        .arg("rm")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to remove network {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

pub async fn stop_container(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("stop")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to stop {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

pub async fn remove_volume(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("volume")
        .arg("rm")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to remove volume {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

pub async fn remove_container(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("rm")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to remove container {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

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
