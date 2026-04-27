mod analyze;
mod app;
mod args;
mod browse;
mod completion;
mod completions;
mod constants;
mod container;
mod neo4j;
mod progress;
mod ps;
mod start;
mod stop;

use crate::analyze::analyze;
use crate::args::{wildfly_container_argument, wildfly_containers_argument};
use crate::browse::browse;
use crate::completion::complete_versions;
use crate::completions::completions;
use crate::ps::ps;
use crate::start::start;
use crate::stop::stop;
use anyhow::Result;
use app::build_app;
use clap_complete::engine::ArgValueCompleter;
use wildfly_container_versions::WildFlyContainer;

fn build_app_full() -> clap::Command {
    build_app()
        .mut_subcommand("analyze", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_version)
                    .add(ArgValueCompleter::new(complete_versions))
            })
        })
        .mut_subcommand("start", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_version_enumeration)
                    .add(ArgValueCompleter::new(complete_versions))
            })
        })
        .mut_subcommand("stop", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_version_enumeration)
                    .add(ArgValueCompleter::new(complete_versions))
            })
        })
        .mut_subcommand("browse", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_version)
                    .add(ArgValueCompleter::new(complete_versions))
            })
        })
}

#[tokio::main]
async fn main() -> Result<()> {
    clap_complete::CompleteEnv::with_factory(build_app_full).complete();

    let matches = build_app_full().get_matches();

    match matches.subcommand() {
        Some(("analyze", m)) => analyze(&wildfly_container_argument(m)).await,
        Some(("start", m)) => start(&wildfly_containers_argument(m)).await,
        Some(("stop", m)) => {
            let all = m.get_flag("all");
            let containers = m.get_one::<Vec<WildFlyContainer>>("identifier");
            stop(containers.map(|v| v.as_slice()), all).await
        }
        Some(("ps", _)) => ps().await,
        Some(("browse", m)) => Ok(browse(&wildfly_container_argument(m))?),
        Some(("completions", m)) => completions(m),
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
