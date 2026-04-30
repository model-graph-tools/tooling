//! Feature pack analysis pipeline.
//!
//! Downloads a doc-zip archive from the feature pack's Maven coordinates and
//! feeds it to the analyzer, bypassing the need for a running WildFly instance.

use super::cleanup::{build_neo4j_image, cleanup_minimal};
use super::neo4j_ops::start_neo4j;
use super::runner::run_doc_zip_analyzer;
use crate::container::{create_network, network_name, pull_image};
use crate::download::download_file;
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::progress::{Progress, step_header};
use anyhow::anyhow;
use console::style;
use indicatif::MultiProgress;
use std::path::PathBuf;
use tokio::task::JoinSet;
use wildfly_meta::{FeaturePack, MetaItem};

/// Total number of pipeline steps shown in step headers.
const TOTAL_STEPS: u32 = 4;

/// Orchestrates the feature pack analysis pipeline: download doc-zip, start
/// Neo4J, run analyzer, build image, and clean up.
pub(super) async fn run_feature_pack_analysis(
    fp: &FeaturePack,
    item: &MetaItem,
) -> anyhow::Result<()> {
    let neo4j_image = Neo4JImage::new(item);
    let neo4j = Neo4JContainer::new(neo4j_image);
    let network = network_name(item);

    create_network(&network).await?;

    let result = async {
        step_header(1, TOTAL_STEPS, "Preparing environment...");
        let multi_progress = MultiProgress::new();
        let mut tasks = JoinSet::new();

        let analyzer_image = super::runner::ANALYZER_IMAGE.to_string();
        let analyzer_img_progress = Progress::join(&multi_progress, "Analyzer image");
        tasks.spawn(async move {
            let result = pull_image(&analyzer_image, &analyzer_img_progress).await;
            match &result {
                Ok(()) => analyzer_img_progress.finish_success(Some("Ready")),
                Err(e) => analyzer_img_progress.finish_error(&e.to_string()),
            }
            result.map(|()| PathBuf::new())
        });

        let dl_progress = Progress::join(&multi_progress, &format!("Download {}", fp.short_name()));
        let url = fp.download_url();
        tasks.spawn(async move {
            let result = download_doc_zip(&url, &dl_progress).await;
            match &result {
                Ok(_) => dl_progress.finish_success(Some("Ready")),
                Err(e) => dl_progress.finish_error(&e.to_string()),
            }
            result
        });

        let neo4j_clone = neo4j.clone();
        let network_clone = network.clone();
        let neo4j_progress = Progress::join(&multi_progress, "Neo4J");
        tasks.spawn(async move {
            let result = start_neo4j(&neo4j_clone, &network_clone, &neo4j_progress).await;
            match &result {
                Ok(()) => neo4j_progress.finish_success(Some("Ready")),
                Err(e) => neo4j_progress.finish_error(&e.to_string()),
            }
            result.map(|()| PathBuf::new())
        });

        let results = tasks.join_all().await;
        let mut doc_zip_path: Option<PathBuf> = None;
        for result in results {
            let path = result?;
            if !path.as_os_str().is_empty() {
                doc_zip_path = Some(path);
            }
        }
        let doc_zip_path =
            doc_zip_path.ok_or_else(|| anyhow!("Doc-zip download did not produce a result"))?;

        step_header(2, TOTAL_STEPS, "Analyzing...");
        let progress = Progress::new(&fp.short_name());
        let result = run_doc_zip_analyzer(&doc_zip_path, &neo4j, &network, &progress).await;
        match &result {
            Ok(()) => progress.finish_success(Some("Done")),
            Err(e) => progress.finish_error(&e.to_string()),
        }
        result?;

        build_neo4j_image(&neo4j).await
    }
    .await;

    if let Err(ref e) = result {
        eprintln!("\n{}: {}", style("Error").red().bold(), e);
    }

    cleanup_minimal(&neo4j, &network).await;

    result
}

/// Downloads a doc-zip archive to a temporary directory.
///
/// Skips the download if the file already exists locally.
async fn download_doc_zip(url: &str, progress: &Progress) -> anyhow::Result<PathBuf> {
    let filename = url.rsplit('/').next().unwrap_or("doc.zip");
    download_file(url, filename, progress).await
}
