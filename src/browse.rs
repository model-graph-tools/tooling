use crate::container::verify_container_command;
use crate::neo4j::{Neo4JContainer, Neo4JImage};
use console::style;
use wildfly_container_versions::WildFlyContainer;

pub fn browse(wildfly_container: &WildFlyContainer) -> anyhow::Result<()> {
    verify_container_command()?;

    let image = Neo4JImage::new(wildfly_container);
    let neo4j = Neo4JContainer::new(image);
    let url = format!("http://localhost:{}/browser", neo4j.ports.http);
    println!(
        "\nOpening Neo4J browser for WildFly {} at {}",
        style(wildfly_container.display_version()).cyan(),
        style(&url).cyan()
    );
    webbrowser::open(&url)?;
    Ok(())
}
