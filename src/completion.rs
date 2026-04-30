//! Shell completion candidate generation for WildFly versions and feature packs.

use std::ffi::OsStr;

use clap_complete::engine::CompletionCandidate;
use wildfly_meta::{DslOptions, all_meta_items, suggest_meta_items};

use crate::registry::{images_registry, packs_registry};

/// Returns completions for single-value arguments (versions + feature packs, no ranges).
pub fn complete_single_identifier(_current: &OsStr) -> Vec<CompletionCandidate> {
    all_meta_items(images_registry(), packs_registry())
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

/// Returns completions for multi-value arguments (comma-separated, ranges supported).
pub fn complete_multiple_identifiers(current: &OsStr) -> Vec<CompletionCandidate> {
    let input = current.to_str().unwrap_or("");
    suggest_meta_items(
        input,
        images_registry(),
        packs_registry(),
        &DslOptions::all(),
        &DslOptions::none(),
    )
    .into_iter()
    .map(CompletionCandidate::new)
    .collect()
}
