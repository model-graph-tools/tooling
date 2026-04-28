//! Analyzer version and download URL.

/// Container image registry and organization prefix.
pub static MODEL_GRAPH_TOOLS_REPOSITORY: &str = "quay.io/modelgraphtools/model";

/// A static string that defines the current version of the analyzer.
pub static ANALYZER_VERSION: &str = "0.1.2";

/// Returns the GitHub release download URL for the current analyzer version.
pub fn analyzer_url() -> String {
    format!(
        "https://github.com/model-graph-tools/analyzer/releases/download/v{v}/analyzer-{v}.jar",
        v = ANALYZER_VERSION
    )
}

/// Neo4J Docker image version tag used for the base image.
pub static NEO4J_VERSION: &str = "5.26-community";

/// Neo4J Docker image repository.
pub static NEO4J_IMAGE: &str = "docker.io/neo4j";

/// Target platforms for multi-arch image builds.
pub static PLATFORMS: &str = "linux/amd64,linux/arm64";

/// URL for the welcome page shown in the Neo4J browser after connecting.
pub static WELCOME_URL: &str = "https://model-graph-tools.github.io/assets/welcome.html";

/// URL for the schema SVG graphic referenced by the welcome page.
pub static SCHEMA_SVG_URL: &str = "https://model-graph-tools.github.io/assets/schema.svg";
