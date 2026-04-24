use crate::container::container_command;
use crate::progress::Progress;
use anyhow::bail;
use std::process::Stdio;
use wildfly_container_versions::WildFlyContainer;

pub static NEO4J_VERSION: &str = "5.26.12-community";
pub static NEO4J_IMAGE: &str = "docker.io/neo4j";

const DOCKERFILE_TEMPLATE: &str = r#"ARG NEO4J_VERSION=5.26
FROM neo4j:${NEO4J_VERSION}
COPY --chown=neo4j:neo4j databases /data/databases
COPY --chown=neo4j:neo4j transactions /data/transactions
ENV NEO4J_AUTH=none
ENV NEO4J_server_databases_default__to__read__only=true
ENV NEO4J_browser_post__connect__cmd="play https://model-graph-tools.github.io/assets/welcome.html"
ENV NEO4J_browser_remote__content__hostname__whitelist="model-graph-tools.github.io"
"#;

#[derive(Eq, PartialEq, Clone)]
pub struct Neo4J {
    pub wildfly_container: WildFlyContainer,
    pub bolt_port: u16,
    pub http_port: u16,
}

impl Neo4J {
    pub fn new(wildfly_container: &WildFlyContainer) -> Neo4J {
        let offset =
            (wildfly_container.version.major * 10 + wildfly_container.version.minor) as u16;
        Neo4J {
            wildfly_container: wildfly_container.clone(),
            bolt_port: 6000 + offset,
            http_port: 7000 + offset,
        }
    }

    pub fn image_name() -> String {
        format!("{}:{}", NEO4J_IMAGE, NEO4J_VERSION)
    }

    pub fn container_name(&self) -> String {
        format!("mgt-neo4j-{}", self.wildfly_container.identifier)
    }

    pub fn volume_name(&self) -> String {
        format!("mgt-neo4j-data-{}", self.wildfly_container.identifier)
    }

    pub fn image_tag(&self) -> String {
        format!("mgt-neo4j:{}", self.wildfly_container.identifier)
    }

    pub async fn build_image(&self, progress: &Progress) -> anyhow::Result<()> {
        let build_dir = tempfile::tempdir()?;
        let build_path = build_dir.path();

        progress.show_progress("copying database files...");
        let container = self.container_name();
        copy_from_container(&container, "/data/databases", build_path).await?;
        copy_from_container(&container, "/data/transactions", build_path).await?;

        std::fs::write(build_path.join("Dockerfile"), DOCKERFILE_TEMPLATE)?;

        progress.show_progress("building image...");
        let mut build_cmd = container_command()?;
        build_cmd
            .arg("build")
            .arg("-t")
            .arg(self.image_tag())
            .arg(build_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let output = build_cmd.output().await?;
        if !output.status.success() {
            bail!(
                "Image build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }
}

async fn copy_from_container(
    container: &str,
    src: &str,
    dest: &std::path::Path,
) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("cp")
        .arg(format!("{}:{}", container, src))
        .arg(dest)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to copy {} from container: {}",
            src,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
