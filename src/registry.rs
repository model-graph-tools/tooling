//! Global registry initialization for WildFly images and feature packs.
//!
//! Uses `OnceLock` to make registries accessible from clap value parsers
//! and completers, which require `Fn(&str) -> Result<T>` signatures
//! with no captured state.

use std::sync::OnceLock;

use anyhow::Result;
use wildfly_meta::{
    FeaturePackRegistry, WildFlyImageRegistry, feature_packs_path, update_all,
    wildfly_images_path,
};

static IMAGES: OnceLock<WildFlyImageRegistry> = OnceLock::new();
static PACKS: OnceLock<FeaturePackRegistry> = OnceLock::new();

/// Loads registries from TOML config files, downloading them first if missing.
///
/// Handles stale config files from older wildfly_meta versions by deleting
/// them and re-downloading.
pub fn init_registries() -> Result<()> {
    let images = load_or_update(
        WildFlyImageRegistry::load_default,
        wildfly_images_path(),
    )?;
    let packs = load_or_update(
        FeaturePackRegistry::load_default,
        feature_packs_path(),
    )?;
    IMAGES.set(images).ok();
    PACKS.set(packs).ok();
    Ok(())
}

fn load_or_update<T>(load: fn() -> Result<T>, path: std::path::PathBuf) -> Result<T> {
    if let Ok(reg) = load() {
        return Ok(reg);
    }
    match update_all() {
        Ok(_) => load(),
        Err(_) => {
            let _ = std::fs::remove_file(&path);
            update_all()?;
            load()
        }
    }
}

pub fn images_registry() -> &'static WildFlyImageRegistry {
    IMAGES.get().expect("WildFlyImageRegistry not initialized")
}

pub fn packs_registry() -> &'static FeaturePackRegistry {
    PACKS.get().expect("FeaturePackRegistry not initialized")
}
