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
mod start;
mod stop;
mod versions;

pub use analyze::analyze;
pub use browse::browse;
pub use completions::completions;
pub use feature_packs::feature_packs_cmd;
pub use images::images;
pub use ps::ps;
pub use push::push;
pub use start::start;
pub use stop::stop;
pub use versions::versions;
