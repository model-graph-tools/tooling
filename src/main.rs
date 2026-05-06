//! CLI entry point and subcommand dispatch for `mgt`.

mod app;
mod args;
mod command;
mod completion;
mod constants;
mod container;
mod download;
mod error;
mod json;
mod label;
mod neo4j;
mod progress;
mod registry;

use crate::args::{meta_item_argument, meta_items_argument};
use crate::command::{
    analyze, browse, completions, feature_packs_cmd, images, ps, push, resolve, start, stop,
    update, versions,
};
use crate::completion::{complete_multiple_identifiers, complete_single_identifier};
use crate::error::{JsonErrorEnvelope, MgtError};
use crate::registry::{images_registry, init_registries, packs_registry};
use anyhow::Result;
use app::build_app;
use clap_complete::engine::ArgValueCompleter;
use wildfly_meta::{DslOptions, MetaItem, parse_meta_item, parse_meta_items};

/// Extends [`build_app()`] with value parsers and tab-completion handlers.
fn build_app_full() -> clap::Command {
    build_app()
        .mut_subcommand("analyze", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_single)
                    .add(ArgValueCompleter::new(complete_single_identifier))
            })
        })
        .mut_subcommand("push", |sub_cmd| {
            sub_cmd
                .mut_arg("identifier", |arg| {
                    arg.value_parser(parse_list)
                        .add(ArgValueCompleter::new(complete_multiple_identifiers))
                })
                .mut_arg("chunks", |arg| arg.value_parser(clap::value_parser!(u16)))
        })
        .mut_subcommand("start", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_list)
                    .add(ArgValueCompleter::new(complete_multiple_identifiers))
            })
        })
        .mut_subcommand("stop", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_list)
                    .add(ArgValueCompleter::new(complete_multiple_identifiers))
            })
        })
        .mut_subcommand("resolve", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_list)
                    .add(ArgValueCompleter::new(complete_multiple_identifiers))
            })
        })
        .mut_subcommand("browse", |sub_cmd| {
            sub_cmd.mut_arg("identifier", |arg| {
                arg.value_parser(parse_list)
                    .add(ArgValueCompleter::new(complete_multiple_identifiers))
            })
        })
}

#[tokio::main]
async fn main() {
    let json = std::env::args().any(|a| a == "--json");
    if let Err(e) = run(json).await {
        if json {
            let envelope = JsonErrorEnvelope::from_anyhow(&e);
            match serde_json::to_string(&envelope) {
                Ok(json) => println!("{json}"),
                Err(ser) => eprintln!("Error: {e:#}\n(JSON serialization also failed: {ser})"),
            }
        } else {
            eprintln!("Error: {e:#}");
        }
        std::process::exit(1);
    }
}

async fn run(json: bool) -> Result<()> {
    let registry_error = init_registries().await.err();

    if registry_error.is_none() {
        clap_complete::CompleteEnv::with_factory(build_app_full).complete();
    }

    let app = if registry_error.is_none() {
        build_app_full()
    } else {
        build_app()
    };
    let matches = match app.try_get_matches() {
        Ok(m) => m,
        Err(e) if json && e.use_stderr() => return Err(classify_clap_error(e)),
        Err(e) => e.exit(),
    };

    // Registry-free commands: handle first, return early
    match matches.subcommand() {
        Some(("update", _)) => return update().await,
        Some(("ps", _)) => return ps(json).await,
        Some(("completions", m)) => return completions(m),
        _ => {}
    }

    // All remaining commands require registries
    if let Some(e) = registry_error {
        return Err(MgtError::registry_init_failed(&e.to_string()).into());
    }

    match matches.subcommand() {
        Some(("analyze", m)) => analyze(&meta_item_argument(m)).await,
        Some(("push", m)) => {
            let chunk_size = m.get_one::<u16>("chunks").copied().unwrap_or(0);
            push(&meta_items_argument(m), chunk_size).await
        }
        Some(("start", m)) => start(&meta_items_argument(m), json).await,
        Some(("stop", m)) => {
            let all = m.get_flag("all");
            let items = m.get_one::<Vec<MetaItem>>("identifier");
            stop(items.map(|v| v.as_slice()), all, json).await
        }
        Some(("versions", _)) => {
            versions(json);
            Ok(())
        }
        Some(("feature-packs", _)) => {
            feature_packs_cmd(json);
            Ok(())
        }
        Some(("images", m)) => {
            let wildfly = m.get_flag("wildfly");
            let feature_packs = m.get_flag("feature-packs");
            images(wildfly, feature_packs).await
        }
        Some(("resolve", m)) => {
            resolve(&meta_items_argument(m), json);
            Ok(())
        }
        Some(("browse", m)) => {
            for item in &meta_items_argument(m) {
                browse(item)?;
            }
            Ok(())
        }
        _ => unreachable!("Unknown subcommand"),
    }
}

fn classify_clap_error(err: clap::Error) -> anyhow::Error {
    match err.kind() {
        clap::error::ErrorKind::ValueValidation => {
            MgtError::unknown_identifier(err.to_string().trim()).into()
        }
        _ => MgtError::clap_parse_error(err.to_string().trim()).into(),
    }
}

// ------------------------------------------------------ validation

/// Clap value parser that converts a CLI string into a single [`MetaItem`].
fn parse_single(input: &str) -> Result<MetaItem, String> {
    parse_meta_item(input, images_registry(), packs_registry()).map_err(|err| err.to_string())
}

/// Clap value parser that converts a comma-separated or range string into a list of [`MetaItem`]s.
fn parse_list(input: &str) -> Result<Vec<MetaItem>, String> {
    parse_meta_items(
        input,
        images_registry(),
        packs_registry(),
        &DslOptions::all(),
        &DslOptions::none(),
    )
    .map_err(|err| err.to_string())
}
