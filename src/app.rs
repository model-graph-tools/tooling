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
            .about("Analyze the management model of a WildFly instance and build an image with a Neo4J database")
            .arg(Arg::new("identifier")
                .required(true)
                .help("A WildFly version (e.g. 39, 26.1)")))

        // neo4j
        .subcommand(Command::new("neo4j")
            .about("Start and stop a Neo4J model database")

            // start
            .subcommand(Command::new("start")
                .about("Start one or several Neo4J model databases")
                .arg(Arg::new("identifier")
                    .required(true)
                    .help("A WildFly version, version range or a feature pack identifier"))

            // stop
            .subcommand(Command::new("stop")
                .about("Stop one or several Neo4J model databases")
                .arg(Arg::new("identifier")
                    .required_unless_present("all")
                    .help("A WildFly version, version range or a feature pack identifier"))
                .arg(Arg::new("all")
                    .short('a')
                    .long("all")
                    .action(ArgAction::SetTrue)
                    .help("Stop all running Neo4J model databases.")))))
}
