# Update Freshness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent the update UI from appearing for identical or older runtime releases.

**Architecture:** The build script exports a runtime revision and build timestamp. The updater gates releases by tag identity and GitHub publication time. The Windows release workflow uses the same Git selector as the build script and uses that runtime ID as the release tag.

**Tech Stack:** Rust, GitHub Actions PowerShell, Node built-in test runner.

## Global Constraints

- Do not expose API keys or user data.
- Retain the existing `build-<id>` release-tag format.
- Treat absent or invalid timestamps as no update.

---

### Task 1: Define and verify update freshness

**Files:**

- Modify: `src/update.rs`
- Modify: `build.rs`
- Modify: `src/main.rs`
- Test: `src/update.rs`

**Interfaces:**

- Produces: `release_is_newer(tag: &str, published_at: &str, current_build: &str, current_timestamp: i64) -> bool`.

- [ ] **Step 1: Write the failing tests**

```rust
assert!(release_is_newer("build-next", "2026-08-12T16:00:00Z", "current", 1_786_000_000));
assert!(!release_is_newer("build-current", "2026-08-12T16:00:00Z", "current", 1));
assert!(!release_is_newer("build-next", "2026-08-12T15:00:00Z", "current", 1_786_000_000));
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run: `cargo test update::tests::only_newer_runtime_releases_are_offered`

- [ ] **Step 3: Implement the smallest timestamp gate and build metadata export**

Use GitHub `published_at` with RFC 3339 parsing and export a UTC Unix timestamp from `build.rs`.

- [ ] **Step 4: Run the focused test and verify it passes**

Run: `cargo test update::tests::only_newer_runtime_releases_are_offered`

### Task 2: Make release tags runtime-stable

**Files:**

- Modify: `.github/workflows/release-windows.yml`
- Modify: `tests/release-trigger-scope.test.cjs`

**Interfaces:**

- Consumes: the Git revision that last changed runtime inputs.
- Produces: one GitHub release per runtime ID.

- [ ] **Step 1: Write a failing workflow contract test**

```js
assert.match(release, /git log -1 --format=%H/);
assert.match(release, /gh release view \$tag/);
assert.match(release, /gh release create \$tag/);
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run: `node --test tests/release-trigger-scope.test.cjs`

- [ ] **Step 3: Change the workflow to derive and deduplicate the runtime tag**

Build the package first, calculate the runtime ID, skip an existing tag, then publish the archive under that tag.

- [ ] **Step 4: Run the focused test and verify it passes**

Run: `node --test tests/release-trigger-scope.test.cjs`

### Task 3: Verify the release client

**Files:**

- Modify: none unless verification identifies a fault.

- [ ] **Step 1: Run all static tests**

Run: `node --test tests/*.test.cjs`

- [ ] **Step 2: Compile and run Rust tests**

Run: `cargo test`

- [ ] **Step 3: Build the release executable and inspect its embedded build ID**

Run: `cargo build --release; rg -a "$(git log -1 --format=%H -- src assets Cargo.toml Cargo.lock build.rs)" target\\release\\commandblock.exe`
