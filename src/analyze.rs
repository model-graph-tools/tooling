use crate::constants::analyzer_url;
use crate::container::container_command;
use crate::neo4j::Neo4J;
use crate::progress::{AnalysisStatus, Progress, done, step_header, summary};
use anyhow::{Context, anyhow, bail};
use console::style;
use indicatif::MultiProgress;
use std::collections::VecDeque;
use std::env::temp_dir;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep};
use wado::{AdminContainer, Ports, ServerType, StandaloneInstance};
use which::which;
use wildfly_container_versions::WildFlyContainer;

const TOTAL_STEPS: u32 = 4;

// ------------------------------------------------------ configuration

struct WildFlyConfiguration {
    config: &'static str,
    suffix: &'static str,
    append: bool,
}

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

// ------------------------------------------------------ analyze

pub async fn analyze(wildfly_container: &WildFlyContainer) -> anyhow::Result<()> {
    crate::container::verify_container_command()?;
    which("java").with_context(|| "java not found")?;

    let instant = Instant::now();
    let version = wildfly_container.display_version();
    println!(
        "\n{}",
        style(format!("Analyzing WildFly {}", version)).bold()
    );

    run_analysis(wildfly_container).await?;

    done(instant);
    Ok(())
}

async fn run_analysis(wildfly_container: &WildFlyContainer) -> anyhow::Result<()> {
    let configs = wildfly_configurations(wildfly_container);
    let admin_container = AdminContainer::new(wildfly_container.clone(), ServerType::Standalone);
    let neo4j = Neo4J::new(wildfly_container);

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

    prepare_environment(&instances, &configs, &neo4j).await?;
    let analyzer_jar = temp_dir().join("analyzer.jar");

    let result = async {
        run_analyzers(&analyzer_jar, &instances, &configs, &neo4j).await?;
        build_neo4j_image(&neo4j).await
    }
    .await;

    if let Err(ref e) = result {
        eprintln!("\n{}: {}", style("Error").red().bold(), e);
    }
    cleanup(&instances, &neo4j).await?;

    result
}

// ------------------------------------------------------ step 1: prepare environment

