//! Neo4J container, image, and port management.

use crate::container::run_container_cmd;
use crate::progress::Progress;
use crate::source::Source;

/// Neo4J Docker image version tag used for the base image.
pub static NEO4J_VERSION: &str = "5.26.12-community";

/// Neo4J Docker image repository.
pub static NEO4J_IMAGE: &str = "docker.io/neo4j";

/// Dockerfile template for building a Neo4J image with pre-populated databases.
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

/// Bolt and HTTP ports for a Neo4J container, derived from the source's port offset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ports {
    pub bolt: u16,
    pub http: u16,
}

impl Ports {
    /// Computes default bolt (6000+offset) and http (7000+offset) ports.
    pub fn default_ports(source: &Source) -> Ports {
        let offset = source.port_offset();
        Ports {
            bolt: 6000 + offset,
            http: 7000 + offset,
        }
    }
}

// ------------------------------------------------------ image

/// A Neo4J image configuration tied to an analysis source.
#[derive(Clone, Eq, PartialEq)]
pub struct Neo4JImage {
    pub source: Source,
}

impl Neo4JImage {
    /// Creates an image configuration from the given source.
    pub fn new(source: &Source) -> Neo4JImage {
        Neo4JImage {
            source: source.clone(),
        }
    }

    /// Returns the upstream Neo4J base image reference (e.g. `docker.io/neo4j:5.26.12-community`).
    pub fn base_image_name() -> String {
        format!("{}:{}", NEO4J_IMAGE, NEO4J_VERSION)
    }

    /// Returns the tagged image name on quay.io for this source.
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

    /// Copies database files from the running container and builds a tagged image.
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
        let image_tag = self.image_tag();
        let build_path_str = build_path.to_string_lossy();
        run_container_cmd(
            &["build", "-t", &image_tag, &build_path_str],
            "Image build failed",
        )
        .await
    }
}

// ------------------------------------------------------ container

/// A Neo4J container with its image and assigned ports.
#[derive(Clone, Eq, PartialEq)]
pub struct Neo4JContainer {
    pub image: Neo4JImage,
    pub ports: Ports,
}

impl Neo4JContainer {
    /// Creates a container with default ports derived from the image's source.
    pub fn new(image: Neo4JImage) -> Neo4JContainer {
        let ports = Ports::default_ports(&image.source);
        Neo4JContainer { image, ports }
    }

    /// Returns the container name (e.g. `mgt-neo4j-340`).
    pub fn container_name(&self) -> String {
        format!("mgt-neo4j-{}", self.image.source.container_id())
    }

    /// Returns the data volume name (e.g. `mgt-neo4j-data-340`).
    pub fn volume_name(&self) -> String {
        format!("mgt-neo4j-data-{}", self.image.source.container_id())
    }
}

// ------------------------------------------------------ running container

/// A running Neo4J container with its runtime ID and status.
pub struct RunningNeo4JContainer {
    pub container: Neo4JContainer,
    pub id: String,
    pub status: String,
}

// ------------------------------------------------------ helper

/// Copies a directory from a running container to a local destination path.
async fn copy_from_container(
    container: &str,
    src: &str,
    dest: &std::path::Path,
) -> anyhow::Result<()> {
    let src_arg = format!("{container}:{src}");
    let dest_str = dest.to_string_lossy();
    run_container_cmd(
        &["cp", &src_arg, &dest_str],
        &format!("Failed to copy {src} from container"),
    )
    .await
}
