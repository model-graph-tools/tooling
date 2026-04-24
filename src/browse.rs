use crate::container::verify_container_command;
use crate::neo4j::Neo4J;
use console::style;
use wildfly_container_versions::WildFlyContainer;

pub fn browse(wildfly_container: &WildFlyContainer) -> anyhow::Result<()> {
    verify_container_command()?;

    let neo4j = Neo4J::new(wildfly_container);
    let url = format!("http://localhost:{}/browser", neo4j.http_port);
    println!(
        "\nOpening Neo4J browser for WildFly {} at {}",
        style(wildfly_container.display_version()).cyan(),
        style(&url).cyan()
    );
    webbrowser::open(&url)?;
    Ok(())
}