async fn prepare_environment(
    instances: &[StandaloneInstance],
    configs: &[WildFlyConfiguration],
    neo4j: &Neo4J,
) -> anyhow::Result<PathBuf> {
    step_header(1, TOTAL_STEPS, "Preparing environment...");
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    // Download analyzer
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

    // Start WildFly instances
    for (instance, cfg) in instances.iter().zip(configs.iter()) {
        let instance = instance.clone();
        let config = cfg.config.to_string();
        let progress = Progress::join(&multi_progress, &config);
        tasks.spawn(async move {
            let result = start_wildfly(&instance, &config, &progress).await;
            match &result {
                Ok(()) => progress.finish_success(Some("ready")),
                Err(e) => progress.finish_error(&e.to_string()),
            }
            result.map(|()| PrepareResult::WildFly)
        });
    }

    // Start Neo4J
    let neo4j_clone = neo4j.clone();
    let neo4j_progress = Progress::join(&multi_progress, "neo4j");
    tasks.spawn(async move {
        let result = start_neo4j(&neo4j_clone, &neo4j_progress).await;
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

// ------------------------------------------------------ step 2: analyze

async fn run_analyzers(
    analyzer_jar: &Path,
    instances: &[StandaloneInstance],
    configs: &[WildFlyConfiguration],
    neo4j: &Neo4J,
) -> anyhow::Result<()> {
    step_header(2, TOTAL_STEPS, "Analyzing...");
    for (instance, cfg) in instances.iter().zip(configs.iter()) {
        let progress = Progress::new(cfg.config);
        let mode = if cfg.append { "--append" } else { "--clean" };
        let result = run_analyzer(analyzer_jar, instance, neo4j, mode, &progress).await;
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

// ------------------------------------------------------ step 3: build neo4j image

async fn build_neo4j_image(neo4j: &Neo4J) -> anyhow::Result<()> {
    step_header(3, TOTAL_STEPS, "Building Neo4J image...");
    let progress = Progress::new(&neo4j.image_tag());

    progress.show_progress("stopping neo4j...");
    stop_container(&neo4j.container_name()).await?;

    neo4j.build_image(&progress).await?;

    progress.finish_success(Some("ready"));
    Ok(())
}

// ------------------------------------------------------ step 4: cleanup

async fn cleanup(instances: &[StandaloneInstance], neo4j: &Neo4J) -> anyhow::Result<()> {
    step_header(4, TOTAL_STEPS, "Cleaning up...");
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();
    let mut status: Vec<AnalysisStatus> = Vec::new();

    // Stop WildFly instances (parallel)
    for instance in instances {
        let container_name = instance.name.clone();
        let progress = Progress::join(&multi_progress, &container_name);
        tasks.spawn(async move {
            match stop_container(&container_name).await {
                Ok(()) => {
                    progress.finish_success(Some("stopped"));
                    AnalysisStatus::success(&container_name)
                }
                Err(e) => {
                    let msg = e.to_string();
                    progress.finish_error(&msg);
                    AnalysisStatus::error(&container_name, &msg)
                }
            }
        });
    }
    status.extend(tasks.join_all().await);

    // Stop and remove Neo4J container, then remove volume (sequential)
    let neo4j_container = neo4j.container_name();
    let neo4j_progress = Progress::new(&neo4j_container);
    let _ = stop_container(&neo4j_container).await;
    match remove_container(&neo4j_container).await {
        Ok(()) => {
            neo4j_progress.finish_success(Some("removed"));
            status.push(AnalysisStatus::success(&neo4j_container));
        }
        Err(e) => {
            let msg = e.to_string();
            neo4j_progress.finish_error(&msg);
            status.push(AnalysisStatus::error(&neo4j_container, &msg));
        }
    }

    let volume_name = neo4j.volume_name();
    let volume_progress = Progress::new(&volume_name);
    match remove_volume(&volume_name).await {
        Ok(()) => {
            volume_progress.finish_success(Some("removed"));
            status.push(AnalysisStatus::success(&volume_name));
        }
        Err(e) => {
            let msg = e.to_string();
            volume_progress.finish_error(&msg);
            status.push(AnalysisStatus::error(&volume_name, &msg));
        }
    }

    let count = status.len();
    summary(count, &status);
    Ok(())
}

// ------------------------------------------------------ container operations

async fn start_wildfly(
    instance: &StandaloneInstance,
    configuration: &str,
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

async fn start_neo4j(neo4j: &Neo4J, progress: &Progress) -> anyhow::Result<()> {
    progress.show_progress("creating volume...");
    let mut volume_cmd = container_command()?;
    volume_cmd
        .arg("volume")
        .arg("create")
        .arg("--ignore")
        .arg(neo4j.volume_name())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = volume_cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to create volume: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    progress.show_progress("starting container...");
    let mut run_cmd = container_command()?;
    run_cmd
        .arg("run")
        .arg("--detach")
        .arg("--name")
        .arg(neo4j.container_name())
        .arg("--publish")
        .arg(format!("{}:7687", neo4j.bolt_port))
        .arg("--publish")
        .arg(format!("{}:7474", neo4j.http_port))
        .arg("--env")
        .arg("NEO4J_AUTH=none")
        .arg("--volume")
        .arg(format!("{}:/data", neo4j.volume_name()))
        .arg(Neo4J::image_name())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = run_cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to start Neo4J: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    progress.show_progress("waiting for Neo4J...");
    healthcheck(
        &format!("http://localhost:{}/browser", neo4j.http_port),
        progress,
    )
    .await?;

    Ok(())
}

async fn stop_container(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("stop")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to stop {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

async fn remove_container(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("rm")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to remove container {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

async fn remove_volume(name: &str) -> anyhow::Result<()> {
    let mut cmd = container_command()?;
    cmd.arg("volume")
        .arg("rm")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd.output().await?;
    if !output.status.success() {
        bail!(
            "Failed to remove volume {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

// ------------------------------------------------------ analyzer

async fn download_analyzer(url: &str, progress: &Progress) -> anyhow::Result<PathBuf> {
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

const ERROR_BUFFER_CAPACITY: usize = 20;

async fn run_analyzer(
    analyzer_jar: &Path,
    instance: &StandaloneInstance,
    neo4j: &Neo4J,
    mode: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    progress.show_progress("starting analyzer...");

    let log_path = temp_dir().join(format!(
        "mgt-analyzer-{}-{}.log",
        instance.admin_container.wildfly_container.identifier,
        if mode == "--clean" { "fha" } else { "mp" }
    ));
    let mut log_file = BufWriter::new(File::create(&log_path)?);
    let mut error_buffer: VecDeque<String> = VecDeque::with_capacity(ERROR_BUFFER_CAPACITY);

    let mut child = tokio::process::Command::new("java")
        .arg("-DbatchMode=true")
        .arg("-jar")
        .arg(analyzer_jar)
        .arg(mode)
        .arg("--wildfly")
        .arg(format!("localhost:{}", instance.ports.management))
        .arg("--wildfly-user")
        .arg("admin")
        .arg("--wildfly-password")
        .arg("admin")
        .arg("--neo4j")
        .arg(format!("localhost:{}", neo4j.bolt_port))
        // .arg("/")
        .arg("/subsystem=undertow")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

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
                        append_line(&mut error_buffer, &line);
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
                        append_line(&mut error_buffer, &line);
                    }
                    None => stderr_done = true,
                }
            }
        }
        if stdout_done && stderr_done {
            break;
        }
    }

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
        let line = "08:58:18.768 [main] INFO  o.w.modelgraph.analyzer.Analyzer - Read /subsystem=elytron";
        assert_eq!(
            parse_analyzer_resource(line),
            Some("/subsystem=elytron")
        );
    }

    #[test]
    fn parse_unrelated_line() {
        assert_eq!(parse_analyzer_resource("Some other log line"), None);
    }

    #[test]
    fn parse_empty_line() {
        assert_eq!(parse_analyzer_resource(""), None);
    }

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

// ------------------------------------------------------ healthcheck

const MAX_HEALTHCHECK_ATTEMPTS: u32 = 30;

async fn healthcheck(url: &str, progress: &Progress) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    for attempt in 1..=MAX_HEALTHCHECK_ATTEMPTS {
        progress.show_progress(&format!(
            "healthcheck {}/{}",
            attempt, MAX_HEALTHCHECK_ATTEMPTS
        ));
        if let Ok(response) = client.get(url).send().await
            && response.status().is_success()
        {
            return Ok(());
        }
        sleep(Duration::from_secs(1)).await;
    }
    bail!(
        "Healthcheck failed after {} attempts: {}",
        MAX_HEALTHCHECK_ATTEMPTS,
        url
    )
}
