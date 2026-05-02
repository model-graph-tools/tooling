# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/model-graph-tools/tooling/compare/v0.2.5...HEAD
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
