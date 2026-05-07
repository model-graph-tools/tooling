//! Global registry initialization for WildFly images and feature packs.
//!
//! Uses `OnceLock` to make registries accessible from clap value parsers
//! and completers, which require `Fn(&str) -> Result<T>` signatures
//! with no captured state.

use std::sync::OnceLock;

use anyhow::Result;
use wildfly_meta::{FeaturePackRegistry, WildFlyImageRegistry};

static IMAGES: OnceLock<WildFlyImageRegistry> = OnceLock::new();
static PACKS: OnceLock<FeaturePackRegistry> = OnceLock::new();

const RESOLUTION_HINT: &str = "Run 'mgt update' to download the configuration files";

/// Loads registries from TOML config files, downloading them first if missing.
///
/// Runs on a blocking thread to avoid panics from `reqwest::blocking` inside the tokio runtime.
pub async fn init_registries() -> Result<()> {
    tokio::task::spawn_blocking(|| {
        let images = WildFlyImageRegistry::load_or_update(RESOLUTION_HINT)?;
        let packs = FeaturePackRegistry::load_or_update(RESOLUTION_HINT)?;
        IMAGES.set(images).ok();
        PACKS.set(packs).ok();
        Ok(())
    })
    .await?
}

/// Synchronous version of registry initialization for use in tests.
#[cfg(test)]
pub fn init_registries_sync() -> Result<()> {
    let images = WildFlyImageRegistry::load_or_update(RESOLUTION_HINT)?;
    let packs = FeaturePackRegistry::load_or_update(RESOLUTION_HINT)?;
    IMAGES.set(images).ok();
    PACKS.set(packs).ok();
    Ok(())
}

pub fn images_registry() -> Result<&'static WildFlyImageRegistry> {
    IMAGES
        .get()
        .ok_or_else(|| anyhow::anyhow!("WildFly image registry not initialized"))
}

pub fn packs_registry() -> Result<&'static FeaturePackRegistry> {
    PACKS
        .get()
        .ok_or_else(|| anyhow::anyhow!("Feature pack registry not initialized"))
}
