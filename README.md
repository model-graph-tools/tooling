![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/model-graph-tools/tooling/verify.yml)
[![Crates.io](https://img.shields.io/crates/v/mgt.svg)](https://crates.io/crates/mgt)

# Model Graph Tooling

`mgt` is a command line tool for working with the [WildFly](https://www.wildfly.org/) management model graph. It serves two main purposes:

1. **Analysis Pipeline** — Orchestrates starting WildFly containers, spinning up [Neo4J](https://neo4j.com/) databases, running the Java-based [analyzer](https://github.com/model-graph-tools/analyzer), and building self-contained Neo4J images with pre-populated databases.
2. **Model Container Management** — Starts and stops Neo4J model containers, making the management model graph available for querying and exploration.

`mgt` is a key component of the [model graph tools](https://model-graph-tools.github.io/) ecosystem and powers the [Model Graph Tools Claude Code Plugin](https://github.com/model-graph-tools/claude-plugin), which provides an MCP server for exploring the WildFly management model directly from Claude Code.

- [Installation](#installation)
- [Identifiers](#identifiers)
- [Commands](#commands)
    - [analyze](#analyze)
    - [push](#push)
    - [start](#start)
    - [stop](#stop)
    - [browse](#browse)
    - [images](#images)
    - [versions](#versions)
    - [feature-packs](#feature-packs)
    - [resolve](#resolve)
    - [ps](#ps)
    - [completions](#completions)
    - [update](#update)

# Installation

[Precompiled binaries](https://github.com/model-graph-tools/tooling/releases) are available for macOS (Intel & Apple Silicon), Linux, and Windows.

## Quick Install

```shell
curl -fsSL https://model-graph-tools.github.io/mgt/install.sh | sh
```

This detects your OS and architecture, downloads the latest binary, and installs it to `~/.local/bin`. To install to a different directory:

```shell
curl -fsSL https://model-graph-tools.github.io/mgt/install.sh | MGT_INSTALL_DIR=/usr/local/bin sh
```

## Brew

```shell
brew tap hpehl/tap
brew install mgt
```

## npm

Platform-specific npm packages are published under `@model-graph-tools/mgt-{platform}`. These are primarily used by the [MCP server](https://github.com/model-graph-tools/claude-plugin) to bundle the `mgt` binary automatically, but you can also install them directly:

```shell
npm install -g @model-graph-tools/mgt-darwin-arm64   # macOS Apple Silicon
npm install -g @model-graph-tools/mgt-darwin-x64     # macOS Intel
npm install -g @model-graph-tools/mgt-linux-x64      # Linux x64
npm install -g @model-graph-tools/mgt-linux-arm64    # Linux ARM64
npm install -g @model-graph-tools/mgt-win32-x64      # Windows x64
```

## Cargo

```shell
cargo install mgt
```

## Build from source

1. `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` (see [Install Rust and Cargo](https://www.rust-lang.org/tools/install))
2. `git clone git@github.com:model-graph-tools/tooling.git`
3. `cd tooling`
4. `cargo build --release && cargo install --path .`

This installs the `mgt` binary to `~/.cargo/bin/` which should be in your `$PATH`.

## Shell Completions

`mgt` provides dynamic shell completions. The easiest way to set them up is:

```shell
mgt completions --install
```

This auto-detects your shell and installs the completion script to the standard location. You can also specify the shell explicitly:

```shell
mgt completions fish --install
```

To print the completion script to stdout (e.g. for manual setup or piping):

```shell
mgt completions fish
```

Supported shells: `bash`, `zsh`, `fish`, `elvish`, `powershell`.

# Identifiers

Most commands accept an identifier argument. Identifiers can be WildFly versions, feature pack names, or a combination of both.

## WildFly Versions

WildFly versions are specified as `<major>[.<minor>]` where major is >= 10 and minor is optional (0-9). You can use comma-separated lists and ranges:

**Examples**

- `34` — single version
- `26.1` — version with minor
- `10,26.1,34` — comma-separated list
- `20..29` — range

All supported versions can be listed with `mgt versions`.

## Feature Packs

Feature packs are identified by their shortcut name (e.g. `ai`, `graphql`). All supported feature packs can be listed with `mgt feature-packs`.

## Mixed

Identifiers can mix WildFly versions and feature packs:

```shell
mgt start 34,ai,graphql
```

# Commands

> [!IMPORTANT]
> Most commands require `podman` to be present with `docker` as a fallback.

## analyze

Analyzes the management model of a WildFly instance or feature pack and builds a self-contained Neo4J image with the results. For each identifier, the command:

1. Starts a WildFly standalone instance from [quay.io/wado/wado-sa](https://quay.io/repository/wado/wado-sa).
2. Starts an empty Neo4J database from [docker.io/neo4j](https://hub.docker.com/_/neo4j).
3. Downloads and runs the [analyzer](https://github.com/model-graph-tools/analyzer).
4. Builds a self-contained Neo4J image published to [quay.io/modelgraphtools/model](https://quay.io/repository/modelgraphtools/model).
5. Shuts down instances and cleans up resources.

```shell
mgt analyze 34
mgt analyze ai
```

## push

Pushes model DB images to the remote registry. Images must have been built previously with `mgt analyze`. Requires `podman login quay.io` beforehand.

```shell
mgt push 34
mgt push 34,ai,graphql
mgt push 26..29 --chunks 2
```

## start

Starts model containers from previously built images.

```shell
mgt start 34
mgt start 34,ai,graphql
```

## stop

Stops running model containers by identifier or all at once.

```shell
mgt stop 34
mgt stop --all
```

## browse

Opens the Neo4J browser for running model containers.

```shell
mgt browse 34
mgt browse ai
```

## images

Lists all locally available model images. Use `--wildfly` or `--feature-packs` to filter.

```shell
mgt images
mgt images --wildfly
mgt images --feature-packs
```

## versions

Lists all supported WildFly versions.

```shell
mgt versions
```

## feature-packs

Lists all supported feature packs.

```shell
mgt feature-packs
```

## resolve

Resolves identifiers to their canonical form. Useful for scripting and verifying how `mgt` interprets version strings and feature pack shortcuts.

```shell
mgt resolve 34
mgt resolve 34,ai,graphql
```

## ps

Lists all running model containers.

```shell
mgt ps
```

## completions

Generates and installs shell completions. See [Shell Completions](#shell-completions).

```shell
mgt completions --install
mgt completions fish --install
mgt completions fish
```

## update

Updates the WildFly and feature pack configuration files used by `mgt`. These files are hosted at [github.com/hpehl/wildfly-meta](https://github.com/hpehl/wildfly-meta/) and stored locally in `~/.config/wildfly-meta/`.

```shell
mgt update
```
