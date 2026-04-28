//! Neo4J container, image, and port management.

use crate::constants::{neo4j_model_db_dockerfile, NEO4J_IMAGE, NEO4J_VERSION};
use crate::container::run_container_cmd;
use crate::progress::Progress;
use crate::source::Source;

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

        std::fs::write(build_path.join("Dockerfile"), neo4j_model_db_dockerfile())?;

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
