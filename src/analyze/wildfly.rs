//! WildFly-specific analysis pipeline.
//!
//! Starts one or more WildFly containers with different server configurations,
//! runs the analyzer against each, and stores the combined results in Neo4J.

use super::cleanup::{build_neo4j_image, cleanup};
use super::neo4j_ops::start_neo4j;
use super::runner::{download_analyzer, run_analyzer};
use crate::constants::analyzer_url;
use crate::container::{container_command, create_network, healthcheck, network_name};
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::progress::{Progress, step_header};
use crate::source::Source;
use anyhow::{anyhow, bail};
use console::style;
use indicatif::MultiProgress;
use std::env::temp_dir;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::task::JoinSet;
use wado::{AdminContainer, Ports, ServerType, StandaloneInstance};
use wildfly_container_versions::WildFlyContainer;

const TOTAL_STEPS: u32 = 4;

/// A WildFly server configuration to analyze.
struct WildFlyConfiguration {
    config: &'static str,
    suffix: &'static str,
    /// Whether to append results to an existing Neo4J database or clean first.
    append: bool,
}

/// Returns the server configurations to analyze for a given WildFly version.
///
/// Versions 28+ include both `standalone-full-ha.xml` and
/// `standalone-microprofile.xml`; older versions only use `standalone-full-ha.xml`.
fn wildfly_configurations(wildfly_container: &WildFlyContainer) -> Vec<WildFlyConfiguration> {
    let major = wildfly_container.version.major;
    let mut configs = vec![WildFlyConfiguration {
        config: "standalone-full-ha.xml",
        suffix: "fha",
        append: false,
    }];
    if major >= 28 {
        configs.push(WildFlyConfiguration {
            config: "standalone-microprofile.xml",
            suffix: "mp",
            append: true,
        });
    }
    configs
}

/// Orchestrates the full WildFly analysis pipeline: prepare environment,
/// run analyzers, build Neo4J image, and clean up.
pub(super) async fn run_wildfly_analysis(
    wildfly_container: &WildFlyContainer,
    source: &Source,
) -> anyhow::Result<()> {
    let configs = wildfly_configurations(wildfly_container);
    let admin_container = AdminContainer::new(wildfly_container.clone(), ServerType::Standalone);
    let neo4j_image = Neo4JImage::new(source);
    let neo4j = Neo4JContainer::new(neo4j_image);
    let network = network_name(source);

    let instances: Vec<StandaloneInstance> = configs
        .iter()
        .enumerate()
        .map(|(i, cfg)| {
            let default_ports = Ports::default_ports(wildfly_container);
            StandaloneInstance::new(
                admin_container.clone(),
                format!(
                    "mgt-wado-sa-{}-{}",
                    wildfly_container.identifier, cfg.suffix
                ),
                Ports {
                    http: default_ports.http + i as u16,
                    management: default_ports.management + i as u16,
                },
            )
        })
        .collect();

    create_network(&network).await?;
    prepare_environment(&instances, &configs, &neo4j, &network).await?;

    let result = async {
        run_analyzers(&instances, &configs, &neo4j, &network).await?;
        build_neo4j_image(&neo4j).await
    }
    .await;

    if let Err(ref e) = result {
        eprintln!("\n{}: {}", style("Error").red().bold(), e);
    }
    cleanup(&instances, &neo4j, &network).await?;

    result
}

