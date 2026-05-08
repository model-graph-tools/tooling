# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.5] - 2026-05-08

## [0.3.5] - 2026-05-08

### Added
- Add README documentation to platform-specific npm packages
- Document npm package distribution in project README and CLAUDE.md

## [0.3.4] - 2026-05-08

### Added
- Distribute `mgt` binary via platform-specific npm packages (`@model-graph-tools/mgt-{platform}`)

### Changed
- Replace npm publish shell script with matrix-based GitHub Actions workflow for parallel per-platform publishing

## [0.3.3] - 2026-05-08

### Fixed
- Fix test race condition in registry initialization by using `sync::Once` to prevent concurrent `load_or_update` calls across test threads

## [0.3.2] - 2026-05-08

### Added
- Add `--json` flag support to the `update` command for machine-readable output including update status, version info, and diffs

### Changed
- Replace `expect()` panics in registry accessors and argument extractors with `Result`-based error propagation
- Replace `unreachable!()` with `bail!()` in subcommand dispatch for graceful error handling
- Replace `unwrap()` on JSON serialization with `?` operator in `versions`, `feature-packs`, `resolve`, and `ps` commands
- Log warnings instead of silently ignoring `writeln!` errors in analyzer log streaming
- Add `Debug` derive to `Neo4JImage` and `Neo4JContainer` structs
- Add `#[must_use]` attribute to `Neo4JImage::new()`

## [0.3.1] - 2026-05-07

### Added
- Pass `--stability=experimental` when starting WildFly >= 31 for analysis, ensuring all resources (experimental, preview, community, default) are captured in the model database

### Changed
- Update `wildfly_meta` to 0.7.1

## [0.3.0] - 2026-05-06

### Added
- Structured error codes via `MgtError` enum with 13 typed variants (`CONTAINER_RUNTIME_NOT_FOUND`, `UNKNOWN_IDENTIFIER`, `IMAGE_PULL_FAILED`, etc.) for stable machine-parseable error reporting
- `JsonErrorEnvelope` for top-level JSON error output when `--json` is active — printed to stdout with exit code 1
- `error_code` field on `CommandResult` for per-item error codes in `start`/`stop` JSON output

### Changed
- Replace all `expect()` and `unwrap()` calls in production code with proper error propagation via `?` and `context()`
- Use checked arithmetic for port offset calculation to prevent potential overflow
- Log warnings instead of silencing errors in cleanup and log file deletion paths
- Extract `CommandResult::success()` and `CommandResult::error()` constructors to reduce duplication in `start`/`stop` commands
- Remove unnecessary `clone()` in push command by collecting references instead of owned values
- Extract `PROGRESS_TEXT_MAX_WIDTH` constant for progress bar text truncation
- Add `#[must_use]` attribute to `MgtError` struct
- Add error context with file paths to download operations
- Remove unused `MgtError::internal()` constructor
- Replace all `bail!()` error sites in `container.rs` and `start.rs` with typed `MgtError` constructors carrying parameterized messages
- Restructure `main.rs` to detect `--json` before clap parsing, use `try_get_matches()` to intercept parse errors, and wrap all errors as `JsonErrorEnvelope` in JSON mode
- Classify clap `ValueValidation` errors as `UNKNOWN_IDENTIFIER` for better machine error handling

## [0.2.8] - 2026-05-05

### Changed

- Update `wildfly_meta` to 0.7.0
- Use `image_tag` for container image references instead of constructing from `version` + `suffix`
- Display `release_version` and `core_release_version` in `versions` command output
- Remove `suffix_display()` helper

## [0.2.7] - 2026-05-05

### Changed
- Make registry initialization lazy so `update`, `ps`, `completions`, `--help`, and `--version` work without existing config files
- Add `mgt update` post-install step to the shell install script to bootstrap config on first install

## [0.2.6] - 2026-05-04

### Added
- Add `install.sh` script for quick installation via `curl | sh` with automatic OS and architecture detection
- Add `aarch64-apple-darwin` (Apple Silicon) build target to release workflow

### Changed
- Update Homebrew formula to serve native binaries for both Apple Silicon and Intel Macs via `on_arm`/`on_intel` blocks

