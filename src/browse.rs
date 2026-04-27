use crate::container::verify_container_command;
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use crate::source::Source;
use console::style;

pub fn browse(source: &Source) -> anyhow::Result<()> {
    verify_container_command()?;

    let image = Neo4JImage::new(source);
    let neo4j = Neo4JContainer::new(image);
    let url = format!("http://localhost:{}/browser", neo4j.ports.http);
    println!(
        "\nOpening Neo4J browser for {} at {}",
        style(source.display_name()).cyan(),
        style(&url).cyan()
    );
    webbrowser::open(&url)?;
    Ok(())
}
