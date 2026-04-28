//! Typed argument extraction helpers for clap `ArgMatches`.

use crate::source::Source;
use clap::ArgMatches;

/// Extracts a single required `Source` from the `identifier` argument.
pub fn source_argument(matches: &ArgMatches) -> Source {
    matches
        .get_one::<Source>("identifier")
        .expect("Argument <identifier> expected!")
        .clone()
}

/// Extracts a required list of `Source`s from the `identifier` argument.
pub fn sources_argument(matches: &ArgMatches) -> Vec<Source> {
    matches
        .get_one::<Vec<Source>>("identifier")
        .expect("Argument <identifier> expected!")
        .clone()
}
