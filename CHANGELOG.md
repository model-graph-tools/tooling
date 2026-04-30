# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/model-graph-tools/tooling/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/model-graph-tools/tooling/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/model-graph-tools/tooling/releases/tag/v0.0.1
