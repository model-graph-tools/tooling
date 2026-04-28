//! CLI entry point and subcommand dispatch for `mgt`.

mod app;
mod args;
mod command;
mod completion;
mod constants;
mod container;
mod download;
mod feature_pack;
mod label;
mod neo4j;
mod progress;
mod source;

use crate::args::{source_argument, sources_argument};
use crate::command::{
    analyze, browse, completions, feature_packs_cmd, images, ps, start, stop, versions,
};
use crate::completion::{complete_multiple_identifiers, complete_single_identifier};
use crate::source::Source;
use anyhow::Result;
use app::build_app;
use clap_complete::engine::ArgValueCompleter;

/// Extends [`build_app()`] with value parsers and tab-completion handlers.
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
        Some(("versions", _)) => {
            versions();
            Ok(())
        }
        Some(("feature-packs", _)) => {
            feature_packs_cmd();
            Ok(())
        }
        Some(("images", m)) => {
            let wildfly = m.get_flag("wildfly");
            let feature_packs = m.get_flag("feature-packs");
            images(wildfly, feature_packs).await
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

/// Clap value parser that converts a CLI string into a single [`Source`].
fn parse_source(input: &str) -> Result<Source, String> {
    Source::parse(input).map_err(|err| err.to_string())
}

/// Clap value parser that converts a comma-separated or range string into a list of [`Source`]s.
fn parse_source_list(input: &str) -> Result<Vec<Source>, String> {
    Source::parse_list(input).map_err(|err| err.to_string())
}
