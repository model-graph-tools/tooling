//! WildFly-specific analysis pipeline.
//!
//! Starts one or more WildFly containers with different server configurations,
//! runs the analyzer against each, and stores the combined results in Neo4J.

use super::cleanup::{build_neo4j_image, cleanup};
use super::neo4j_ops::start_neo4j;
use super::runner::{ANALYZER_IMAGE, download_analyzer, run_analyzer};
use crate::constants::{WADO_SA_REPOSITORY, analyzer_url};
use crate::container::{container_command, create_network, healthcheck, network_name, pull_image};
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::progress::{Progress, step_header};
use anyhow::{anyhow, bail};
use console::style;
use indicatif::MultiProgress;
use std::collections::HashMap;
use std::env::temp_dir;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::LazyLock;
use tokio::task::JoinSet;
use wildfly_meta::{MetaItem, WildFlyImage};

/// Total number of pipeline steps shown in step headers.
const TOTAL_STEPS: u32 = 4;

/// Per-version mapping of WildFly identifiers to their server configurations.
///
/// Keyed by `WildFlyImage.identifier` (e.g. `100` for 10.0, `261` for 26.1).
static CONFIGURATIONS: LazyLock<HashMap<u16, Vec<WildFlyConfiguration>>> = LazyLock::new(|| {
    let fha = || WildFlyConfiguration {
        config: "standalone-full-ha.xml",
        suffix: "fha",
        append: false,
    };
    let mp = || WildFlyConfiguration {
        config: "standalone-microprofile.xml",
        suffix: "mp",
        append: true,
    };

    let mut m = HashMap::new();
    // 10.0 - 18.0: standalone-full-ha.xml only
    for id in [100, 101, 110, 120, 130, 140, 150, 160, 170, 180] {
        m.insert(id, vec![fha()]);
    }
    // 19.0+ : standalone-full-ha.xml + standalone-microprofile.xml
    for id in [
        190, 191, 200, 210, 220, 230, 240, 250, 260, 261, 270, 280, 290, 300, 310, 320, 330, 340,
        350, 360, 370, 380, 390,
    ] {
        m.insert(id, vec![fha(), mp()]);
    }
    m
});

/// Default configurations for unknown/future WildFly versions.
static DEFAULT_CONFIGURATIONS: LazyLock<Vec<WildFlyConfiguration>> = LazyLock::new(|| {
    vec![
        WildFlyConfiguration {
            config: "standalone-full-ha.xml",
            suffix: "fha",
            append: false,
        },
        WildFlyConfiguration {
            config: "standalone-microprofile.xml",
            suffix: "mp",
            append: true,
        },
    ]
});

/// A WildFly server configuration to analyze.
struct WildFlyConfiguration {
    config: &'static str,
    suffix: &'static str,
    /// Whether to append results to an existing Neo4J database or clean first.
    append: bool,
}

/// A WildFly instance used during analysis, replacing wado types.
#[derive(Clone)]
pub(super) struct AnalysisInstance {
    pub image_ref: String,
    pub identifier: u16,
    pub name: String,
    pub http_port: u16,
    pub management_port: u16,
    pub supports_stability: bool,
}

/// Returns the server configurations to analyze for a given WildFly version.
///
/// Looks up the version in the explicit mapping. Unknown/future versions
/// fall back to both `standalone-full-ha.xml` and `standalone-microprofile.xml`.
fn wildfly_configurations(image: &WildFlyImage) -> &'static [WildFlyConfiguration] {
    CONFIGURATIONS
        .get(&image.identifier)
        .map(|v| v.as_slice())
        .unwrap_or(&DEFAULT_CONFIGURATIONS)
}

