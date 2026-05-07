# Stability Level Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `supports_stability()` to `WildFlyImage` so consumers can determine whether a WildFly version supports the `--stability` CLI parameter.

**Architecture:** A constant defines the minimum identifier threshold (310 = WildFly 31). The method returns `true` for dev builds (identifier 0) and for any version >= 31. Tests cover boundary cases.

**Tech Stack:** Rust, cargo test

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src/wildfly_image/mod.rs` | Modify | Add constant and `supports_stability()` method |
| `src/wildfly_image/tests.rs` | Modify | Add boundary-case tests |

---

### Task 1: Add `supports_stability()` to `WildFlyImage`

**Files:**
- Modify: `src/wildfly_image/mod.rs`
- Modify: `src/wildfly_image/tests.rs`

**Working directory:** `/Users/hpehl/dev/wildfly/wildfly-meta`

- [ ] **Step 1: Write the failing tests**

Add the following test section at the end of `src/wildfly_image/tests.rs`, before the closing of the module:

```rust
// ------------------------------------------------------ stability support

#[test]
fn supports_stability_below_threshold() {
    let reg = test_registry();
    let img = reg.get(300).unwrap(); // WildFly 30.0
    assert!(!img.supports_stability());
}

#[test]
fn supports_stability_at_threshold() {
    let reg = test_registry();
    let img = reg.get(310).unwrap(); // WildFly 31.0
    assert!(img.supports_stability());
}

#[test]
fn supports_stability_above_threshold() {
    let reg = test_registry();
    let img = reg.get(390).unwrap(); // WildFly 39.0
    assert!(img.supports_stability());
}

#[test]
fn supports_stability_dev() {
    let dev = wildfly_dev();
    assert!(dev.supports_stability());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib wildfly_image::tests::supports_stability`

Expected: All four tests FAIL with "no method named `supports_stability` found"

- [ ] **Step 3: Add the constant and method**

In `src/wildfly_image/mod.rs`, add the constant after the existing port base constants (after line 22):

```rust
/// Minimum identifier for stability level support (WildFly 31.0).
const STABILITY_MIN_IDENTIFIER: u16 = 310;
```

Add the method to the `impl WildFlyImage` block, after the `management_port()` method (after line 112):

```rust
    /// Returns `true` if this WildFly version supports the `--stability` CLI parameter.
    ///
    /// Stability levels were introduced in WildFly 31. Dev builds always support stability
    /// since they track the latest WildFly source.
    pub fn supports_stability(&self) -> bool {
        self.is_dev() || self.identifier >= STABILITY_MIN_IDENTIFIER
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib wildfly_image::tests::supports_stability`

Expected: All four tests PASS

- [ ] **Step 5: Run the full test suite**

Run: `cargo test`

Expected: All existing tests still pass, no regressions

- [ ] **Step 6: Run clippy and format**

Run: `cargo clippy && cargo fmt`

Expected: No warnings, no formatting changes

- [ ] **Step 7: Commit**

```bash
git add src/wildfly_image/mod.rs src/wildfly_image/tests.rs
git commit -m "feat: add supports_stability() to WildFlyImage"
```
