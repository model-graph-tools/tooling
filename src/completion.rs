//! Shell completion candidate generation for WildFly versions and feature packs.

use std::ffi::OsStr;

use clap_complete::engine::CompletionCandidate;
use wildfly_meta::{DslOptions, all_meta_items, suggest_meta_items};

use crate::registry::{images_registry, packs_registry};

/// Returns completions for single-value arguments (versions + feature packs, no ranges).
pub fn complete_single_identifier(_current: &OsStr) -> Vec<CompletionCandidate> {
    let (Ok(images), Ok(packs)) = (images_registry(), packs_registry()) else {
        return vec![];
    };
    all_meta_items(images, packs)
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

/// Returns completions for multi-value arguments (comma-separated, ranges supported).
pub fn complete_multiple_identifiers(current: &OsStr) -> Vec<CompletionCandidate> {
    let input = current.to_str().unwrap_or("");
    let (Ok(images), Ok(packs)) = (images_registry(), packs_registry()) else {
        return vec![];
    };
    suggest_meta_items(
        input,
        images,
        packs,
        &DslOptions::all(),
        &DslOptions::none(),
    )
    .into_iter()
    .map(CompletionCandidate::new)
    .collect()
}
