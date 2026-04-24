mod analyze;
mod app;
mod args;
mod constants;
mod container;
mod neo4j;
mod progress;

use crate::analyze::analyze;
use crate::args::wildfly_container_argument;
use anyhow::Result;
use app::build_app;
use wildfly_container_versions::WildFlyContainer;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = build_app()
        .mut_subcommand("analyze", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| arg.value_parser(parse_version))
        })
        .get_matches();

    match matches.subcommand() {
        Some(("analyze", m)) => analyze(&wildfly_container_argument(m)).await,
        _ => unreachable!("Unknown subcommand"),
    }
}

// ------------------------------------------------------ validation

fn parse_version(version: &str) -> Result<WildFlyContainer, String> {
    WildFlyContainer::version(version).map_err(|err| err.to_string())
}
