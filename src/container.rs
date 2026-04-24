use anyhow::{Error, bail};
use std::path::PathBuf;
use tokio::process::Command;
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
