# Update curl Fallback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow CommandBlock to download updates on Windows networks where `ureq` receives repeated truncated GitHub CDN responses.

**Architecture:** Retain the existing `ureq` resume path first. When it exhausts its three attempts, an isolated `curl.exe` fallback downloads the ZIP to a temporary file; the current checksum validation remains the boundary before package extraction.

**Tech Stack:** Rust, Windows `curl.exe`, Cargo unit tests.

## Global Constraints

- Preserve `ureq` as the first transport.
- Run curl without a visible console window.
- Require a successful exit status and exact known package length before accepting fallback bytes.
- Preserve SHA-256 verification and no-extraction-before-verification safety.

---

### Task 1: Add a fallback after primary download exhaustion

**Files:**
- Modify: `src/update.rs:264-341`
- Test: `src/update.rs` unit-test module

**Interfaces:**
- Produces: `fn choose_download_result<T, P, F>(primary: P, fallback: F) -> Result<T, String>`
- Consumes: `fn read_package_with_curl(release: &Release) -> Result<Vec<u8>, String>`

- [ ] **Step 1: Write a failing test**

```rust
#[test]
fn download_uses_fallback_after_primary_transport_exhausts() {
    let mut fallback_calls = 0;
    let result = choose_download_result(
        || Err("primary Unexpected EOF".to_string()),
        || { fallback_calls += 1; Ok("archive") },
    );
    assert_eq!(result.unwrap(), "archive");
    assert_eq!(fallback_calls, 1);
}
```

- [ ] **Step 2: Verify red**

Run: `cargo test update::tests::download_uses_fallback_after_primary_transport_exhausts --lib`

Expected: FAIL because `choose_download_result` does not exist.

- [ ] **Step 3: Implement fallback selection and curl transport**

Implement `choose_download_result` so it returns primary bytes on success, otherwise runs exactly one fallback and includes the primary error if fallback fails. Call it from `read_package_with_progress` after its existing retry loop. `read_package_with_curl` runs `curl.exe --fail --location --retry 3 --retry-all-errors --silent --show-error --output <temporary-file> <release-url>` with `CREATE_NO_WINDOW`, checks exit status and expected length, then removes its temporary file after reading it.

- [ ] **Step 4: Verify green**

Run: `cargo test update::tests::download_uses_fallback_after_primary_transport_exhausts --lib`

Expected: PASS.

- [ ] **Step 5: Run full regression verification and commit**

Run: `cargo test --lib`, `node --test tests/*.test.cjs`, and `cargo build --release`.

Commit:

```bash
git add src/update.rs docs/superpowers/specs/2026-08-12-update-curl-fallback-design.md docs/superpowers/plans/2026-08-12-update-curl-fallback.md
git commit -m "fix(update): fall back to Windows curl"
```
