use anyhow::{Error, bail};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
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
