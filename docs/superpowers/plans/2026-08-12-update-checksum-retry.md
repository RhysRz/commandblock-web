# Update Checksum Retry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Retry transient checksum-download failures so a valid update archive can still be verified and staged.

**Architecture:** Add a small generic bounded retry helper in `src/update.rs`. The existing archive logic and the checksum fetch each use it; checksum comparison and extraction remain unchanged.

**Tech Stack:** Rust, existing `ureq`, Cargo unit tests.

## Global Constraints

- Make at most three fetch attempts.
- Keep the one- and two-second production retry delays.
- Preserve mandatory SHA-256 verification before staging any executable.
- Keep archive download progress reporting unchanged.

---

### Task 1: Retry checksum fetches

**Files:**
- Modify: `src/update.rs:237-341`
- Test: `src/update.rs` unit-test module

**Interfaces:**
- Produces: `fn with_fetch_retries<T, F, P>(fetch: F, pause: P) -> Result<T, String>`
- Consumes: `fn read_bytes(url: &str) -> Result<Vec<u8>, String>`

- [ ] **Step 1: Write a failing test**

```rust
#[test]
fn checksum_fetch_retries_until_the_third_attempt() {
    let mut attempts = 0;
    let result = with_fetch_retries(
        || {
            attempts += 1;
            if attempts < 3 { Err("Unexpected EOF".to_string()) } else { Ok("checksum") }
        },
        |_| {},
    );
    assert_eq!(result.unwrap(), "checksum");
    assert_eq!(attempts, 3);
}
```

- [ ] **Step 2: Verify the test fails**

Run: `cargo test update::tests::checksum_fetch_retries_until_the_third_attempt --lib`

Expected: FAIL because `with_fetch_retries` does not exist.

- [ ] **Step 3: Implement bounded retry and wire it to checksum fetch**

```rust
fn with_fetch_retries<T, F, P>(mut fetch: F, mut pause: P) -> Result<T, String>
where
    F: FnMut() -> Result<T, String>,
    P: FnMut(u8),
{
    let mut last_error = String::new();
    for attempt in 0..3 {
        match fetch() {
            Ok(value) => return Ok(value),
            Err(error) => {
                last_error = error;
                if attempt < 2 { pause(attempt); }
            }
        }
    }
    Err(format!("ดาวน์โหลดไฟล์ตรวจสอบไม่สำเร็จหลังลองใหม่ 3 ครั้ง: {last_error}"))
}
```

Use it in `stage_release` for `read_bytes(&release.checksum_url)` with `retry_delay` and `thread::sleep`.

- [ ] **Step 4: Verify green**

Run: `cargo test update::tests::checksum_fetch_retries_until_the_third_attempt --lib`

Expected: PASS.

- [ ] **Step 5: Run regression checks and commit**

Run: `cargo test --lib`, `node --test tests/*.test.cjs`, and `cargo build --release`.

Commit:

```bash
git add src/update.rs docs/superpowers/plans/2026-08-12-update-checksum-retry.md
git commit -m "fix(update): retry checksum downloads"
```
