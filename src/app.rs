use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use clap::{Arg, ArgAction, Command, crate_name, crate_version};

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
                .help("A WildFly version (e.g. 39, 26.1) or feature pack (e.g. cloud:9.0.0.Final)")))

        // start
        .subcommand(Command::new("start")
            .about("Start Neo4J model DB containers")
            .arg(Arg::new("identifier")
                .required(true)
                .help("WildFly versions, feature packs, or a mix (e.g. 34,cloud:9.0.0.Final)")))

        // stop
        .subcommand(Command::new("stop")
            .about("Stop Neo4J model DB containers")
            .arg(Arg::new("identifier")
                .required_unless_present("all")
                .help("WildFly versions, feature packs, or a mix (e.g. 34,cloud:9.0.0.Final)"))
            .arg(Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Stop all running Neo4J model DB containers")))

        // ps
        .subcommand(Command::new("ps")
            .about("List running Neo4J model DB containers"))

        // browse
        .subcommand(Command::new("browse")
            .about("Open the Neo4J browser for a running Neo4J model DB")
            .arg(Arg::new("identifier")
                .required(true)
                .help("A WildFly version (e.g. 39, 26.1) or feature pack (e.g. cloud:9.0.0.Final)")))

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