## [0.2.5] - 2026-05-02

### Added
- Add `resolve` subcommand to resolve identifiers to their canonical form without starting any containers

## [0.2.4] - 2026-05-01

### Fixed
- Use the original input expression as the identifier in JSON output for `start` and `stop` commands

## [0.2.3] - 2026-05-01

### Changed
- Skip pulling container images that are already available locally to avoid hitting Docker Hub rate limits

## [0.2.2] - 2026-04-30

### Changed
- Pre-select bolt connection URL when opening the Neo4J browser

## [0.2.1] - 2026-04-30

### Added
- Add `trace_progress`, `finish_output`, and `stderr_reader` helpers for streaming child process output to progress spinners

### Fixed
- Fix `push` command hanging indefinitely by streaming container runtime output in real time instead of buffering it

## [0.2.0] - 2026-04-30

### Added
- Add global `--json` flag for machine-readable JSON output on `versions`, `feature-packs`, `ps`, `start`, and `stop` commands
- Add `ContainerInfo` and `CommandResult` serializable types for structured JSON output of container commands
- Add `Progress::hidden()` constructor for silent progress bars in JSON mode

## [0.1.1] - 2026-04-30

### Fixed
- Fix WildFly analysis using upstream WildFly images instead of wado-sa images

## [0.1.0] - 2026-04-30

### Added
- Add `update` subcommand to update WildFly and feature pack configuration files from `wildfly_meta`

### Changed
- Migrate from `wado`/`wildfly_container_versions` to `wildfly_meta` crate for all WildFly image and feature pack metadata
- Upgrade `wildfly_meta` to 0.5.0 with built-in fail-safe handling and synchronous registry initialization

### Fixed
- Use synchronous registry initialization in tests to prevent race conditions

## [0.0.2] - 2026-04-28

### Added
- Add `push` subcommand to push Neo4J model DB images to quay.io with parallel execution, progress tracking, and optional `--chunks` flag for batched pushes
- Add `source-type` and `source-name` container labels for reliable container listing
- Serve the welcome page locally via nginx reverse proxy with source info

### Fixed
- Fix gear emoji (COG) using correct Unicode codepoint `U+2699 U+FE0F`
- Fix feature pack port conflicts by raising port offset base from 1,000 to 10,000, preventing bolt port collisions with WildFly HTTP ports
- Pre-pull container images (Neo4J, WildFly, analyzer JRE) before starting containers to prevent healthcheck timeouts on first run
- Replace dots with dashes in feature pack container IDs
- Increase health check retries to 60
- Fix feature pack welcome labels using correct casing (AI, GraphQL, gRPC, MyFaces) via new `name` field

### Changed
- Build multi-arch manifest images (linux/amd64, linux/arm64) instead of single-arch images during analysis
- Extend WildFly configuration mapping to cover all versions from 10.0 to 39.0, adding `standalone-microprofile.xml` analysis for versions 19.0+

## [0.0.1] - 2026-04-28

- First release 🎉

[Unreleased]: https://github.com/model-graph-tools/tooling/compare/v0.3.5...HEAD
[0.3.5]: https://github.com/model-graph-tools/tooling/compare/v0.3.4...v0.3.5
[0.3.5]: https://github.com/model-graph-tools/tooling/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/model-graph-tools/tooling/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/model-graph-tools/tooling/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/model-graph-tools/tooling/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/model-graph-tools/tooling/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/model-graph-tools/tooling/compare/v0.2.8...v0.3.0
[0.2.8]: https://github.com/model-graph-tools/tooling/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/model-graph-tools/tooling/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/model-graph-tools/tooling/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/model-graph-tools/tooling/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/model-graph-tools/tooling/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/model-graph-tools/tooling/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/model-graph-tools/tooling/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/model-graph-tools/tooling/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/model-graph-tools/tooling/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/model-graph-tools/tooling/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/model-graph-tools/tooling/compare/v0.0.2...v0.1.0
[0.0.2]: https://github.com/model-graph-tools/tooling/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/model-graph-tools/tooling/releases/tag/v0.0.1