/// Orchestrates the full WildFly analysis pipeline: prepare environment,
/// run analyzers, build Neo4J image, and clean up.
pub(super) async fn run_wildfly_analysis(
    image: &WildFlyImage,
    item: &MetaItem,
) -> anyhow::Result<()> {
    let configs = wildfly_configurations(image);
    let neo4j_image = Neo4JImage::new(item);
    let neo4j = Neo4JContainer::new(neo4j_image)?;
    let network = network_name(item);

    let wado_image_ref = format!("{}:{}", WADO_SA_REPOSITORY, image.image_tag);
    let instances: Vec<AnalysisInstance> = configs
        .iter()
        .enumerate()
        .map(|(i, cfg)| AnalysisInstance {
            image_ref: wado_image_ref.clone(),
            identifier: image.identifier,
            name: format!("mgt-wado-sa-{}-{}", image.identifier, cfg.suffix),
            http_port: image.http_port() + i as u16,
            management_port: image.management_port() + i as u16,
            supports_stability: image.supports_stability(),
        })
        .collect();

    create_network(&network).await?;
    prepare_environment(&instances, configs, &neo4j, &network).await?;

    let result = async {
        run_analyzers(&instances, configs, &neo4j, &network).await?;
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
    instances: &[AnalysisInstance],
    configs: &[WildFlyConfiguration],
    neo4j: &Neo4JContainer,
    network: &str,
) -> anyhow::Result<PathBuf> {
    step_header(1, TOTAL_STEPS, "Preparing environment...");
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    let wf_image = instances[0].image_ref.clone();
    let wf_progress = Progress::join(&multi_progress, "Wildfly image");
    tasks.spawn(async move {
        let result = pull_image(&wf_image, &wf_progress).await;
        match &result {
            Ok(()) => wf_progress.finish_success(Some("Ready")),
            Err(e) => wf_progress.finish_error(&e.to_string()),
        }
        result.map(|()| PrepareResult::WildFly)
    });

    let analyzer_image = ANALYZER_IMAGE.to_string();
    let analyzer_img_progress = Progress::join(&multi_progress, "Analyzer image");
    tasks.spawn(async move {
        let result = pull_image(&analyzer_image, &analyzer_img_progress).await;
        match &result {
            Ok(()) => analyzer_img_progress.finish_success(Some("Ready")),
            Err(e) => analyzer_img_progress.finish_error(&e.to_string()),
        }
        result.map(|()| PrepareResult::WildFly)
    });

    let dl_progress = Progress::join(&multi_progress, "Analyzer");
    let url = analyzer_url();
    tasks.spawn(async move {
        let result = download_analyzer(&url, &dl_progress).await;
        match &result {
            Ok(_) => dl_progress.finish_success(Some("Ready")),
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
                Ok(()) => progress.finish_success(Some("Ready")),
                Err(e) => progress.finish_error(&e.to_string()),
            }
            result.map(|()| PrepareResult::WildFly)
        });
    }

    let neo4j_clone = neo4j.clone();
    let network_clone = network.to_string();
    let neo4j_progress = Progress::join(&multi_progress, "Neo4J");
    tasks.spawn(async move {
        let result = start_neo4j(&neo4j_clone, &network_clone, &neo4j_progress).await;
        match &result {
            Ok(()) => neo4j_progress.finish_success(Some("Ready")),
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

/// Discriminates results from the parallel environment preparation tasks.
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
    instances: &[AnalysisInstance],
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
            Ok(()) => progress.finish_success(Some("Done")),
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
    instance: &AnalysisInstance,
    configuration: &str,
    network: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    progress.show_progress("Starting container...");
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
        .arg(format!("{}:8080", instance.http_port))
        .arg("--publish")
        .arg(format!("{}:9990", instance.management_port))
        .arg(&instance.image_ref)
        .args(["-c", configuration]);
    if instance.supports_stability {
        command.arg("--stability=experimental");
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = command.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to start WildFly: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    progress.show_progress("Waiting for WildFly...");
    healthcheck(
        &format!("http://localhost:{}", instance.management_port),
        progress,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::init_registries_sync;
    use std::sync::Once;
    use wildfly_meta::parse_wildfly_image;

    static INIT: Once = Once::new();

    fn init() {
        INIT.call_once(|| {
            init_registries_sync().expect("Failed to initialize registries");
        });
    }

    #[test]
    fn configs_old_version() {
        init();
        let img = parse_wildfly_image("10", crate::registry::images_registry().unwrap()).unwrap();
        let configs = wildfly_configurations(&img);
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].config, "standalone-full-ha.xml");
        assert!(!configs[0].append);
    }

    #[test]
    fn configs_new_version() {
        init();
        let img = parse_wildfly_image("39", crate::registry::images_registry().unwrap()).unwrap();
        let configs = wildfly_configurations(&img);
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].config, "standalone-full-ha.xml");
        assert!(!configs[0].append);
        assert_eq!(configs[1].config, "standalone-microprofile.xml");
        assert!(configs[1].append);
    }

    #[test]
    fn configs_boundary_version() {
        init();
        let img = parse_wildfly_image("19", crate::registry::images_registry().unwrap()).unwrap();
        let configs = wildfly_configurations(&img);
        assert_eq!(configs.len(), 2);
    }

    #[test]
    fn configs_below_boundary() {
        init();
        let img = parse_wildfly_image("18", crate::registry::images_registry().unwrap()).unwrap();
        let configs = wildfly_configurations(&img);
        assert_eq!(configs.len(), 1);
    }

    #[test]
    fn configs_unmapped_identifier_uses_default() {
        assert!(!CONFIGURATIONS.contains_key(&999));
        assert_eq!(DEFAULT_CONFIGURATIONS.len(), 2);
        assert_eq!(DEFAULT_CONFIGURATIONS[0].config, "standalone-full-ha.xml");
        assert_eq!(
            DEFAULT_CONFIGURATIONS[1].config,
            "standalone-microprofile.xml"
        );
    }
}
