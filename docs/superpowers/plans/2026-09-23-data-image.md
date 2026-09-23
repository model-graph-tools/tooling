# Data Image Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce a separate OCI data image (`quay.io/modelgraphtools/data:<tag>`) to decouple Neo4J database files from the model image, fixing the VOLUME-related `podman cp` failure in repack.

**Architecture:** The analyze pipeline extracts database files from the stopped Neo4J container and builds a single-arch data image (`FROM scratch`). Both analyze and repack then build the model image using a multi-stage Dockerfile that copies data via `COPY --from=<data-image>`. Push sends both data and model images.

**Tech Stack:** Rust, podman/docker CLI, OCI images, quay.io registry

**Spec:** `docs/superpowers/specs/2026-09-23-data-image-design.md`

## Global Constraints

- Data image base: `FROM scratch` (no runtime)
- Data image architecture: single-arch (`linux/amd64`) — content is platform-independent
- Data image repository: `quay.io/modelgraphtools/data`
- Model image repository: `quay.io/modelgraphtools/model` (unchanged)
- Tags use `major.minor.patch` for WildFly versions, `shortcut-version` for feature packs (unchanged)
- Model image remains fully self-contained (data copied at build time)
- Container runtime: podman preferred, docker fallback (unchanged)

## Review Focus

1. **Missing data image on repack:** If `mgt repack 41` is run before `mgt analyze 41` (or before the data image is pushed), the data image pull will fail. The error message from `pull_image` should clearly indicate which data image is missing. Tested in Task 3.
2. **Data image tag mismatch between analyze and repack:** If `data_image_tag()` and `image_tag()` use inconsistent tag derivation, repack will reference a nonexistent data image. Covered by unit tests in Task 1.
3. **Multi-stage COPY --from with multi-arch model build:** The model image is built as a multi-arch manifest (`--platform linux/amd64,linux/arm64`), but `COPY --from` references a single-arch data image. Podman resolves this correctly since it pulls the data image once and copies from it for each platform build. Verified by manual end-to-end test.
4. **Push ordering:** If the model image is pushed but the data image is not, a consumer pulling the model image and inspecting its Dockerfile history would see a reference to a data image that doesn't exist on the registry. Both must be pushed together. Covered in Task 4.
5. **Stale data image after re-analyze:** If `mgt analyze 41` is run twice, the data image should be overwritten locally. The `build_data_image` step tags the new image, replacing the old one. No special handling needed.

---

### Task 1: Add data image support to constants and Neo4JImage

**Files:**
- Modify: `src/constants.rs:4` (add `DATA_REPOSITORY`)
- Modify: `src/neo4j.rs:44-70` (add `data_image_tag()`, update `model_db_dockerfile` signature)

**Interfaces:**
- Consumes: `MODEL_GRAPH_TOOLS_REPOSITORY` from `constants.rs`
- Produces:
  - `pub static DATA_REPOSITORY: &str` in `constants.rs`
  - `pub fn data_image_tag(&self) -> String` on `Neo4JImage`
  - `fn model_db_dockerfile(source_name: &str, rest_api_version: &str, data_image_tag: &str) -> String` (updated signature)
  - `fn data_image_dockerfile() -> String` (module-level helper)

- [ ] **Step 1: Write failing tests for `data_image_tag`**

Add to the `#[cfg(test)] mod tests` block in `src/neo4j.rs`. These tests need access to the registries (like the tests in `command/analyze/wildfly.rs`). Add the registry init pattern and use `parse_wildfly_image` / `parse_feature_pack` to get real `MetaItem` values:

```rust
use crate::registry::init_registries_sync;
use std::sync::Once;
use wildfly_meta::{parse_wildfly_image, parse_feature_pack};

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        init_registries_sync().expect("Failed to initialize registries");
    });
}

#[test]
fn data_image_tag_wildfly() {
    init();
    let registry = crate::registry::images_registry().unwrap();
    let img = parse_wildfly_image("41", registry).unwrap();
    let image = Neo4JImage::new(&MetaItem::Image(img));
    // The tag uses the full version from the registry (e.g. "41.0.1")
    assert!(image.data_image_tag().starts_with("quay.io/modelgraphtools/data:41.0"));
}

#[test]
fn data_image_tag_feature_pack() {
    init();
    let registry = crate::registry::packs_registry().unwrap();
    let fp = parse_feature_pack("ai", registry).unwrap();
    let image = Neo4JImage::new(&MetaItem::FeaturePack(fp));
    assert!(image.data_image_tag().starts_with("quay.io/modelgraphtools/data:ai-"));
}
```

