# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`mgt` is a Rust CLI tool that orchestrates the analysis of the WildFly management model. It automates the pipeline of starting WildFly containers, spinning up Neo4J databases, running the Java-based [analyzer](https://github.com/model-graph-tools/analyzer), and building self-contained Neo4J images. Part of the [model graph tools](https://model-graph-tools.github.io/) ecosystem.

## Build Commands

```shell
cargo build              # debug build
cargo build --release    # release build (uses LTO)
cargo run -- analyze 34  # run with arguments
cargo test               # run tests
cargo clippy             # lint
cargo fmt                # format
```

The binary is named `mgt`. Rust edition 2024.

## Architecture

The codebase is organized into top-level modules and a `command/` submodule tree:

### Top-Level Modules

- **`main.rs`** — Entry point. Parses CLI args via clap, dispatches to subcommands.
- **`app.rs`** — Defines the clap `Command` tree (subcommands, args, styling). Separated from `main.rs` so both the runtime and the completion system can build the command tree independently.
- **`args.rs`** — Helpers to extract typed arguments from clap `ArgMatches`.
- **`command/`** — Subcommand implementations (see below).
- **`constants.rs`** — Analyzer version/URL, Neo4J version/image, and Dockerfile generation.
- **`container.rs`** — Container runtime abstraction. Prefers `podman`, falls back to `docker`.
- **`neo4j.rs`** — Neo4J container, image, and port management.
- **`source.rs`** — Unified abstraction over WildFly versions and feature packs as analysis sources.
- **`feature_pack.rs`** — Feature pack definitions (name, shortcut, GAV coordinates).
- **`download.rs`** — HTTP download helpers (analyzer JAR).
- **`progress.rs`** — Progress bar and status display utilities.
- **`completion.rs`** — Dynamic shell completion logic for identifiers.
- **`label.rs`** — Display formatting for sources.

### Command Submodules (`src/command/`)

- **`analyze/`** — Core analysis pipeline, split into focused files:
  - `mod.rs` — Entry point, parallel orchestration across sources.
  - `runner.rs` — Per-source analysis workflow (start containers → run analyzer → build image → cleanup).
  - `wildfly.rs` — WildFly container start/stop for analysis.
  - `neo4j_ops.rs` — Neo4J container start/stop for analysis.
  - `feature_pack.rs` — Feature pack resolution and container setup.
  - `cleanup.rs` — Resource cleanup after analysis.
- **`push.rs`** — Push Neo4J model DB images to quay.io.
- **`start.rs`** — Start Neo4J model DB containers.
- **`stop.rs`** — Stop Neo4J model DB containers.
- **`browse.rs`** — Open Neo4J browser for running containers.
- **`versions.rs`** — List supported WildFly versions.
- **`feature_packs.rs`** — List supported feature packs.
- **`images.rs`** — List locally available Neo4J model DB images.
- **`ps.rs`** — List running Neo4J model DB containers.
- **`completions.rs`** — Shell completion generation and installation.

## Key Dependencies

- **`wado`** / **`wildfly_container_versions`** — WildFly container abstractions (images, ports, version parsing).
- **`clap`** / **`clap_complete`** — CLI argument parsing with dynamic shell completions.
- **`tokio`** — Async runtime for concurrent container orchestration.
- **`reqwest`** — Downloads the analyzer JAR from GitHub releases.

## Runtime Requirements

- `podman` or `docker` must be available on PATH
- Network access to pull container images and download the analyzer JAR

## Identifiers

Commands accept WildFly versions, feature pack shortcuts, or a mix. WildFly versions are specified as `<major>[.<minor>]` where major >= 10 and minor 0-9. Supports comma-separated lists (`10,26.1,34`) and ranges (`20..29`). Feature packs use shortcut names (e.g. `ai`, `graphql`).

## Release Process

Uses `cargo-release` with `release.toml`. Run `./release.sh <version>` to bump version, update changelog, tag, and push. The GitHub Actions release workflow then creates a GitHub release, publishes to crates.io, uploads platform binaries, and updates the Homebrew formula.
