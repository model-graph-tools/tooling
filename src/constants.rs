pub static ANALYZER_VERSION: &str = "0.1.1";

pub fn analyzer_url() -> String {
    format!(
        "https://github.com/model-graph-tools/analyzer/releases/download/v{v}/analyzer-{v}.jar",
        v = ANALYZER_VERSION
    )
}
