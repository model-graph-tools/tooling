//! Shared cleanup and Neo4J image build logic used by both analysis pipelines.

use crate::container::{remove_container, remove_network, remove_volume, stop_container};
use crate::neo4j::Neo4JContainer;
use crate::progress::{CommandStatus, Progress, step_header, summary};
use indicatif::MultiProgress;
use tokio::task::JoinSet;
use wado::StandaloneInstance;

/// Total number of pipeline steps shown in step headers.
const TOTAL_STEPS: u32 = 4;

/// Stops the Neo4J container and commits its data volume into a tagged image.
pub(super) async fn build_neo4j_image(neo4j: &Neo4JContainer) -> anyhow::Result<()> {
    step_header(3, TOTAL_STEPS, "Building Neo4J image...");
    let progress = Progress::new(&neo4j.image.image_tag());

    progress.show_progress("stopping neo4j...");
    stop_container(&neo4j.container_name()).await?;

    neo4j
        .image
        .build_image(&neo4j.container_name(), &progress)
        .await?;

    progress.finish_success(Some("ready"));
    Ok(())
}

/// Full cleanup for WildFly analysis: stops WildFly instances, removes Neo4J
/// container and volume, and tears down the network.
pub(super) async fn cleanup(
    instances: &[StandaloneInstance],
    neo4j: &Neo4JContainer,
    network: &str,
) -> anyhow::Result<()> {
    step_header(4, TOTAL_STEPS, "Cleaning up...");
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();
    let mut status: Vec<CommandStatus> = Vec::new();

    for instance in instances {
        let container_name = instance.name.clone();
        let progress = Progress::join(&multi_progress, &container_name);
        tasks.spawn(async move {
            match stop_container(&container_name).await {
                Ok(()) => {
                    progress.finish_success(Some("stopped"));
                    CommandStatus::success(&container_name)
                }
                Err(e) => {
                    let msg = e.to_string();
                    progress.finish_error(&msg);
                    CommandStatus::error(&container_name, &msg)
                }
            }
        });
    }
    status.extend(tasks.join_all().await);

    status.extend(cleanup_neo4j_and_network(neo4j, network).await);

    let count = status.len();
    summary(count, &status);
    Ok(())
}

/// Cleanup for feature pack analysis: removes Neo4J container and volume,
/// and tears down the network (no WildFly instances to stop).
pub(super) async fn cleanup_minimal(neo4j: &Neo4JContainer, network: &str) -> Vec<CommandStatus> {
    step_header(4, TOTAL_STEPS, "Cleaning up...");
    let status = cleanup_neo4j_and_network(neo4j, network).await;
    let count = status.len();
    summary(count, &status);
    status
}

/// Removes the Neo4J container, its data volume, and the container network.
async fn cleanup_neo4j_and_network(neo4j: &Neo4JContainer, network: &str) -> Vec<CommandStatus> {
    let neo4j_container = neo4j.container_name();
    let _ = stop_container(&neo4j_container).await;
    vec![
        cleanup_resource(&neo4j_container, remove_container(&neo4j_container).await),
        cleanup_resource(
            &neo4j.volume_name(),
            remove_volume(&neo4j.volume_name()).await,
        ),
        cleanup_resource(network, remove_network(network).await),
    ]
}

/// Reports the outcome of a single resource cleanup operation via the progress bar.
fn cleanup_resource(name: &str, result: anyhow::Result<()>) -> CommandStatus {
    let progress = Progress::new(name);
    match result {
        Ok(()) => {
            progress.finish_success(Some("removed"));
            CommandStatus::success(name)
        }
        Err(e) => {
            let msg = e.to_string();
            progress.finish_error(&msg);
            CommandStatus::error(name, &msg)
        }
    }
}
