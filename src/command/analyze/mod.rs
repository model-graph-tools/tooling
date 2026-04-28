//! Analysis pipeline for WildFly management models.
//!
//! Orchestrates the end-to-end process of starting containers, running the
//! analyzer, building Neo4J images, and cleaning up resources. Supports two
//! analysis modes:
//!
//! - **WildFly**: Starts WildFly containers with different configurations,
//!   connects the analyzer via the management interface, and stores results
//!   in Neo4J.
//! - **Feature pack**: Downloads a doc-zip archive and feeds it directly to
//!   the analyzer without a running WildFly instance.

mod cleanup;
mod feature_pack;
mod neo4j_ops;
mod runner;
mod wildfly;

use crate::progress::done;
use crate::source::Source;
use console::style;
use tokio::time::Instant;

/// Entry point for the analysis pipeline.
///
/// Dispatches to the WildFly or feature pack analysis path based on the
/// source type and prints timing information on completion.
pub async fn analyze(source: &Source) -> anyhow::Result<()> {
    crate::container::verify_container_command()?;

    let instant = Instant::now();
    println!(
        "\n{}",
        style(format!("Analyzing {}", source.display_name())).bold()
    );

    match source {
        Source::WildFly(wc) => wildfly::run_wildfly_analysis(wc, source).await?,
        Source::FeaturePack(fp) => feature_pack::run_feature_pack_analysis(fp, source).await?,
    }

    done(instant);
    Ok(())
}
