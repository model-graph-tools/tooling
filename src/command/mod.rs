//! Subcommand implementations for the `mgt` CLI.
//!
//! Each subcommand lives in its own module. This module re-exports the public
//! entry points so `main.rs` can dispatch to them directly.

mod analyze;
mod browse;
mod completions;
mod feature_packs;
mod images;
mod ps;
mod push;
mod repack;
mod resolve;
mod start;
mod stop;
mod update;
mod versions;

pub use analyze::analyze;
pub use browse::browse;
pub use completions::completions;
pub use feature_packs::feature_packs_cmd;
pub use images::images;
pub use ps::ps;
pub use push::push;
pub use repack::repack;
pub use resolve::resolve;
pub use start::start;
pub use stop::stop;
pub use update::update;
pub use versions::versions;

use crate::registry::{images_registry, packs_registry};
use wildfly_meta::MetaItem;

/// Resolves the list of `MetaItem`s to process.
///
/// When `all` is true, returns all known versions and feature packs from the
/// registries. Otherwise, returns the explicitly provided items.
fn resolve_targets(items: Option<&[MetaItem]>, all: bool) -> anyhow::Result<Vec<MetaItem>> {
    if all {
        let images = images_registry()?;
        let packs = packs_registry()?;

        let mut targets: Vec<MetaItem> = Vec::new();
        for img in images.all() {
            targets.push(MetaItem::Image(img.clone()));
        }
        for fp in packs.all() {
            targets.push(MetaItem::FeaturePack(fp.clone()));
        }
        Ok(targets)
    } else {
        let provided = items.ok_or_else(|| anyhow::anyhow!("Argument <identifier> expected"))?;
        Ok(provided.to_vec())
    }
}