/// Prepares the environment by downloading the analyzer JAR, starting WildFly
/// instances, and starting the Neo4J container — all in parallel.
async fn prepare_environment(
    instances: &[StandaloneInstance],
    configs: &[WildFlyConfiguration],
    neo4j: &Neo4JContainer,
    network: &str,
) -> anyhow::Result<PathBuf> {
    step_header(1, TOTAL_STEPS, "Preparing environment...");
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    let dl_progress = Progress::join(&multi_progress, "analyzer");
    let url = analyzer_url();
    tasks.spawn(async move {
        let result = download_analyzer(&url, &dl_progress).await;
        match &result {
            Ok(_) => dl_progress.finish_success(Some("ready")),
            Err(e) => dl_progress.finish_error(&e.to_string()),
        }
        result.map(PrepareResult::Analyzer)
    });

    for (instance, cfg) in instances.iter().zip(configs.iter()) {
        let instance = instance.clone();
        let config = cfg.config.to_string();
        let network = network.to_string();
        let progress = Progress::join(&multi_progress, &config);
        tasks.spawn(async move {
            let result = start_wildfly(&instance, &config, &network, &progress).await;
            match &result {
                Ok(()) => progress.finish_success(Some("ready")),
                Err(e) => progress.finish_error(&e.to_string()),
            }
            result.map(|()| PrepareResult::WildFly)
        });
    }

    let neo4j_clone = neo4j.clone();
    let network_clone = network.to_string();
    let neo4j_progress = Progress::join(&multi_progress, "neo4j");
    tasks.spawn(async move {
        let result = start_neo4j(&neo4j_clone, &network_clone, &neo4j_progress).await;
        match &result {
            Ok(()) => neo4j_progress.finish_success(Some("ready")),
            Err(e) => neo4j_progress.finish_error(&e.to_string()),
        }
        result.map(|()| PrepareResult::Neo4J)
    });

    let results = tasks.join_all().await;
    let mut analyzer_jar: Option<PathBuf> = None;
    for result in results {
        match result? {
            PrepareResult::Analyzer(path) => analyzer_jar = Some(path),
            PrepareResult::WildFly | PrepareResult::Neo4J => {}
        }
    }

    analyzer_jar.ok_or_else(|| anyhow!("Analyzer download task did not produce a result"))
}

enum PrepareResult {
    Analyzer(PathBuf),
    WildFly,
    Neo4J,
}

/// Runs the analyzer sequentially for each WildFly configuration.
///
/// The first configuration cleans the Neo4J database; subsequent
/// configurations append to it.
async fn run_analyzers(
    instances: &[StandaloneInstance],
    configs: &[WildFlyConfiguration],
    neo4j: &Neo4JContainer,
    network: &str,
) -> anyhow::Result<()> {
    step_header(2, TOTAL_STEPS, "Analyzing...");
    let analyzer_jar = temp_dir().join("analyzer.jar");
    for (instance, cfg) in instances.iter().zip(configs.iter()) {
        let progress = Progress::new(cfg.config);
        let mode = if cfg.append { "--append" } else { "--clean" };
        let result = run_analyzer(&analyzer_jar, instance, neo4j, network, mode, &progress).await;
        match &result {
            Ok(()) => progress.finish_success(Some("done")),
            Err(e) => {
                progress.finish_error(&e.to_string());
                return result;
            }
        }
    }
    Ok(())
}

/// Starts a WildFly container and waits for it to become healthy.
async fn start_wildfly(
    instance: &StandaloneInstance,
    configuration: &str,
    network: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    progress.show_progress("starting container...");
    let mut command = container_command()?;
    command
        .arg("run")
        .arg("--rm")
        .arg("--detach")
        .arg("--name")
        .arg(&instance.name)
        .arg("--network")
        .arg(network)
        .arg("--publish")
        .arg(format!("{}:8080", instance.ports.http))
        .arg("--publish")
        .arg(format!("{}:9990", instance.ports.management))
        .arg(instance.admin_container.image_name())
        .args(["-c", configuration])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = command.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to start WildFly: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    progress.show_progress("waiting for WildFly...");
    healthcheck(
        &format!("http://localhost:{}", instance.ports.management),
        progress,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configs_old_version() {
        let wc = WildFlyContainer::version("10").unwrap();
        let configs = wildfly_configurations(&wc);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].config, "standalone-full-ha.xml");
        assert!(!configs[0].append);
    }

    #[test]
    fn configs_new_version() {
        let wc = WildFlyContainer::version("39").unwrap();
        let configs = wildfly_configurations(&wc);
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].config, "standalone-full-ha.xml");
        assert!(!configs[0].append);
        assert_eq!(configs[1].config, "standalone-microprofile.xml");
        assert!(configs[1].append);
    }

    #[test]
    fn configs_boundary_version() {
        let wc = WildFlyContainer::version("28").unwrap();
        let configs = wildfly_configurations(&wc);
        assert_eq!(configs.len(), 2);
    }

    #[test]
    fn configs_below_boundary() {
        let wc = WildFlyContainer::version("27").unwrap();
        let configs = wildfly_configurations(&wc);
        assert_eq!(configs.len(), 1);
    }
}
