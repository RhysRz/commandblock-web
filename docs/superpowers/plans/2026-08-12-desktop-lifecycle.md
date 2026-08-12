# Desktop Lifecycle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the Windows app install a verified update with one click and keep an opted-in Desktop Connector available from the system tray.

**Architecture:** `src/update.rs` exposes staged-update state and launches the existing helper only after the local UI explicitly requests restart. A new connector session module persists a Supabase refresh token in Windows Credential Manager, while the sidecar owns the tray menu and optional HKCU Run registration.

**Tech Stack:** Rust, ureq, existing local HTTP GUI, Windows Credential Manager via `keyring`, `tray-icon`, registry `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run`.

## Global Constraints

- Never store an account password, API key, command output, or screen content in the registry or Supabase.
- Only a checksum-verified staged package may be applied.
- Autostart is explicit opt-in, user-scoped, non-elevated, and removable.

---

### Task 1: Restart to apply a verified update

**Files:**
- Modify: `src/update.rs`, `src/gui.rs`, `src/ui.html`
- Test: `tests/desktop-update-controls.test.cjs`, `src/update.rs`

**Interfaces:**
- Produces `pub fn launch_staged_update() -> Result<(), String>` and `POST /api/update {"action":"install"}`.

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn staged_update_requires_both_expected_executables() {
    assert!(!stage_is_complete(temp.path()));
}
```

```js
assert.match(ui, /id="updateInstall"/);
assert.match(ui, /action:"install"/);
assert.match(gui, /"install"/);
```

- [ ] **Step 2: Run the focused tests and observe missing symbols/controls**

Run: `cargo test update::tests::staged_update_requires_both_expected_executables; node --test tests/desktop-update-controls.test.cjs`

Expected: FAIL because `stage_is_complete` and `updateInstall` do not exist.

- [ ] **Step 3: Implement the minimal verified-install path**

```rust
pub fn launch_staged_update() -> Result<(), String> {
    let (stage, base, helper) = staged_paths()?;
    std::process::Command::new(helper)
        .args(["--apply", stage.to_str().ok_or("พาธอัปเดตไม่ถูกต้อง")?, base.to_str().ok_or("พาธแอปไม่ถูกต้อง")?, &std::process::id().to_string()])
        .spawn().map_err(|e| format!("เริ่มติดตั้งอัปเดตไม่ได้: {e}"))?;
    Ok(())
}
```

The UI renders `ติดตั้งและเปิดใหม่` only in `ready` state. On HTTP 202 it shows a short status then calls `window.close()`; if closing is denied, it explains that the user can close the app manually.

- [ ] **Step 4: Re-run focused tests**

Run: `cargo test update::tests; node --test tests/desktop-update-controls.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add src/update.rs src/gui.rs src/ui.html tests/desktop-update-controls.test.cjs && git commit -m "feat(update): install staged build on restart"`

### Task 2: Persist Connector sessions without passwords

**Files:**
- Modify: `Cargo.toml`, `src/connector.rs`, `src/bin/commandblock-connector.rs`
- Create: `src/connector_session.rs`
- Test: `src/connector_session.rs`

**Interfaces:**
- Produces `ConnectorCredentials::{load, save, clear}` and `authenticate_or_prompt(&Agent) -> Result<ConnectorSession, String>`.

- [ ] **Step 1: Write failing credential lifecycle test**

```rust
#[test]
fn logout_removes_the_saved_refresh_token() {
    let store = MemoryCredentialStore::with_token("refresh");
    store.clear().unwrap();
    assert_eq!(store.load().unwrap(), None);
}
```

- [ ] **Step 2: Run it**

Run: `cargo test connector_session::tests::logout_removes_the_saved_refresh_token`

Expected: FAIL because the module does not exist.

- [ ] **Step 3: Implement credential adapter and refresh flow**

Use `keyring::Entry::new("CommandBlock", "connector-refresh-token")` only behind a `CredentialStore` trait. After password login, save `refresh_token`; at next launch post only that token to Supabase refresh grant. If refresh fails, delete it and request interactive login. Never log either token.

- [ ] **Step 4: Re-run it**

Run: `cargo test connector_session::tests`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add Cargo.toml src/connector.rs src/connector_session.rs src/bin/commandblock-connector.rs && git commit -m "feat(connector): reuse secure refresh sessions"`

### Task 3: Add tray controls and user-level autostart

**Files:**
- Modify: `Cargo.toml`, `src/connector.rs`, `src/gui.rs`, `src/ui.html`
- Create: `src/connector_tray.rs`
- Test: `tests/connector-tray-contract.test.cjs`, `src/connector_tray.rs`

**Interfaces:**
- Produces `connector_tray::run(mode: ConnectorMode, autostart: bool) -> Result<(), String>` and `GET/POST /api/connector-settings`.

- [ ] **Step 1: Write failing tests**

```js
assert.match(ui, /id="connectorAutostart"/);
assert.match(gui, /\/api\/connector-settings/);
```

```rust
#[test]
fn autostart_command_uses_only_the_current_executable() {
    assert!(autostart_command(Path::new("C:/App/commandblock-connector.exe")).contains("--connector"));
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test connector_tray::tests; node --test tests/connector-tray-contract.test.cjs`

Expected: FAIL because settings endpoint, menu, and command builder do not exist.

- [ ] **Step 3: Implement tray and settings**

Create menu actions `Open CommandBlock`, `Connect Desktop`, `Start Remote PC`, `Stop`, `Log out`, and `Exit`. Use `tray-icon` notification/state icon; register only a quoted executable path plus fixed `--connector` under the current user’s Run key. The GUI toggle defaults false and `POST` accepts exactly `{autostart:boolean}`.

- [ ] **Step 4: Re-run tests and build**

Run: `cargo test; node --test tests/connector-tray-contract.test.cjs; cargo build --release --bins`

Expected: all pass and release sidecars build.

- [ ] **Step 5: Commit**

Run: `git add Cargo.toml src/connector.rs src/connector_tray.rs src/gui.rs src/ui.html tests/connector-tray-contract.test.cjs && git commit -m "feat(connector): add tray and autostart controls"`
