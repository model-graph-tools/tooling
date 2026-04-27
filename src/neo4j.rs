use crate::container::container_command;
use crate::progress::Progress;
use crate::source::Source;
use anyhow::bail;
use std::process::Stdio;

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

// ------------------------------------------------------ ports

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ports {
    pub bolt: u16,
    pub http: u16,
}

impl Ports {
    pub fn default_ports(source: &Source) -> Ports {
        let offset = source.port_offset();
        Ports {
            bolt: 6000 + offset,
            http: 7000 + offset,
        }
    }
}

// ------------------------------------------------------ image

#[derive(Clone, Eq, PartialEq)]
pub struct Neo4JImage {
    pub source: Source,
}

impl Neo4JImage {
    pub fn new(source: &Source) -> Neo4JImage {
        Neo4JImage {
            source: source.clone(),
        }
    }

    pub fn base_image_name() -> String {
        format!("{}:{}", NEO4J_IMAGE, NEO4J_VERSION)
    }

    pub fn image_tag(&self) -> String {
        match &self.source {
            Source::WildFly(wc) => {
                format!(
                    "quay.io/modelgraphtools/wildfly-management-model:{}",
                    wc.version
                )
            }
            Source::FeaturePack(fp) => {
                format!(
                    "quay.io/modelgraphtools/wildfly-management-model:{}-{}",
                    fp.shortcut, fp.version
                )
            }
        }
    }

    pub async fn build_image(
        &self,
        container_name: &str,
        progress: &Progress,
    ) -> anyhow::Result<()> {
        let build_dir = tempfile::tempdir()?;
        let build_path = build_dir.path();

        progress.show_progress("copying database files...");
        copy_from_container(container_name, "/data/databases", build_path).await?;
        copy_from_container(container_name, "/data/transactions", build_path).await?;

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

// ------------------------------------------------------ container

#[derive(Clone, Eq, PartialEq)]
pub struct Neo4JContainer {
    pub image: Neo4JImage,
    pub ports: Ports,
}

impl Neo4JContainer {
    pub fn new(image: Neo4JImage) -> Neo4JContainer {
        let ports = Ports::default_ports(&image.source);
        Neo4JContainer { image, ports }
    }

    pub fn container_name(&self) -> String {
        format!("mgt-neo4j-{}", self.image.source.container_id())
    }

    pub fn volume_name(&self) -> String {
        format!("mgt-neo4j-data-{}", self.image.source.container_id())
    }
}

// ------------------------------------------------------ running container

pub struct RunningNeo4JContainer {
    pub container: Neo4JContainer,
    pub id: String,
    pub status: String,
}

// ------------------------------------------------------ helper

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
