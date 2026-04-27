use crate::container::{neo4j_container_details, verify_container_command};
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};

pub async fn ps() -> anyhow::Result<()> {
    verify_container_command()?;

    let containers = neo4j_container_details().await?;
    if containers.is_empty() {
        println!("\nNo running Neo4J model DB containers found.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Version", "Name", "Ports", "Status", "ID"]);

    for container in containers {
        table.add_row(vec![
            Cell::new(container.version).fg(Color::DarkMagenta),
            Cell::new(container.name).fg(Color::DarkYellow),
            Cell::new(container.ports).fg(Color::Green),
            Cell::new(container.status),
            Cell::new(container.id).fg(Color::Grey),
        ]);
    }

    println!("\n{table}");
    Ok(())
}
