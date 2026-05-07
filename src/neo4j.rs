//! Neo4J container, image, and port management.

use crate::constants::{
    MODEL_GRAPH_TOOLS_REPOSITORY, NEO4J_IMAGE, NEO4J_VERSION, PLATFORMS, SCHEMA_SVG_URL,
    WELCOME_URL,
};
use crate::container::{container_command, run_container_cmd};
use crate::progress::Progress;
use std::process::Stdio;
use wildfly_meta::MetaItem;

// ------------------------------------------------------ ports

/// Bolt and HTTP ports for a Neo4J container, derived from the source's port offset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ports {
    pub bolt: u16,
    pub http: u16,
}

impl Ports {
    /// Computes default bolt (6000+offset) and http (7000+offset) ports.
    pub fn default_ports(item: &MetaItem) -> anyhow::Result<Ports> {
        let offset = item.port_offset();
        Ok(Ports {
            bolt: 6000u16
                .checked_add(offset)
                .ok_or_else(|| anyhow::anyhow!("Port offset {offset} too large for bolt port"))?,
            http: 7000u16
                .checked_add(offset)
                .ok_or_else(|| anyhow::anyhow!("Port offset {offset} too large for http port"))?,
        })
    }
}

// ------------------------------------------------------ image

/// A Neo4J image configuration tied to an analysis source.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Neo4JImage {
    pub item: MetaItem,
}

impl Neo4JImage {
    /// Creates an image configuration from the given meta item.
    #[must_use]
    pub fn new(item: &MetaItem) -> Neo4JImage {
        Neo4JImage { item: item.clone() }
    }

    /// Returns the upstream Neo4J base image reference (e.g. `docker.io/neo4j:5.26.12-community`).
    pub fn base_image_name() -> String {
        format!("{}:{}", NEO4J_IMAGE, NEO4J_VERSION)
    }

    /// Returns the tagged image name on quay.io for this source.
    pub fn image_tag(&self) -> String {
        match &self.item {
            MetaItem::Image(img) => {
                format!("{}:{}", MODEL_GRAPH_TOOLS_REPOSITORY, img.version)
            }
            MetaItem::FeaturePack(fp) => {
                format!(
                    "{}:{}-{}",
                    MODEL_GRAPH_TOOLS_REPOSITORY, fp.shortcut, fp.version
                )
            }
        }
    }

    /// Copies database files from the running container and builds a multi-arch manifest image.
    pub async fn build_image(
        &self,
        container_name: &str,
        progress: &Progress,
    ) -> anyhow::Result<()> {
        let build_dir = tempfile::tempdir()?;
        let build_path = build_dir.path();

        progress.show_progress("Copying database files...");
        copy_from_container(container_name, "/data/databases", build_path).await?;
        copy_from_container(container_name, "/data/transactions", build_path).await?;

        std::fs::write(
            build_path.join("Dockerfile"),
            model_db_dockerfile(&self.item.full_name()),
        )?;

        let image_tag = self.image_tag();

        // Remove any existing manifest (ignore errors if it doesn't exist)
        progress.show_progress("Creating manifest...");
        let mut rm_cmd = container_command()?;
        rm_cmd
            .arg("manifest")
            .arg("rm")
            .arg(&image_tag)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let _ = rm_cmd.output().await;

        run_container_cmd(
            &["manifest", "create", &image_tag],
            "Manifest creation failed",
        )
        .await?;

        progress.show_progress("Building image...");
        let build_path_str = build_path.to_string_lossy();
        run_container_cmd(
            &[
                "build",
                "--platform",
                PLATFORMS,
                "--manifest",
                &image_tag,
                &build_path_str,
            ],
            "Image build failed",
        )
        .await
    }
}

// ------------------------------------------------------ container

/// A Neo4J container with its image and assigned ports.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Neo4JContainer {
    pub image: Neo4JImage,
    pub ports: Ports,
}

impl Neo4JContainer {
    /// Creates a container with default ports derived from the image's source.
    pub fn new(image: Neo4JImage) -> anyhow::Result<Neo4JContainer> {
        let ports = Ports::default_ports(&image.item)?;
        Ok(Neo4JContainer { image, ports })
    }

    /// Returns the container name (e.g. `mgt-neo4j-340`).
    pub fn container_name(&self) -> String {
        format!("mgt-neo4j-{}", self.image.item.container_name())
    }

    /// Returns the data volume name (e.g. `mgt-neo4j-data-340`).
    pub fn volume_name(&self) -> String {
        format!("mgt-neo4j-data-{}", self.image.item.container_name())
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

/// Returns a Dockerfile for building a Neo4J image with pre-populated databases
/// and an nginx reverse proxy that serves the welcome page from the same origin.
fn model_db_dockerfile(source_name: &str) -> String {
    format!(
        r#"FROM neo4j:{NEO4J_VERSION}

USER root
RUN apt-get update && apt-get install -y --no-install-recommends nginx curl \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /var/www/html /var/lib/nginx/body /var/lib/nginx/proxy \
        /var/lib/nginx/fastcgi /var/lib/nginx/uwsgi /var/lib/nginx/scgi \
        /var/log/nginx /run \
    && chown -R neo4j:neo4j /var/www/html /var/lib/nginx /var/log/nginx /run \
    && curl -fsSL -o /var/www/html/welcome.html \
       {WELCOME_URL} \
    && curl -fsSL -o /var/www/html/schema.svg \
       {SCHEMA_SVG_URL} \
    && sed -i 's|{{{{SOURCE_NAME}}}}|{source_name}|g' /var/www/html/welcome.html

RUN printf 'pid /run/nginx.pid;\n\
error_log /var/log/nginx/error.log;\n\
events {{\n\
    worker_connections 64;\n\
}}\n\
http {{\n\
    include /etc/nginx/mime.types;\n\
    access_log /var/log/nginx/access.log;\n\
    client_body_temp_path /var/lib/nginx/body;\n\
    proxy_temp_path /var/lib/nginx/proxy;\n\
    server {{\n\
        listen 7474;\n\
        location = /welcome.html {{\n\
            root /var/www/html;\n\
        }}\n\
        location = /schema.svg {{\n\
            root /var/www/html;\n\
        }}\n\
        location / {{\n\
            proxy_pass http://localhost:7475;\n\
            proxy_http_version 1.1;\n\
            proxy_set_header Upgrade $http_upgrade;\n\
            proxy_set_header Connection "upgrade";\n\
            proxy_set_header Host $host;\n\
        }}\n\
    }}\n\
}}\n' > /etc/nginx/nginx.conf

RUN printf '#!/bin/bash\nnginx -c /etc/nginx/nginx.conf\nexec /startup/docker-entrypoint.sh neo4j\n' \
    > /entrypoint.sh && chmod +x /entrypoint.sh

USER neo4j
COPY --chown=neo4j:neo4j databases /data/databases
COPY --chown=neo4j:neo4j transactions /data/transactions
ENV NEO4J_AUTH=none
ENV NEO4J_server_databases_default__to__read__only=true
ENV NEO4J_server_http_listen__address=:7475
ENTRYPOINT ["/entrypoint.sh"]
"#
    )
}
