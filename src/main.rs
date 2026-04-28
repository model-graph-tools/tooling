mod analyze;
mod app;
mod args;
mod browse;
mod completion;
mod completions;
mod constants;
mod container;
mod feature_pack;
mod label;
mod neo4j;
mod progress;
mod ps;
mod source;
mod start;
mod stop;

use crate::analyze::analyze;
use crate::args::{source_argument, sources_argument};
use crate::browse::browse;
use crate::completion::{complete_multiple_identifiers, complete_single_identifier};
use crate::completions::completions;
use crate::ps::ps;
use crate::source::Source;
use crate::start::start;
use crate::stop::stop;
use anyhow::Result;
use app::build_app;
use clap_complete::engine::ArgValueCompleter;

fn build_app_full() -> clap::Command {
    build_app()
        .mut_subcommand("analyze", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_source)
                    .add(ArgValueCompleter::new(complete_single_identifier))
            })
        })
        .mut_subcommand("start", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_source_list)
                    .add(ArgValueCompleter::new(complete_multiple_identifiers))
            })
        })
        .mut_subcommand("stop", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_source_list)
                    .add(ArgValueCompleter::new(complete_multiple_identifiers))
            })
        })
        .mut_subcommand("browse", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_source_list)
                    .add(ArgValueCompleter::new(complete_multiple_identifiers))
            })
        })
}

#[tokio::main]
async fn main() -> Result<()> {
    clap_complete::CompleteEnv::with_factory(build_app_full).complete();

    let matches = build_app_full().get_matches();

    match matches.subcommand() {
        Some(("analyze", m)) => analyze(&source_argument(m)).await,
        Some(("start", m)) => start(&sources_argument(m)).await,
        Some(("stop", m)) => {
            let all = m.get_flag("all");
            let sources = m.get_one::<Vec<Source>>("identifier");
            stop(sources.map(|v| v.as_slice()), all).await
        }
        Some(("ps", _)) => ps().await,
        Some(("browse", m)) => {
            for source in &sources_argument(m) {
                browse(source)?;
            }
            Ok(())
        }
        Some(("completions", m)) => completions(m),
        _ => unreachable!("Unknown subcommand"),
    }
}

// ------------------------------------------------------ validation

fn parse_source(input: &str) -> Result<Source, String> {
    Source::parse(input).map_err(|err| err.to_string())
}

fn parse_source_list(input: &str) -> Result<Vec<Source>, String> {
    Source::parse_list(input).map_err(|err| err.to_string())
}
