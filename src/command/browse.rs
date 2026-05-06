//! Opens the Neo4J browser for a given source's container.

use crate::container::verify_container_command;
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use console::style;
use wildfly_meta::MetaItem;

/// Opens the Neo4J browser UI in the default web browser for the given meta item.
pub fn browse(item: &MetaItem) -> anyhow::Result<()> {
    verify_container_command()?;

    let image = Neo4JImage::new(item);
    let neo4j = Neo4JContainer::new(image)?;
    let url = format!(
        "http://localhost:{}/browser?dbms=bolt://localhost:{}",
        neo4j.ports.http, neo4j.ports.bolt
    );
    println!(
        "\nOpening Neo4J browser for {} at {}",
        style(item.short_name()).cyan(),
        style(&url).cyan()
    );
    webbrowser::open(&url)?;
    Ok(())
}
