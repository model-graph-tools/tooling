//! Analyzer version and download URL.

pub static ANALYZER_VERSION: &str = "0.1.1";

/// Returns the GitHub release download URL for the current analyzer version.
pub fn analyzer_url() -> String {
    format!(
        "https://github.com/model-graph-tools/analyzer/releases/download/v{v}/analyzer-{v}.jar",
        v = ANALYZER_VERSION
    )
}
