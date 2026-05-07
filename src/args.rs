//! Typed argument extraction helpers for clap `ArgMatches`.

use anyhow::Result;
use clap::ArgMatches;
use wildfly_meta::MetaItem;

/// Extracts a single required `MetaItem` from the `identifier` argument.
pub fn meta_item_argument(matches: &ArgMatches) -> Result<MetaItem> {
    matches
        .get_one::<MetaItem>("identifier")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Required argument 'identifier' not found"))
}

/// Extracts a required list of `MetaItem`s from the `identifier` argument.
pub fn meta_items_argument(matches: &ArgMatches) -> Result<Vec<MetaItem>> {
    matches
        .get_one::<Vec<MetaItem>>("identifier")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Required argument 'identifier' not found"))
}
