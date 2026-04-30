# Migration Guide: `wildfly_container_versions` to `wildfly_meta`

This document describes how to migrate **wado** and **mgt** from the old `wildfly_container_versions` crate (and mgt's dependency on wado) to the new `wildfly_meta` crate.

## Goal

**Before:**

```
wildfly_container_versions
    ├── wado (uses WildFlyContainer, VERSIONS, DEVELOPMENT_TAG)
    └── mgt  (uses WildFlyContainer, VERSIONS)

wado
    └── mgt  (uses AdminContainer, StandaloneInstance, Ports, ServerType)
```

**After:**

```
wildfly_meta
    ├── wado
    └── mgt
```

Both wado and mgt depend only on `wildfly_meta`. mgt no longer depends on wado.

## Type Mapping

| Old (`wildfly_container_versions`) | New (`wildfly_meta`) | Notes |
|---|---|---|
| `WildFlyContainer` | `WildFlyImage` | Same fields, same semantics |
| `VERSIONS` (`BTreeMap<u16, WildFlyContainer>`) | `ImageRegistry` | Dynamic (TOML-loaded) instead of hardcoded. Use `ImageRegistry::load_default()` |
| `DEVELOPMENT_TAG` | `DEVELOPMENT_TAG` | Unchanged |
| `DEVELOPMENT_VERSION` | `DEVELOPMENT_VERSION` | Unchanged |

## API Mapping

### Version Lookup

```rust
// OLD
use wildfly_container_versions::{WildFlyContainer, VERSIONS};

let wc = WildFlyContainer::version("34")?;
let wc = WildFlyContainer::lookup(340)?;
let all = VERSIONS.values();

// NEW
use wildfly_meta::{ImageRegistry, WildFlyImage, parse_image};

let registry = ImageRegistry::load_default()?;
let img = parse_image("34", &registry)?;
let img = registry.get(340);
let all = registry.all();
```

### Version Parsing (DSL)

```rust
// OLD
let containers = WildFlyContainer::enumeration("3x10,23..26,5x28,34,dev")?;
let range = WildFlyContainer::range("20..30")?;

// NEW
use wildfly_meta::{ImageRegistry, FeaturePackRegistry, parse_list, ParseOptions};

let images = ImageRegistry::load_default()?;
let packs = FeaturePackRegistry::load_default()?;
let items = parse_list("3x10,23..26,5x28,34,dev", &images, &packs, &ParseOptions::all())?;
```

The new `parse_list` returns `Vec<MetaItem>` which wraps both `WildFlyImage` and `FeaturePack`. If you only need images, filter or use `parse_image` for single lookups.

### Port Computation

```rust
// OLD
let http = wc.http_port();
let mgmt = wc.management_port();

// NEW (same API on WildFlyImage)
let http = img.http_port();
let mgmt = img.management_port();
```

### Shell Completion

```rust
// OLD (manual iteration over VERSIONS)
let completions: Vec<String> = VERSIONS.values()
    .map(|wc| wc.display_version())
    .collect();

// NEW
use wildfly_meta::{ImageRegistry, FeaturePackRegistry, suggest, CompletionOptions};

let images = ImageRegistry::load_default()?;
let packs = FeaturePackRegistry::load_default()?;
let options = CompletionOptions { feature_packs: true, ranges: true };
let completions = suggest(partial_input, &images, &packs, &options);
```

### Updating Metadata

```rust
// OLD: versions were hardcoded, required crate version bump

// NEW
use wildfly_meta::{update_all, UpdateResult};

let result: UpdateResult = update_all()?;
println!("{}", result.summary());
```

## Cargo.toml Changes

### wado

```toml
# Remove:
wildfly_container_versions = "39.0.5"

# Add:
wildfly_meta = "<latest>"
```

### mgt

```toml
# Remove:
wildfly_container_versions = "39.0.5"
wado = "0.4.12"

# Add:
wildfly_meta = "<latest>"
```

## Registry Initialization

The key architectural difference: `wildfly_container_versions` had a static `VERSIONS` map compiled into the binary. `wildfly_meta` loads data from TOML files at runtime.

Recommended pattern: load the registries once at startup and pass them down.

```rust
use wildfly_meta::{ImageRegistry, FeaturePackRegistry, update_all};

fn main() -> anyhow::Result<()> {
    // Download/update TOML files from GitHub (if needed)
    update_all()?;

    // Load registries (from ~/.config/wildfly-meta/)
    let images = ImageRegistry::load_default()?;
    let packs = FeaturePackRegistry::load_default()?;

    // Pass &images and &packs to subcommands
    // ...
    Ok(())
}
```

## Removing mgt's Dependency on wado

mgt imports four types from wado: `AdminContainer`, `StandaloneInstance`, `Ports`, and `ServerType`. These are container orchestration types, not metadata, so they do **not** belong in `wildfly_meta`.

### What mgt uses from wado

| Type | Usage in mgt |
|---|---|
| `AdminContainer` | `AdminContainer::new(wc, ServerType::Standalone)`, `.image_name()`, `.wildfly_container.identifier` |
| `StandaloneInstance` | `StandaloneInstance::new(ac, name, ports)`, `.name`, `.ports.http`, `.ports.management`, `.admin_container.image_name()` |
| `Ports` | `Ports::default_ports(wc)`, `.http`, `.management`, direct struct construction with offset |
| `ServerType` | Only `ServerType::Standalone` is ever used |

### Replacement Strategy

All of these can be replaced by using `WildFlyImage` directly. mgt should define a small local struct:

```rust
use wildfly_meta::WildFlyImage;

#[derive(Clone)]
struct AnalysisInstance {
    image: WildFlyImage,
    name: String,
    http_port: u16,
    management_port: u16,
}

impl AnalysisInstance {
    fn new(image: WildFlyImage, name: String, http_port: u16, management_port: u16) -> Self {
        Self { image, name, http_port, management_port }
    }

    fn from_image(image: WildFlyImage, name: String) -> Self {
        let http = image.http_port();
        let mgmt = image.management_port();
        Self::new(image, name, http, mgmt)
    }

    fn with_offset(image: WildFlyImage, name: String, offset: u16) -> Self {
        let http = image.http_port() + offset;
        let mgmt = image.management_port() + offset;
        Self::new(image, name, http, mgmt)
    }

    fn image_name(&self) -> String {
        self.image.image_name()
    }
}
```

### Field-by-field replacement in mgt

| Old (wado) | New (wildfly_meta + local struct) |
|---|---|
| `AdminContainer::new(wc, ServerType::Standalone)` | Not needed; use `WildFlyImage` directly |
| `admin_container.image_name()` | `image.image_name()` (same method on `WildFlyImage`) |
| `admin_container.wildfly_container.identifier` | `image.identifier` |
| `Ports::default_ports(&wc)` | `image.http_port()` / `image.management_port()` |
| `StandaloneInstance::new(ac, name, ports)` | `AnalysisInstance::from_image(image, name)` or `::with_offset(image, name, offset)` |
| `instance.name` | `instance.name` |
| `instance.ports.http` | `instance.http_port` |
| `instance.ports.management` | `instance.management_port` |

### Files to change in mgt

1. **`Cargo.toml`** - Remove `wado` and `wildfly_container_versions` deps, add `wildfly_meta`
2. **`src/source.rs`** - Replace `WildFlyContainer` with `WildFlyImage`, replace `WildFlyContainer::version()` with `parse_image()`, replace `WildFlyContainer::enumeration()` with `parse_list()`
3. **`src/completion.rs`** - Replace `VERSIONS` iteration with `ImageRegistry` methods or `suggest()`
4. **`src/command/versions.rs`** - Replace `VERSIONS` with `registry.all()`
5. **`src/command/images.rs`** - Replace `VERSIONS` with `registry.all()`
6. **`src/command/analyze/wildfly.rs`** - Replace `AdminContainer`/`StandaloneInstance`/`Ports`/`ServerType` with local `AnalysisInstance` struct using `WildFlyImage`
7. **`src/command/analyze/runner.rs`** - Update to use `AnalysisInstance` instead of `StandaloneInstance`
8. **`src/command/analyze/cleanup.rs`** - Update to use `AnalysisInstance` instead of `StandaloneInstance`

### Files to change in wado

1. **`Cargo.toml`** - Remove `wildfly_container_versions` dep, add `wildfly_meta`
2. **All files importing `WildFlyContainer`** (14 files) - Replace with `WildFlyImage`
3. **All files importing `VERSIONS`** (5 files) - Replace with `ImageRegistry` methods
4. **`src/wildfly/admin_container.rs`** - Update `AdminContainer` to wrap `WildFlyImage` instead of `WildFlyContainer`
5. **`src/wildfly/instance.rs`** - Update `Ports::default_ports()` to take `&WildFlyImage`
6. **`src/wildfly/management.rs`** - Update `ManagementClient` to use `WildFlyImage`
7. **`src/completion/version.rs`** - Use `ImageRegistry` + `suggest()` for shell completions

## Feature Packs (New Capability)

`wildfly_meta` adds feature pack support that `wildfly_container_versions` did not have. Both wado and mgt can optionally use this for feature pack management. Feature packs are identified by shortcut (e.g., "ai", "graphql") and loaded from `feature-packs.toml`.

## `WildFlyImage` Struct Reference

```rust
pub struct WildFlyImage {
    pub identifier: u16,       // e.g., 340 for WildFly 34.0
    pub version: Version,      // semver::Version
    pub short_version: String, // e.g., "34.0"
    pub core_version: Version, // WildFly Core version
    pub suffix: String,        // e.g., "Final-jdk21"
    pub repository: String,    // e.g., "quay.io/wildfly/wildfly"
    pub platforms: Vec<String>, // e.g., ["linux/amd64", "linux/arm64"]
    // port_offset: u16 (private, used by http_port/management_port)
}
```

**Methods:**

- `image_name() -> String` - Full container image reference (or GitHub URL for dev)
- `is_dev() -> bool` - Whether this is the development/source build
- `display_version() -> String` - "dev" or short_version
- `http_port() -> u16` - HTTP port (8000 + offset)
- `management_port() -> u16` - Management port (9000 + offset)

**Traits:** `Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd`
