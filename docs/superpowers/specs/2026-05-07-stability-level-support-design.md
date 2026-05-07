# Stability Level Support for WildFly Analysis

## Problem

WildFly 31 introduced stability levels (`experimental`, `preview`, `community`, `default`) that control which management model resources are visible. Without passing `--stability=experimental` when starting WildFly for analysis, resources at lower stability levels are hidden and excluded from the Neo4J model database. This means they cannot be found later by Cypher queries.

## Design

### Overview

Add `--stability=experimental` to the WildFly container startup arguments when the version supports it (>= 31). This ensures the analyzer captures all resources regardless of their stability level.

### Changes

#### 1. `wildfly_meta` crate

Add a `supports_stability()` method to `WildFlyImage`:

- Define a constant `STABILITY_MIN_IDENTIFIER: u16 = 310` (WildFly 31.0).
- Add `pub fn supports_stability(&self) -> bool` that returns `true` when `identifier >= STABILITY_MIN_IDENTIFIER`.
- The dev build (identifier 0) returns `true` since dev builds track the latest WildFly source, which supports stability levels.
- Add unit tests for boundary cases: WildFly 30 (false), 31 (true), 39 (true), dev (true).
- User will bump the crate version and release separately.

#### 2. `mgt` tooling (`src/command/analyze/wildfly.rs`)

- Add `supports_stability: bool` field to `AnalysisInstance`.
- Populate it from `image.supports_stability()` when creating instances in `run_wildfly_analysis`.
- In `start_wildfly`, conditionally append `--stability=experimental` to the container command args after `-c <configuration>`.
- Update `Cargo.toml` to use the new `wildfly_meta` version.
- Add unit tests verifying the `AnalysisInstance` field is set correctly for versions below and above 31.

### Sequence

1. Release `wildfly_meta` with `supports_stability()`.
2. Update `mgt` to depend on the new version and use the method.

### Not in scope

- Making the stability level configurable via CLI args on `mgt analyze`. The analyzer always wants the lowest level to capture everything.
- Feature pack analysis. Feature packs don't start a WildFly server; they use doc-zip archives, so stability levels don't apply.
