//! Typed argument extraction helpers for clap `ArgMatches`.

use clap::ArgMatches;
use wildfly_meta::MetaItem;

/// Extracts a single required `MetaItem` from the `identifier` argument.
pub fn meta_item_argument(matches: &ArgMatches) -> MetaItem {
    matches
        .get_one::<MetaItem>("identifier")
        .expect("Argument <identifier> expected!")
        .clone()
}

/// Extracts a required list of `MetaItem`s from the `identifier` argument.
pub fn meta_items_argument(matches: &ArgMatches) -> Vec<MetaItem> {
    matches
        .get_one::<Vec<MetaItem>>("identifier")
        .expect("Argument <identifier> expected!")
        .clone()
}
