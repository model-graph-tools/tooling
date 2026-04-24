mod analyze;
mod app;
mod args;
mod browse;
mod constants;
mod container;
mod neo4j;
mod progress;
mod start;
mod stop;

use crate::analyze::analyze;
use crate::args::{wildfly_container_argument, wildfly_containers_argument};
use crate::browse::browse;
use crate::start::start;
use crate::stop::stop;
use anyhow::Result;
use app::build_app;
use wildfly_container_versions::WildFlyContainer;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = build_app()
        .mut_subcommand("analyze", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| arg.value_parser(parse_version))
        })
        .mut_subcommand("start", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| arg.value_parser(parse_version_enumeration))
        })
        .mut_subcommand("stop", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| arg.value_parser(parse_version_enumeration))
        })
        .mut_subcommand("browse", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| arg.value_parser(parse_version))
        })
        .get_matches();

    match matches.subcommand() {
        Some(("analyze", m)) => analyze(&wildfly_container_argument(m)).await,
        Some(("start", m)) => start(&wildfly_containers_argument(m)).await,
        Some(("stop", m)) => {
            let all = m.get_flag("all");
            let containers = m.get_one::<Vec<WildFlyContainer>>("identifier");
            stop(containers.map(|v| v.as_slice()), all).await
        }
        Some(("browse", m)) => Ok(browse(&wildfly_container_argument(m))?),
        _ => unreachable!("Unknown subcommand"),
    }
}

// ------------------------------------------------------ validation

fn parse_version(version: &str) -> Result<WildFlyContainer, String> {
    WildFlyContainer::version(version).map_err(|err| err.to_string())
}

fn parse_version_enumeration(version: &str) -> Result<Vec<WildFlyContainer>, String> {
    WildFlyContainer::enumeration(version).map_err(|err| err.to_string())
}
