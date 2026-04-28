//! Analyzer JAR download, execution, and output processing.
//!
//! Handles downloading the analyzer from GitHub releases, running it as a
//! containerized Java process, and streaming its stdout/stderr to log files
//! while tracking progress.

use crate::container::container_command;
use crate::progress::Progress;
use anyhow::{anyhow, bail};
use console::style;
use std::collections::VecDeque;
use std::env::temp_dir;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;

use crate::neo4j::Neo4JContainer;
use wado::StandaloneInstance;

pub(super) const ANALYZER_IMAGE: &str = "eclipse-temurin:25-jre";
const ERROR_BUFFER_CAPACITY: usize = 20;

/// Downloads the analyzer JAR to a temporary directory.
///
/// Skips the download if the JAR already exists locally.
pub(super) async fn download_analyzer(url: &str, progress: &Progress) -> anyhow::Result<PathBuf> {
    let path = temp_dir().join("analyzer.jar");
    if path.exists() {
        return Ok(path);
    }

    progress.show_progress("downloading...");
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let mut file = File::create(&path)?;
        let content = response.bytes().await?;
        file.write_all(&content)?;
        Ok(path)
    } else {
        Err(anyhow!("Failed to download {}: {}", url, response.status()))
    }
}

/// Runs the analyzer against a live WildFly instance via the management interface.
///
/// The analyzer connects to WildFly over the container network and writes
/// results to Neo4J. The `mode` parameter controls whether the Neo4J database
/// is cleaned first (`--clean`) or appended to (`--append`).
pub(super) async fn run_analyzer(
    analyzer_jar: &Path,
    instance: &StandaloneInstance,
    neo4j: &Neo4JContainer,
    network: &str,
    mode: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    progress.show_progress("starting analyzer...");

    let suffix = if mode == "--clean" { "fha" } else { "mp" };
    let analyzer_container_name = format!(
        "mgt-analyzer-{}-{}",
        instance.admin_container.wildfly_container.identifier, suffix
    );
    let log_path = temp_dir().join(format!(
        "mgt-analyzer-{}-{}.log",
        instance.admin_container.wildfly_container.identifier, suffix
    ));
    let mut log_file = BufWriter::new(File::create(&log_path)?);
    let mut error_buffer: VecDeque<String> = VecDeque::with_capacity(ERROR_BUFFER_CAPACITY);

    let mut cmd = container_command()?;
    let mut child = cmd
        .arg("run")
        .arg("--rm")
        .arg("--name")
        .arg(&analyzer_container_name)
        .arg("--network")
        .arg(network)
        .arg("--volume")
        .arg(format!("{}:/opt/analyzer.jar:ro", analyzer_jar.display()))
        .arg(ANALYZER_IMAGE)
        .arg("java")
        .arg("-DbatchMode=true")
        .arg("-jar")
        .arg("/opt/analyzer.jar")
        .arg(mode)
        .arg("--wildfly")
        .arg(format!("{}:9990", instance.name))
        .arg("--wildfly-user")
        .arg("admin")
        .arg("--wildfly-password")
        .arg("admin")
        .arg("--neo4j")
        .arg(format!("{}:7687", neo4j.container_name()))
        .arg("/")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    stream_output(&mut child, &mut log_file, &mut error_buffer, progress).await?;

    drop(log_file);
    let status = child.wait().await?;
    if !status.success() {
        print_errors(&error_buffer, &log_path);
        bail!(
            "Analyzer failed with exit code {}",
            status.code().unwrap_or(-1)
        );
    }

    let _ = fs::remove_file(&log_path);
    Ok(())
}

