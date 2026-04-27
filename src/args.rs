use crate::source::Source;
use clap::ArgMatches;

pub fn source_argument(matches: &ArgMatches) -> Source {
    matches
        .get_one::<Source>("identifier")
        .expect("Argument <identifier> expected!")
        .clone()
}

pub fn sources_argument(matches: &ArgMatches) -> Vec<Source> {
    matches
        .get_one::<Vec<Source>>("identifier")
        .expect("Argument <identifier> expected!")
        .clone()
}
