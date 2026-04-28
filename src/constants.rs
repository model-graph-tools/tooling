//! Analyzer version and download URL.

pub static ANALYZER_VERSION: &str = "0.1.1";

/// Returns the GitHub release download URL for the current analyzer version.
pub fn analyzer_url() -> String {
    format!(
        "https://github.com/model-graph-tools/analyzer/releases/download/v{v}/analyzer-{v}.jar",
        v = ANALYZER_VERSION
    )
}

/// Neo4J Docker image version tag used for the base image.
pub static NEO4J_VERSION: &str = "5.26.12-community";

/// Neo4J Docker image repository.
pub static NEO4J_IMAGE: &str = "docker.io/neo4j";

/// Returns a Dockerfile for building a Neo4J image with pre-populated databases.
pub fn neo4j_model_db_dockerfile() -> String {
    format!(
        r#"FROM neo4j:{NEO4J_VERSION}
COPY --chown=neo4j:neo4j databases /data/databases
COPY --chown=neo4j:neo4j transactions /data/transactions
ENV NEO4J_AUTH=none
ENV NEO4J_server_databases_default__to__read__only=true
ENV NEO4J_browser_post__connect__cmd="play https://model-graph-tools.github.io/assets/welcome.html"
ENV NEO4J_browser_remote__content__hostname__whitelist="model-graph-tools.github.io"
"#
    )
}