Note: The exact version suffix (e.g. `41.0.1` vs `41.0.2`) depends on the registry data. Use `starts_with` to assert the structure without hardcoding the patch version. If `parse_feature_pack` doesn't exist, use `wildfly_meta::parse_meta_item("ai", registry_images, registry_packs)` and match on the `MetaItem::FeaturePack` variant — check the `wildfly_meta` crate's public API.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test data_image_tag`
Expected: FAIL — method `data_image_tag` does not exist

- [ ] **Step 3: Add `DATA_REPOSITORY` constant**

In `src/constants.rs`, after the `MODEL_GRAPH_TOOLS_REPOSITORY` line (line 4), add:

```rust
/// Container image registry for Neo4J database data images.
pub static DATA_REPOSITORY: &str = "quay.io/modelgraphtools/data";
```

- [ ] **Step 4: Add `data_image_tag()` method**

In `src/neo4j.rs`, inside the `impl Neo4JImage` block, after `image_tag()` (after line 69), add:

```rust
/// Returns the tagged data image name on quay.io for this source.
pub fn data_image_tag(&self) -> String {
    match &self.item {
        MetaItem::Image(img) => {
            format!("{}:{}", DATA_REPOSITORY, img.version)
        }
        MetaItem::FeaturePack(fp) => {
            format!(
                "{}:{}-{}",
                DATA_REPOSITORY, fp.shortcut, fp.version
            )
        }
    }
}
```

Add `use crate::constants::DATA_REPOSITORY;` to the imports at the top of `neo4j.rs` (alongside the existing `MODEL_GRAPH_TOOLS_REPOSITORY` import).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test data_image_tag`
Expected: PASS

- [ ] **Step 6: Write failing test for updated `model_db_dockerfile`**

Update the existing `dockerfile_contains_targetarch` test (and all other `model_db_dockerfile` tests) to pass the new `data_image_tag` parameter. Add a new test:

```rust
#[test]
fn dockerfile_has_multistage_data_from() {
    let df = model_db_dockerfile("wildfly-41.0", "0.1.0", "quay.io/modelgraphtools/data:41.0.1");
    assert!(df.starts_with("FROM quay.io/modelgraphtools/data:41.0.1 AS data\n"));
    assert!(df.contains("COPY --from=data /databases /data/databases"));
    assert!(df.contains("COPY --from=data /transactions /data/transactions"));
}
```

Update ALL existing calls to `model_db_dockerfile` in the test module to pass a third argument `"quay.io/modelgraphtools/data:41.0.1"`.

- [ ] **Step 7: Run tests to verify they fail**

Run: `cargo test model_db_dockerfile`
Expected: FAIL — wrong number of arguments

- [ ] **Step 8: Update `model_db_dockerfile` signature and body**

In `src/neo4j.rs`, change the function signature from:

```rust
fn model_db_dockerfile(source_name: &str, rest_api_version: &str) -> String {
```

to:

```rust
fn model_db_dockerfile(source_name: &str, rest_api_version: &str, data_image_tag: &str) -> String {
```

Update the format string:
- Add `FROM {data_image_tag} AS data\n` as the very first line (before `FROM neo4j:...`)
- Replace `COPY --chown=neo4j:neo4j databases /data/databases` with `COPY --from=data --chown=neo4j:neo4j /databases /data/databases`
- Replace `COPY --chown=neo4j:neo4j transactions /data/transactions` with `COPY --from=data --chown=neo4j:neo4j /transactions /data/transactions`

- [ ] **Step 9: Add `data_image_dockerfile` helper**

In `src/neo4j.rs`, after `model_db_dockerfile`, add:

```rust
fn data_image_dockerfile() -> String {
    "FROM scratch\nCOPY databases /databases\nCOPY transactions /transactions\n".to_string()
}
```

- [ ] **Step 10: Run all tests to verify they pass**

