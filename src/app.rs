//! Base clap command tree with subcommands and arguments.
//!
//! Defines the CLI structure (subcommands, flags, help text) without value
//! parsers or completion logic. [`main.rs`](crate) extends this via
//! `build_app_full()` to wire up parsers and tab-completion.

use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use clap::{Arg, ArgAction, Command, crate_name, crate_version};

/// Builds the base clap `Command` tree with subcommands and arguments.
///
/// Does not attach value parsers or completion handlers — see
/// `build_app_full()` in `main.rs` for the fully wired version.
pub fn build_app() -> Command {
    Command::new(crate_name!())
        .version(crate_version!())
        .about("Command line tool to analyze the WildFly management model.")
        .styles(Styles::styled()
            .header(AnsiColor::Green.on_default() | Effects::BOLD)
            .usage(AnsiColor::Green.on_default() | Effects::BOLD)
            .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
            .placeholder(AnsiColor::Cyan.on_default()))
        .propagate_version(true)
        .subcommand_required(true)

        // analyze
        .subcommand(Command::new("analyze")
            .about("Analyze the management model of a WildFly instance or feature pack and build an image with a Neo4J database")
            .arg(Arg::new("identifier")
                .required(true)
                .help("A WildFly version (e.g. 39, 26.1) or feature pack (e.g. ai, graphql)")))

        // push
        .subcommand(Command::new("push")
            .about("Push Neo4J model DB images to the remote registry")
            .arg(Arg::new("identifier")
                .required(true)
                .help("WildFly versions, feature packs, or a mix (e.g. 34,ai,graphql)"))
            .arg(Arg::new("chunks")
                .short('c')
                .long("chunks")
                .value_name("SIZE")
                .help("Push in batches of SIZE images instead of all at once")))

        // start
        .subcommand(Command::new("start")
            .about("Start Neo4J model DB containers")
            .arg(Arg::new("identifier")
                .required(true)
                .help("WildFly versions, feature packs, or a mix (e.g. 34,ai,graphql)")))

        // stop
        .subcommand(Command::new("stop")
            .about("Stop Neo4J model DB containers")
            .arg(Arg::new("identifier")
                .required_unless_present("all")
                .help("WildFly versions, feature packs, or a mix (e.g. 34,ai,graphql)"))
            .arg(Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Stop all running Neo4J model DB containers")))

        // versions
        .subcommand(Command::new("versions")
            .about("List all supported WildFly versions"))

        // feature-packs
        .subcommand(Command::new("feature-packs")
            .about("List all supported feature packs"))

        // images
        .subcommand(Command::new("images")
            .about("List all available Neo4J model DB images")
            .arg(Arg::new("wildfly")
                .short('w')
                .long("wildfly")
                .action(ArgAction::SetTrue)
                .help("Show only WildFly versions"))
            .arg(Arg::new("feature-packs")
                .short('f')
                .long("feature-packs")
                .action(ArgAction::SetTrue)
                .help("Show only feature packs")))

        // ps
        .subcommand(Command::new("ps")
            .about("List running Neo4J model DB containers"))

        // browse
        .subcommand(Command::new("browse")
            .about("Open the Neo4J browser for running Neo4J model DBs")
            .arg(Arg::new("identifier")
                .required(true)
                .help("WildFly versions, feature packs, or a mix (e.g. 34,ai,graphql)")))

        // update
        .subcommand(Command::new("update")
            .about("Update WildFly and feature pack configuration files"))

        // completions
        .subcommand(Command::new("completions")
            .about("Generate and install shell completions")
            .arg(Arg::new("shell")
                .help("The shell to generate completions for [default: auto-detected]"))
            .arg(Arg::new("install")
                .short('i')
                .long("install")
                .action(ArgAction::SetTrue)
                .help("Install completions to the standard location for the shell")))
}
