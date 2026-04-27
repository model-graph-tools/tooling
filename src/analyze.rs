use crate::constants::analyzer_url;
use crate::container::{
    container_command, create_network, healthcheck, network_name, remove_container, remove_network,
    remove_volume, stop_container,
};
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::progress::{CommandStatus, Progress, done, step_header, summary};
use crate::source::Source;
use anyhow::{anyhow, bail};
use console::style;
use indicatif::MultiProgress;
use std::collections::VecDeque;
use std::env::temp_dir;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::task::JoinSet;
use tokio::time::Instant;
use wado::{AdminContainer, Ports, ServerType, StandaloneInstance};
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

pub async fn analyze(source: &Source) -> anyhow::Result<()> {
    crate::container::verify_container_command()?;

    let instant = Instant::now();
    println!(
        "\n{}",
        style(format!("Analyzing {}", source.display_name())).bold()
    );

    match source {
        Source::WildFly(wc) => run_wildfly_analysis(wc, source).await?,
        Source::FeaturePack(fp) => run_feature_pack_analysis(fp, source).await?,
    }

    done(instant);
    Ok(())
}

// ------------------------------------------------------ WildFly analysis

async fn run_wildfly_analysis(
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

// ------------------------------------------------------ feature pack analysis

async fn run_feature_pack_analysis(
    fp: &crate::feature_pack::FeaturePack,
    source: &Source,
) -> anyhow::Result<()> {
    let neo4j_image = Neo4JImage::new(source);
    let neo4j = Neo4JContainer::new(neo4j_image);
    let network = network_name(source);

    create_network(&network).await?;

    let result = async {
        // Step 1: Prepare environment (download doc-zip + start Neo4J)
        step_header(1, TOTAL_STEPS, "Preparing environment...");
        let multi_progress = MultiProgress::new();
        let mut tasks = JoinSet::new();

        let dl_progress = Progress::join(&multi_progress, "doc-zip");
        let url = fp.download_url();
        tasks.spawn(async move {
            let result = download_doc_zip(&url, &dl_progress).await;
            match &result {
                Ok(_) => dl_progress.finish_success(Some("ready")),
                Err(e) => dl_progress.finish_error(&e.to_string()),
            }
            result
        });

        let neo4j_clone = neo4j.clone();
        let network_clone = network.clone();
        let neo4j_progress = Progress::join(&multi_progress, "neo4j");
        tasks.spawn(async move {
            let result = start_neo4j(&neo4j_clone, &network_clone, &neo4j_progress).await;
            match &result {
                Ok(()) => neo4j_progress.finish_success(Some("ready")),
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

        // Step 2: Run analyzer with doc-zip
        step_header(2, TOTAL_STEPS, "Analyzing...");
        let progress = Progress::new(&fp.display_name());
        let result =
            run_doc_zip_analyzer(&doc_zip_path, &neo4j, &network, &progress).await;
        match &result {
            Ok(()) => progress.finish_success(Some("done")),
            Err(e) => progress.finish_error(&e.to_string()),
        }
        result?;

        // Step 3: Build Neo4J image
        build_neo4j_image(&neo4j).await
    }
    .await;

    if let Err(ref e) = result {
        eprintln!("\n{}: {}", style("Error").red().bold(), e);
    }

    // Step 4: Cleanup
    step_header(4, TOTAL_STEPS, "Cleaning up...");
    let multi_progress = MultiProgress::new();
    let mut status: Vec<CommandStatus> = Vec::new();

    let neo4j_container = neo4j.container_name();
    let neo4j_progress = Progress::join(&multi_progress, &neo4j_container);
    let _ = stop_container(&neo4j_container).await;
    match remove_container(&neo4j_container).await {
        Ok(()) => {
            neo4j_progress.finish_success(Some("removed"));
            status.push(CommandStatus::success(&neo4j_container));
        }
        Err(e) => {
            let msg = e.to_string();
            neo4j_progress.finish_error(&msg);
            status.push(CommandStatus::error(&neo4j_container, &msg));
        }
    }

    let volume_name = neo4j.volume_name();
    let volume_progress = Progress::new(&volume_name);
    match remove_volume(&volume_name).await {
        Ok(()) => {
            volume_progress.finish_success(Some("removed"));
            status.push(CommandStatus::success(&volume_name));
        }
        Err(e) => {
            let msg = e.to_string();
            volume_progress.finish_error(&msg);
            status.push(CommandStatus::error(&volume_name, &msg));
        }
    }

    let network_progress = Progress::new(&network);
    match remove_network(&network).await {
        Ok(()) => {
            network_progress.finish_success(Some("removed"));
            status.push(CommandStatus::success(&network));
        }
        Err(e) => {
            let msg = e.to_string();
            network_progress.finish_error(&msg);
            status.push(CommandStatus::error(&network, &msg));
        }
    }

    let count = status.len();
    summary(count, &status);

    result
}

async fn download_doc_zip(url: &str, progress: &Progress) -> anyhow::Result<PathBuf> {
    let filename = url.rsplit('/').next().unwrap_or("doc.zip");
    let path = temp_dir().join(filename);
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

const ANALYZER_IMAGE: &str = "eclipse-temurin:25-jre";

async fn run_doc_zip_analyzer(
    doc_zip_path: &Path,
    neo4j: &Neo4JContainer,
    network: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    progress.show_progress("starting analyzer...");

    let analyzer_jar = temp_dir().join("analyzer.jar");
    if !analyzer_jar.exists() {
        let dl_progress = Progress::new("analyzer");
        download_analyzer(&analyzer_url(), &dl_progress).await?;
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

// ------------------------------------------------------ step 1: prepare environment

async fn prepare_environment(
    instances: &[StandaloneInstance],
    configs: &[WildFlyConfiguration],
    neo4j: &Neo4JContainer,
    network: &str,
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

    // Start Neo4J
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

// ------------------------------------------------------ step 2: analyze

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

// ------------------------------------------------------ step 3: build neo4j image

async fn build_neo4j_image(neo4j: &Neo4JContainer) -> anyhow::Result<()> {
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

// ------------------------------------------------------ step 4: cleanup

async fn cleanup(
    instances: &[StandaloneInstance],
    neo4j: &Neo4JContainer,
    network: &str,
) -> anyhow::Result<()> {
    step_header(4, TOTAL_STEPS, "Cleaning up...");
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();
    let mut status: Vec<CommandStatus> = Vec::new();

    // Stop WildFly instances (parallel)
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

    // Stop and remove Neo4J container, then remove volume (sequential)
    let neo4j_container = neo4j.container_name();
    let neo4j_progress = Progress::new(&neo4j_container);
    let _ = stop_container(&neo4j_container).await;
    match remove_container(&neo4j_container).await {
        Ok(()) => {
            neo4j_progress.finish_success(Some("removed"));
            status.push(CommandStatus::success(&neo4j_container));
        }
        Err(e) => {
            let msg = e.to_string();
            neo4j_progress.finish_error(&msg);
            status.push(CommandStatus::error(&neo4j_container, &msg));
        }
    }

    let volume_name = neo4j.volume_name();
    let volume_progress = Progress::new(&volume_name);
    match remove_volume(&volume_name).await {
        Ok(()) => {
            volume_progress.finish_success(Some("removed"));
            status.push(CommandStatus::success(&volume_name));
        }
        Err(e) => {
            let msg = e.to_string();
            volume_progress.finish_error(&msg);
            status.push(CommandStatus::error(&volume_name, &msg));
        }
    }

    let network_progress = Progress::new(network);
    match remove_network(network).await {
        Ok(()) => {
            network_progress.finish_success(Some("removed"));
            status.push(CommandStatus::success(network));
        }
        Err(e) => {
            let msg = e.to_string();
            network_progress.finish_error(&msg);
            status.push(CommandStatus::error(network, &msg));
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

async fn start_neo4j(
    neo4j: &Neo4JContainer,
    network: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
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
        .arg("--network")
        .arg(network)
        .arg("--publish")
        .arg(format!("{}:7687", neo4j.ports.bolt))
        .arg("--publish")
        .arg(format!("{}:7474", neo4j.ports.http))
        .arg("--env")
        .arg("NEO4J_AUTH=none")
        .arg("--volume")
        .arg(format!("{}:/data", neo4j.volume_name()))
        .arg(Neo4JImage::base_image_name())
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
        &format!("http://localhost:{}/browser", neo4j.ports.http),
        progress,
    )
    .await?;

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
