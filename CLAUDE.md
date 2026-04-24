# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`mgt` is a Rust CLI tool that orchestrates the analysis of the WildFly management model. It automates the pipeline of starting WildFly containers, spinning up Neo4J databases, running the Java-based [analyzer](https://github.com/model-graph-tools/analyzer), and cleaning up resources. Part of the [model graph tools](https://model-graph-tools.github.io/) ecosystem.

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

The codebase is small (~6 source files) with a straightforward flow:

- **`main.rs`** — Entry point. Parses CLI args via clap, dispatches to subcommands. Currently only `analyze` is wired up (there's also a `neo4j` subcommand defined in the app but not yet connected).
- **`app.rs`** — Defines the clap `Command` tree (subcommands, args, styling). Shared between `main.rs` and `build.rs` via `include!`.
- **`args.rs`** — Helpers to extract typed arguments from clap `ArgMatches`.
- **`analyze.rs`** — Core logic. For each WildFly version: starts WildFly containers (full-ha + microprofile configs), starts a Neo4J container, downloads and runs the analyzer JAR, then cleans up. Uses `tokio::spawn` + `join_all` for parallel analysis across versions, and `tokio::join!` for concurrent container startup/shutdown within a single version.
- **`container.rs`** — Container runtime abstraction. Prefers `podman`, falls back to `docker`.
- **`neo4j.rs`** — Neo4J container configuration (port calculation, naming conventions).
- **`constants.rs`** — Analyzer version and download URL.

## Key Dependencies

- **`wado`** / **`wildfly_container_versions`** — WildFly container abstractions (images, ports, version parsing). The `WildFlyContainer::enumeration()` parser handles version specs like `34`, `10,26.1,34`, `20..29`.
- **`clap`** — CLI argument parsing with shell completion generation at build time (`build.rs` → `completions/`).
- **`tokio`** + **`futures`** — Async runtime for concurrent container orchestration.
- **`reqwest`** — Downloads the analyzer JAR from GitHub releases.

## Runtime Requirements

- `podman` or `docker` must be available on PATH
- `java` must be available on PATH (to run the analyzer JAR)
- Network access to pull container images and download the analyzer JAR

## Version Identifiers

WildFly versions are specified as `<major>[.<minor>]` where major >= 10 and minor 0-9. Supports comma-separated lists (`10,26.1,34`) and ranges (`20..29`).
