# Remote Password Visibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a Remote PC user explicitly choose to show their password while entering it, while preserving masked entry by default.

**Architecture:** `src/remote.rs` gains a small pure parser for the opt-in answer and a password-entry helper. The existing sign-in request continues to receive only the entered string, with no logging or persistence of the password.

**Tech Stack:** Rust, existing `rpassword` crate, Cargo unit tests.

## Global Constraints

- The default (Enter, blank input, or unrecognised input) must keep the password masked.
- Only exact `y` or `yes`, ignoring ASCII case and surrounding whitespace, enables visible input.
- Do not expose, print, persist, or transmit the password beyond the existing Supabase sign-in request.
- Do not modify Desktop Connector password entry.

---

### Task 1: Explicit Remote PC password visibility

**Files:**
- Modify: `src/remote.rs:59-78`
- Test: `src/remote.rs` unit-test module

**Interfaces:**
- Produces: `fn show_password_requested(input: &str) -> bool`
- Produces: `fn prompt_remote_password() -> Result<String, String>`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn visible_password_requires_an_explicit_yes() {
    assert!(show_password_requested("y"));
    assert!(show_password_requested(" YES "));
    assert!(!show_password_requested(""));
    assert!(!show_password_requested("n"));
    assert!(!show_password_requested("show"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test remote::tests::visible_password_requires_an_explicit_yes --lib`

Expected: FAIL because `show_password_requested` is not defined.

- [ ] **Step 3: Write minimal implementation**

```rust
fn show_password_requested(input: &str) -> bool {
    matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn prompt_remote_password() -> Result<String, String> {
    let choice = prompt_optional("แสดงรหัสผ่านขณะพิมพ์หรือไม่? [y/N]: ")?;
    if show_password_requested(&choice) {
        prompt("รหัสผ่าน (แสดง): ")
    } else {
        rpassword::prompt_password("รหัสผ่าน (ซ่อน): ").map_err(|error| error.to_string())
    }
}
```

Use `prompt_remote_password()` in `run()` in place of the direct `rpassword` call. `prompt_optional()` reads stdin like `prompt()` but permits an empty value so Enter selects the safe default.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test remote::tests::visible_password_requires_an_explicit_yes --lib`

Expected: PASS.

- [ ] **Step 5: Run regression verification and commit**

Run: `cargo test --lib` and `node --test tests/*.test.cjs`.

Commit:

```bash
git add src/remote.rs docs/superpowers/plans/2026-08-12-remote-password-visibility.md
git commit -m "feat(remote): allow visible password entry on request"
```
