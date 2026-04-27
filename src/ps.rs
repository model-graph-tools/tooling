use crate::container::{running_neo4j_containers, verify_container_command};
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};

pub async fn ps() -> anyhow::Result<()> {
    verify_container_command()?;

    let containers = running_neo4j_containers().await?;
    if containers.is_empty() {
        println!("\nNo running Neo4J model DB containers found.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Version", "Name", "Ports", "Status", "ID"]);

    for running in &containers {
        let version = running
            .container
            .image
            .wildfly_container
            .display_version();
        let ports = format!(
            "bolt: {}, http: {}",
            running.container.ports.bolt, running.container.ports.http
        );
        table.add_row(vec![
            Cell::new(version).fg(Color::DarkMagenta),
            Cell::new(running.container.container_name()).fg(Color::DarkYellow),
            Cell::new(ports).fg(Color::Green),
            Cell::new(&running.status),
            Cell::new(&running.id).fg(Color::Grey),
        ]);
    }

    println!("\n{table}");
    Ok(())
}