Run: `cargo test`
Expected: PASS (some tests may fail due to `build_image` callers not yet updated — that's expected and fixed in Tasks 2-3)

- [ ] **Step 11: Verify the build compiles**

Run: `cargo build`
Expected: Compiler errors in `build_image` callers (`cleanup.rs`, `repack.rs`) due to changed signature. This is expected — those are fixed in Tasks 2 and 3.

Note: If the compiler errors prevent `cargo test` from running the unit tests in step 10, temporarily update the `build_image` call in `neo4j.rs` to pass `&self.data_image_tag()` as the third arg to `model_db_dockerfile`, and leave the `container_name` parameter for now. Tasks 2 and 3 will complete the refactoring.

- [ ] **Step 12: Commit**

```bash
git add src/constants.rs src/neo4j.rs
git commit -m "feat: add data image tag and multi-stage Dockerfile support"
```

---

### Task 2: Refactor analyze pipeline to build data image

**Files:**
- Modify: `src/neo4j.rs:72-123` (split `build_image` into `build_data_image` + updated `build_image`)
- Modify: `src/command/analyze/cleanup.rs:15-33` (`build_neo4j_image` calls both new methods)

**Interfaces:**
- Consumes:
  - `Neo4JImage::data_image_tag(&self) -> String` (from Task 1)
  - `data_image_dockerfile() -> String` (from Task 1)
  - `model_db_dockerfile(source_name, rest_api_version, data_image_tag) -> String` (from Task 1)
  - `run_container_cmd(args, error_context) -> Result<()>` from `container.rs`
  - `copy_from_container(container, src, dest) -> Result<()>` from `neo4j.rs` (temporarily retained for analyze)
- Produces:
  - `pub async fn build_data_image(&self, container_name: &str, progress: &Progress) -> Result<()>` on `Neo4JImage`
  - Updated `pub async fn build_image(&self, rest_api_version: &str, progress: &Progress) -> Result<()>` on `Neo4JImage` (no `container_name` param)

- [ ] **Step 1: Refactor `build_image` into `build_data_image` + `build_image`**

In `src/neo4j.rs`, replace the current `build_image` method with two methods:

```rust
/// Copies database files from a stopped container and builds a single-arch data image.
pub async fn build_data_image(
    &self,
    container_name: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    let build_dir = tempfile::tempdir()?;
    let build_path = build_dir.path();

    progress.show_progress("Copying database files...");
    copy_from_container(container_name, "/data/databases", build_path).await?;
    copy_from_container(container_name, "/data/transactions", build_path).await?;

    std::fs::write(build_path.join("Dockerfile"), data_image_dockerfile())?;

    let data_tag = self.data_image_tag();
    progress.show_progress("Building data image...");
    let build_path_str = build_path.to_string_lossy();
    run_container_cmd(
        &["build", "--tag", &data_tag, &build_path_str],
        "Data image build failed",
    )
    .await
}

/// Builds a multi-arch model image using a multi-stage Dockerfile that
/// copies data from the data image via COPY --from.
pub async fn build_image(
    &self,
    rest_api_version: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    let build_dir = tempfile::tempdir()?;
    let build_path = build_dir.path();

    std::fs::write(
        build_path.join("Dockerfile"),
        model_db_dockerfile(
            &self.item.full_name(),
            rest_api_version,
            &self.data_image_tag(),
        ),
    )?;

    let image_tag = self.image_tag();

    progress.show_progress("Creating manifest...");
    let mut rm_cmd = container_command()?;
    rm_cmd
        .arg("manifest")
        .arg("rm")
        .arg(&image_tag)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let _ = rm_cmd.output().await;

    run_container_cmd(
        &["manifest", "create", &image_tag],
        "Manifest creation failed",
    )
    .await?;

    progress.show_progress("Building image...");
    let build_path_str = build_path.to_string_lossy();
    run_container_cmd(
        &[
            "build",
            "--platform",
            PLATFORMS,
            "--manifest",
            &image_tag,
            &build_path_str,
        ],
        "Image build failed",
    )
    .await
}
```

- [ ] **Step 2: Update `build_neo4j_image` in `cleanup.rs`**

In `src/command/analyze/cleanup.rs`, update `build_neo4j_image` to call both methods sequentially:

```rust
pub(super) async fn build_neo4j_image(neo4j: &Neo4JContainer) -> anyhow::Result<()> {
    step_header(3, TOTAL_STEPS, "Building Neo4J image...");
    let progress = Progress::new(&neo4j.image.image_tag());

    progress.show_progress("Stopping neo4j...");
    stop_container(&neo4j.container_name()).await?;

    neo4j
        .image
        .build_data_image(&neo4j.container_name(), &progress)
        .await?;

    neo4j
        .image
        .build_image(crate::constants::REST_API_VERSION, &progress)
        .await?;

    progress.finish_success(Some("Ready"));
    Ok(())
}
```

- [ ] **Step 3: Verify the build compiles**

Run: `cargo build`
Expected: Compiler error in `repack.rs` because `build_image` signature changed (no `container_name` param). This is expected — fixed in Task 3.

- [ ] **Step 4: Run unit tests**

Run: `cargo test`
Expected: Unit tests PASS (same caveat as above — repack.rs may cause compile failure; if so, temporarily comment out the `repack_image` function body with `todo!()` to verify the analyze-side tests pass)

- [ ] **Step 5: Commit**

```bash
git add src/neo4j.rs src/command/analyze/cleanup.rs
git commit -m "feat: analyze pipeline builds separate data image"
```

---

### Task 3: Simplify repack to use data image

**Files:**
- Modify: `src/command/repack.rs` (replace container lifecycle with data image pull + build_image)
- Modify: `src/neo4j.rs` (remove `copy_from_container` if no longer used)

**Interfaces:**
- Consumes:
  - `Neo4JImage::data_image_tag(&self) -> String` (from Task 1)
  - `Neo4JImage::build_image(&self, rest_api_version: &str, progress: &Progress) -> Result<()>` (from Task 2)
  - `pull_image(image: &str, progress: &Progress) -> Result<()>` from `container.rs`
- Produces: Simplified `repack_image` function (no container lifecycle)

- [ ] **Step 1: Rewrite `repack_image`**

Replace the entire `repack_image` function in `src/command/repack.rs` with:

```rust
async fn repack_image(
    image: &Neo4JImage,
    rest_api_version: &str,
    progress: &Progress,
) -> anyhow::Result<()> {
    let data_tag = image.data_image_tag();
    pull_image(&data_tag, progress).await?;
    image.build_image(rest_api_version, progress).await
}
```

- [ ] **Step 2: Clean up imports in `repack.rs`**

Update the imports to remove everything that's no longer needed:

```rust
use crate::constants::{REST_API_VERSION, latest_rest_api_version};
use crate::container::{pull_image, verify_container_command};
use crate::neo4j::Neo4JImage;
use crate::progress::{CommandStatus, Progress, done, summary};
use crate::registry::{images_registry, packs_registry};
use anyhow::bail;
use console::style;
use indicatif::MultiProgress;
use tokio::task::JoinSet;
use tokio::time::Instant;
use wildfly_meta::MetaItem;
```

Removed: `container_command`, `remove_container`, `stop_container` from container imports. Removed: `MgtError` from error imports.

- [ ] **Step 3: Remove `copy_from_container` from `neo4j.rs` if unused**

Check if `copy_from_container` is still used. It should only be called from `build_data_image` now. If that's the case, keep it. If the analyze pipeline's `build_data_image` is the sole caller, it stays.

Run: `cargo build`
Expected: PASS — no compile errors

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Manual verification of repack**

Run: `cargo run -- repack 41 --api-version=0.2.1`
Expected: Repack succeeds by pulling the data image and building the model image. No container start/stop in the output — you should see "Pulling data image..." or similar progress, then "Building image..." with no "Starting temporary container" or "Cleaning up temporary container" messages.

Note: This requires that `mgt analyze 41` has been run first (to create the data image locally). If the data image doesn't exist locally or on the registry, the pull will fail with a clear error from `pull_image`.

- [ ] **Step 6: Commit**

```bash
git add src/command/repack.rs src/neo4j.rs
git commit -m "feat: repack uses data image instead of container extraction"
```

---

### Task 4: Update push to handle both data and model images

**Files:**
- Modify: `src/command/push.rs` (push data image alongside model image)

**Interfaces:**
- Consumes:
  - `Neo4JImage::data_image_tag(&self) -> String` (from Task 1)
  - `Neo4JImage::image_tag(&self) -> String` (existing)
  - `local_image_names() -> Result<HashSet<String>>` from `container.rs`
  - `container_command() -> Result<Command>` from `container.rs`
- Produces: Updated `push` and `push_batch` that push both image types

- [ ] **Step 1: Update the local-availability check in `push`**

In `src/command/push.rs`, update the `pushable` filter to check for BOTH images. Replace the filter closure:

```rust
let pushable: Vec<&MetaItem> = items
    .iter()
    .filter(|item| {
        let image = Neo4JImage::new(item);
        let model_tag = image.image_tag();
        let data_tag = image.data_image_tag();
        let model_ok = local.contains(&model_tag);
        let data_ok = local.contains(&data_tag);
        if !model_ok {
            eprintln!(
                "  {} {} not found locally, skipping",
                style("\u{26a0}").yellow(),
                style(&model_tag).cyan()
            );
        }
        if !data_ok {
            eprintln!(
                "  {} {} not found locally, skipping",
                style("\u{26a0}").yellow(),
                style(&data_tag).cyan()
            );
        }
        model_ok && data_ok
    })
    .collect();
```

- [ ] **Step 2: Update `push_batch` to push both images**

In `push_batch`, for each item spawn TWO tasks — one for the data image (`push`) and one for the model image (`manifest push`):

```rust
async fn push_batch(items: &[&MetaItem]) -> anyhow::Result<Vec<CommandStatus>> {
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    for item in items {
        let image = Neo4JImage::new(item);
        let display = item.short_name();

        // Push data image (regular push, single-arch)
        let data_tag = image.data_image_tag();
        let data_display = format!("{} (data)", display);
        let data_progress = Progress::join(&multi_progress, &data_display);
        let mut data_cmd = container_command()?;
        data_cmd
            .arg("push")
            .arg(&data_tag)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut data_child = data_cmd.spawn()?;
        let data_stderr = stderr_reader(&mut data_child)?;
        let data_progress_clone = data_progress.clone();
        tasks.spawn(async move {
            let output = data_child.wait_with_output().await;
            data_progress.finish_output(output, None)
        });
        tokio::spawn(async move {
            data_progress_clone.trace_progress(data_stderr).await;
        });

        // Push model image (manifest push, multi-arch)
        let model_tag = image.image_tag();
        let model_display = format!("{} (model)", display);
        let model_progress = Progress::join(&multi_progress, &model_display);
        let mut model_cmd = container_command()?;
        model_cmd
            .arg("manifest")
            .arg("push")
            .arg(&model_tag)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut model_child = model_cmd.spawn()?;
        let model_stderr = stderr_reader(&mut model_child)?;
        let model_progress_clone = model_progress.clone();
        tasks.spawn(async move {
            let output = model_child.wait_with_output().await;
            model_progress.finish_output(output, None)
        });
        tokio::spawn(async move {
            model_progress_clone.trace_progress(model_stderr).await;
        });
    }

    Ok(tasks.join_all().await)
}
```

- [ ] **Step 3: Update the count display in `push`**

The count of items stays the same (number of identifiers), but the summary now includes twice as many statuses (data + model per identifier). Update the `summary` call:

```rust
// After push_batch returns, the status vec has 2 entries per item
// Keep using count (number of identifiers) for the header, but
// summary uses the full status list
```

Actually, the `summary` function counts successes and failures from the status vec. Since each identifier now produces two statuses, update the summary call to reflect the total number of push operations:

Replace `summary(count, &all_status)` / `summary(count, &status)` with:

```rust
summary(all_status.len(), &all_status);
// or
summary(status.len(), &status);
```

And update the header to say "images" generically:

```rust
println!(
    "\n{}",
    style(format!("Pushing {} Neo4J model DB {}", count, noun)).bold()
);
```

This line stays the same — it counts identifiers. The summary handles the per-image detail.

- [ ] **Step 4: Verify build compiles**

Run: `cargo build`
Expected: PASS

- [ ] **Step 5: Run all tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/command/push.rs
git commit -m "feat: push data and model images together"
```

---

### Task 5: Clean up and final verification

**Files:**
- Modify: `src/neo4j.rs` (verify `copy_from_container` is only used by `build_data_image`)
- No new files

**Interfaces:**
- Consumes: All prior tasks
- Produces: Clean build, all tests pass, end-to-end verified

- [ ] **Step 1: Verify no dead code**

Run: `cargo clippy -- -W dead_code`
Expected: No warnings about unused functions. `copy_from_container` should be used by `build_data_image` only.

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 3: Run clippy**

Run: `cargo clippy`
Expected: No warnings

- [ ] **Step 4: Run formatter**

Run: `cargo fmt`
Expected: No changes (code already formatted)

- [ ] **Step 5: End-to-end manual test (analyze + repack)**

If a WildFly version is available for analysis:

```bash
cargo run -- analyze 41
cargo run -- repack 41 --api-version=0.2.1
```

Verify:
1. `analyze` produces both `quay.io/modelgraphtools/data:41.0.1` and `quay.io/modelgraphtools/model:41.0.1` locally
2. `repack` pulls the data image (already local), builds a new model image — no container lifecycle in output
3. Both images appear in `podman images | grep modelgraphtools`

- [ ] **Step 6: Commit any cleanup**

```bash
git add -A
git commit -m "chore: clean up dead code after data image refactoring"
```

Only commit if there were actual changes from cleanup. Skip if no changes.