/// Runs the analyzer with a doc-zip archive instead of a live WildFly instance.
pub(super) async fn run_doc_zip_analyzer(
    doc_zip_path: &Path,
    neo4j: &Neo4JContainer,
    network: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    progress.show_progress("starting analyzer...");

    let analyzer_jar = temp_dir().join("analyzer.jar");
    if !analyzer_jar.exists() {
        let dl_progress = Progress::new("analyzer");
        download_analyzer(&crate::constants::analyzer_url(), &dl_progress).await?;
        dl_progress.finish_success(Some("ready"));
    }

    let analyzer_container_name = format!("mgt-analyzer-{}", neo4j.container_name());
    let log_path = temp_dir().join(format!("{}.log", analyzer_container_name));
    let mut log_file = BufWriter::new(File::create(&log_path)?);
    let mut error_buffer: VecDeque<String> = VecDeque::with_capacity(ERROR_BUFFER_CAPACITY);

    let mut cmd = container_command()?;
    let mut child = cmd
        .arg("run")
        .arg("--rm")
        .arg("--name")
        .arg(&analyzer_container_name)
        .arg("--network")
        .arg(network)
        .arg("--volume")
        .arg(format!("{}:/opt/analyzer.jar:ro", analyzer_jar.display()))
        .arg("--volume")
        .arg(format!("{}:/opt/doc.zip:ro", doc_zip_path.display()))
        .arg(ANALYZER_IMAGE)
        .arg("java")
        .arg("-DbatchMode=true")
        .arg("-jar")
        .arg("/opt/analyzer.jar")
        .arg("--clean")
        .arg("--doc-zip")
        .arg("/opt/doc.zip")
        .arg("--neo4j")
        .arg(format!("{}:7687", neo4j.container_name()))
        .arg("/")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    stream_output(&mut child, &mut log_file, &mut error_buffer, progress).await?;

    drop(log_file);
    let status = child.wait().await?;
    if !status.success() {
        print_errors(&error_buffer, &log_path);
        bail!(
            "Analyzer failed with exit code {}",
            status.code().unwrap_or(-1)
        );
    }

    let _ = fs::remove_file(&log_path);
    Ok(())
}

/// Streams stdout and stderr from a child process, writing to a log file
/// and updating progress with parsed analyzer resource paths.
async fn stream_output(
    child: &mut tokio::process::Child,
    log_file: &mut BufWriter<File>,
    error_buffer: &mut VecDeque<String>,
    progress: &Progress,
) -> anyhow::Result<()> {
    let stdout = child.stdout.take().expect("stdout should be piped");
    let stderr = child.stderr.take().expect("stderr should be piped");
    let mut stdout_lines = tokio::io::BufReader::new(stdout).lines();
    let mut stderr_lines = tokio::io::BufReader::new(stderr).lines();
    let mut stdout_done = false;
    let mut stderr_done = false;

    loop {
        tokio::select! {
            result = stdout_lines.next_line(), if !stdout_done => {
                match result? {
                    Some(line) => {
                        let _ = writeln!(log_file, "{}", line);
                        append_line(error_buffer, &line);
                        if let Some(resource) = parse_analyzer_resource(&line) {
                            progress.show_progress(resource);
                        }
                    }
                    None => stdout_done = true,
                }
            }
            result = stderr_lines.next_line(), if !stderr_done => {
                match result? {
                    Some(line) => {
                        let _ = writeln!(log_file, "{}", line);
                        append_line(error_buffer, &line);
                    }
                    None => stderr_done = true,
                }
            }
        }
        if stdout_done && stderr_done {
            break;
        }
    }

    Ok(())
}

fn append_line(buffer: &mut VecDeque<String>, line: &str) {
    if buffer.len() >= ERROR_BUFFER_CAPACITY {
        buffer.pop_front();
    }
    buffer.push_back(line.to_string());
}

fn print_errors(buffer: &VecDeque<String>, log_path: &Path) {
    if buffer.is_empty() {
        return;
    }
    println!();
    for line in buffer {
        println!("    {}", style(line).dim());
    }
    println!("    {} {}", style("full log:").dim(), log_path.display());
}

fn parse_analyzer_resource(line: &str) -> Option<&str> {
    let marker = "Read /";
    let pos = line.find(marker)?;
    Some(&line[pos + marker.len() - 1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_batch_mode_resource() {
        let line = "Read /subsystem=undertow/server=*/host=*/setting=access-log";
        assert_eq!(
            parse_analyzer_resource(line),
            Some("/subsystem=undertow/server=*/host=*/setting=access-log")
        );
    }

    #[test]
    fn parse_verbose_resource() {
        let line =
            "08:58:18.768 [main] INFO  o.w.modelgraph.analyzer.Analyzer - Read /subsystem=elytron";
        assert_eq!(parse_analyzer_resource(line), Some("/subsystem=elytron"));
    }

    #[test]
    fn parse_unrelated_line() {
        assert_eq!(parse_analyzer_resource("Some other log line"), None);
    }

    #[test]
    fn parse_empty_line() {
        assert_eq!(parse_analyzer_resource(""), None);
    }
}
