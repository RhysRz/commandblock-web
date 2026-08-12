# Terminal Quick Actions and Mobile Remote Controls Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add one-click Desktop Connector/Remote PC launch buttons to the desktop Terminal and a touch-first remote layout on CommandBlock Web.

**Architecture:** The local HTTP server owns a small allowlisted launcher function, called by the Terminal UI. The web adapter owns mobile-only remote CSS and controls, leaving WebRTC signaling unchanged.

**Tech Stack:** Rust, Wry local HTTP UI, vanilla JavaScript/CSS, Supabase Web adapter.

## Global Constraints

- Only `connector` and `remote` are valid local launcher modes.
- Do not expose arbitrary shell execution through the launcher endpoint.
- Mobile actions must be at least 44px tall.
- Preserve existing `Commandblock.exe --connector` and `Commandblock.exe --remote` commands.

---

### Task 1: Add an allowlisted desktop launcher

**Files:**
- Modify: `src/gui.rs`
- Test: `src/gui.rs`

**Interfaces:**
- Produces: `launch_desktop_mode(mode: &str) -> Result<&'static str, String>`
- Consumes: `crate::connector::launch_sidecar()` and `crate::remote::launch_sidecar()`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn desktop_launcher_accepts_only_connector_and_remote() {
    assert!(launch_desktop_mode("connector").is_ok());
    assert!(launch_desktop_mode("remote").is_ok());
    assert!(launch_desktop_mode("cmd /c whoami").is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test desktop_launcher_accepts_only_connector_and_remote --lib`

- [ ] **Step 3: Implement the allowlist and POST `/api/desktop-mode`**

```rust
match mode {
    "connector" => crate::connector::launch_sidecar().map(|_| "Desktop Connector เปิดแล้ว"),
    "remote" => crate::remote::launch_sidecar().map(|_| "Remote PC เปิดแล้ว"),
    _ => Err("โหมด Desktop ไม่ถูกต้อง".to_string()),
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test desktop_launcher_accepts_only_connector_and_remote --lib`

### Task 2: Add Terminal quick actions

**Files:**
- Modify: `src/ui.html`
- Test: `tests/terminal-remote-controls.test.cjs`

**Interfaces:**
- Consumes: `POST /api/desktop-mode` with `{ "mode": "connector" | "remote" }`
- Produces: `launchDesktopMode(mode)` and buttons `#openConnector`, `#openRemote`

- [ ] **Step 1: Write the failing markup contract**

```js
assert.match(ui, /id="openConnector"/);
assert.match(ui, /id="openRemote"/);
assert.match(ui, /\/api\/desktop-mode/);
```

- [ ] **Step 2: Run it and verify it fails**

Run: `node --test tests/terminal-remote-controls.test.cjs`

- [ ] **Step 3: Add buttons and minimal fetch handler**

```js
async function launchDesktopMode(mode) {
  const res = await fetch('/api/desktop-mode', { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({mode}) });
  const result = await res.json();
  termPrint(result.message || result.error);
}
```

- [ ] **Step 4: Run contract and JS parse checks**

Run: `node --test tests/terminal-remote-controls.test.cjs; node --check src/ui.html`

### Task 3: Make mobile Remote controls touch-first

**Files:**
- Modify: `web/cloud-adapter.js`
- Modify: `tests/terminal-remote-controls.test.cjs`

**Interfaces:**
- Produces: `.cb-remote-actions`, `#cb-remote-disconnect`, mobile media rule with 44px controls
- Consumes: existing `peer`, `activeSession`, and Supabase session close update

- [ ] **Step 1: Extend the failing web contract**

```js
assert.match(adapter, /cb-remote-actions/);
assert.match(adapter, /cb-remote-disconnect/);
assert.match(adapter, /min-height:44px/);
```

- [ ] **Step 2: Run it and verify it fails**

Run: `node --test tests/terminal-remote-controls.test.cjs`

- [ ] **Step 3: Add sticky action row and Disconnect handler**

```js
disconnect.onclick = async () => {
  peer?.close();
  if (activeSession) await client.from('remote_sessions').update({ status:'closed' }).eq('id', activeSession);
  report('ตัดการเชื่อมต่อแล้ว');
};
```

- [ ] **Step 4: Verify generated web bundle**

Run: `node scripts/build-web.mjs; node --check web/cloud-adapter.js; node --check site/cloud-adapter.js`

### Task 4: Full verification and release

**Files:**
- Verify: `src/gui.rs`, `src/ui.html`, `web/cloud-adapter.js`, `tests/terminal-remote-controls.test.cjs`

- [ ] **Step 1: Run all relevant tests**

Run: `cargo test --lib; node --test tests/remote-session-contract.test.cjs tests/terminal-remote-controls.test.cjs`

- [ ] **Step 2: Build release binaries**

Run: `cargo build --release --bins`

- [ ] **Step 3: Commit and push**

```powershell
git add src/gui.rs src/ui.html web/cloud-adapter.js tests/terminal-remote-controls.test.cjs docs/superpowers
git commit -m "feat(ui): add desktop remote quick actions"
git push origin main
```
