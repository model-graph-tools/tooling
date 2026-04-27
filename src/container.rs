use crate::progress::Progress;
use anyhow::{Error, bail};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use which::which;
use wildfly_container_versions::WildFlyContainer;

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

pub fn network_name(wildfly_container: &WildFlyContainer) -> String {
    format!("mgt-network-{}", wildfly_container.identifier)
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

const MGT_NEO4J_PREFIX: &str = "mgt-neo4j-";

pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: String,
    pub ports: String,
}

pub async fn neo4j_container_details() -> anyhow::Result<Vec<ContainerInfo>> {
    let mut cmd = container_command()?;
    cmd.arg("ps")
        .arg("--filter")
        .arg(format!("name={}", MGT_NEO4J_PREFIX))
        .arg("--format")
        .arg("{{.ID}}|{{.Names}}|{{.Status}}|{{.Ports}}")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to list containers: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut containers: Vec<ContainerInfo> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                let name = parts[1].to_string();
                let version = name.strip_prefix(MGT_NEO4J_PREFIX).unwrap_or(&name).to_string();
                Some(ContainerInfo {
                    id: parts[0].to_string(),
                    name,
                    version,
                    status: parts[2].to_string(),
                    ports: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();
    containers.sort_by(|a, b| a.version.cmp(&b.version));
    Ok(containers)
}

pub async fn running_neo4j_containers() -> anyhow::Result<Vec<String>> {
    let mut cmd = container_command()?;
    cmd.arg("ps")
        .arg("--filter")
        .arg(format!("name={}", MGT_NEO4J_PREFIX))
        .arg("--format")
        .arg("{{.Names}}")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to list containers: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let names = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();
    Ok(names)
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
