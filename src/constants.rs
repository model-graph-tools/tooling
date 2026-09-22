//! Analyzer version and download URL.

/// Container image registry and organization prefix.
pub static MODEL_GRAPH_TOOLS_REPOSITORY: &str = "quay.io/modelgraphtools/model";

/// Wado standalone container image repository used for WildFly analysis.
pub static WADO_SA_REPOSITORY: &str = "quay.io/wado/wado-sa";

/// A static string that defines the current version of the analyzer.
pub static ANALYZER_VERSION: &str = "0.1.3";

/// Returns the GitHub release download URL for the current analyzer version.
pub fn analyzer_url() -> String {
    format!(
        "https://github.com/model-graph-tools/analyzer/releases/download/v{v}/analyzer-{v}.jar",
        v = ANALYZER_VERSION
    )
}

/// Resolves the latest REST API release version from GitHub.
///
/// Queries the GitHub releases API and strips the leading `v` from the tag name.
/// Falls back to `REST_API_VERSION` if the API request fails.
pub async fn latest_rest_api_version() -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("mgt")
        .build()?;
    let response = client
        .get("https://api.github.com/repos/model-graph-tools/rest-api/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch latest REST API version: HTTP {}",
            response.status()
        );
    }
    let body: serde_json::Value = response.json().await?;
    let tag = body["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No tag_name in GitHub release response"))?;
    Ok(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

/// Current version of the REST API native binary.
pub static REST_API_VERSION: &str = "0.1.0";

/// Neo4J Docker image version tag used for the base image.
pub static NEO4J_VERSION: &str = "2026.04-community";

/// Neo4J Docker image repository.
pub static NEO4J_IMAGE: &str = "docker.io/neo4j";

/// Target platforms for multi-arch image builds.
pub static PLATFORMS: &str = "linux/amd64,linux/arm64";

/// URL for the welcome page shown in the Neo4J browser after connecting.
pub static WELCOME_URL: &str = "https://model-graph-tools.github.io/assets/welcome.html";

/// URL for the schema SVG graphic referenced by the welcome page.
pub static SCHEMA_SVG_URL: &str = "https://model-graph-tools.github.io/assets/schema.svg";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzer_url_formats_correctly() {
        let url = analyzer_url();
        assert!(url.contains(ANALYZER_VERSION));
        assert!(url.ends_with(".jar"));
    }
}
