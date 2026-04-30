//! Lists running Neo4J model DB containers in a table.

use crate::container::{running_neo4j_containers, verify_container_command};
use crate::json::ContainerInfo;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};

/// Lists all running Neo4J model DB containers in a formatted table.
pub async fn ps(json: bool) -> anyhow::Result<()> {
    verify_container_command()?;

    let containers = running_neo4j_containers().await?;

    if json {
        let infos: Vec<ContainerInfo> = containers
            .iter()
            .map(|running| ContainerInfo {
                identifier: running.container.image.item.expression(),
                source_type: running.container.image.item.kind().to_string(),
                name: running.container.image.item.full_name(),
                container_name: running.container.container_name(),
                bolt: running.container.ports.bolt,
                http: running.container.ports.http,
                status: running.status.clone(),
                id: running.id.clone(),
            })
            .collect();
        println!("{}", serde_json::to_string(&infos).unwrap());
        return Ok(());
    }

    if containers.is_empty() {
        println!("\nNo running model containers found.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Source", "Name", "Ports", "Status", "ID"]);

    for running in &containers {
        let source_name = running.container.image.item.full_name();
        let ports = format!(
            "bolt: {}, http: {}",
            running.container.ports.bolt, running.container.ports.http
        );
        table.add_row(vec![
            Cell::new(source_name).fg(Color::DarkMagenta),
            Cell::new(running.container.container_name()).fg(Color::DarkYellow),
            Cell::new(ports).fg(Color::Green),
            Cell::new(&running.status),
            Cell::new(&running.id).fg(Color::Grey),
        ]);
    }

    println!("\n{table}");
    Ok(())
}
