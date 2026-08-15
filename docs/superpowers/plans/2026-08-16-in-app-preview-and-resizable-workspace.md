# In-App Preview and Resizable Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep local web previews inside CommandBlock by default and make the desktop right workspace panel resizable.

**Architecture:** Rust Preview tools retain the current local-server and `PREVIEW_URL` state but stop launching the external browser. The existing state refresh in `src/ui.html` owns tab selection and iframe loading. A dedicated DOM drag handle changes a desktop CSS variable that controls the right-pane grid column and persists that preference locally.

**Tech Stack:** Rust, embedded HTML/CSS/JavaScript, Node built-in test runner, Cargo.

## Global Constraints

- Preserve the explicit `เปิดในเบราว์เซอร์` control as the only external-browser action.
- Allow only `http://127.0.0.1:` local previews in Preview tools.
- Clamp desktop right-pane width to 240–720 pixels; hide resizing at the existing 900px mobile breakpoint.
- Do not stage or modify `src/config.rs`, `src/diagnostics.rs`, `buff_session.json.bak`, or `cbweb.html`.

---

### Task 1: Make Preview tools in-app by default

**Files:**
- Modify: `src/tools.rs:1124-1226`
- Modify: `src/gui.rs:1734-1755`
- Modify: `src/ui.html:2190-2210, 2266-2283, 2444-2457`
- Test: `tests/preview-plugin-contract.test.cjs`

**Interfaces:**
- Consumes: `PREVIEW_URL`, `last_preview_url()`, and GUI state field `preview_url`.
- Produces: Preview tool output which identifies the Preview tab and does not spawn the system browser.

- [x] **Step 1: Write the failing contract test**

```js
assert.doesNotMatch(previewTools, /open_browser\(&url\)/);
assert.match(previewTools, /เปิดแท็บ Preview ใน CommandBlock/);
assert.match(ui, /if\(String\(obj\.name\|\|""\)\.startsWith\("preview_"\) switchTab\("preview"\)/);
assert.match(ui, /window\.open\(state\.preview_url,"_blank"\)/);
```

- [x] **Step 2: Run the test to verify it fails**

Run: `node --test tests/preview-plugin-contract.test.cjs`

Expected: FAIL because Preview tools still call `open_browser`.

- [x] **Step 3: Implement minimal Preview behavior**

```rust
let _ = PREVIEW_URL.set(url.clone());
format!("[open_preview] เปิดพรีวิวในแท็บ Preview ของ CommandBlock: {url}")
```

Remove only Preview-originated `open_browser` calls. Leave the explicit UI browser button unchanged. After a Preview tool returns, `GuiSink::result` emits `preview_ready`; the UI receives that event, selects the Preview tab, then reloads the iframe URL. This avoids reading `/api/state` before the tool has stored `PREVIEW_URL`.

- [x] **Step 4: Run the focused test to verify it passes**

Run: `node --test tests/preview-plugin-contract.test.cjs`

Expected: PASS.

### Task 2: Add persistent desktop right-pane resizing

**Files:**
- Modify: `src/ui.html:24-35, 461-512, 1184-1188, 2266-2355`
- Test: `tests/preview-plugin-contract.test.cjs`

**Interfaces:**
- Consumes: CSS custom property `--rightpane-width` and `localStorage` key `commandblock.rightPaneWidth`.
- Produces: `#rightPaneResizer` pointer/keyboard interactions and a 240–720 pixel persisted width.

- [x] **Step 1: Write the failing contract test**

```js
assert.match(ui, /id="rightPaneResizer"/);
assert.match(ui, /--rightpane-width/);
assert.match(ui, /commandblock\.rightPaneWidth/);
assert.match(ui, /Math\.max\(240, Math\.min\(720,/);
assert.match(ui, /@media \(max-width:900px\).*rightPaneResizer/s);
```

- [x] **Step 2: Run the test to verify it fails**

Run: `node --test tests/preview-plugin-contract.test.cjs`

Expected: FAIL because the handle and storage behavior do not yet exist.

- [x] **Step 3: Implement the minimal resizer**

```js
function setRightPaneWidth(width) {
  const next = Math.max(240, Math.min(720, Math.round(width)));
  document.body.style.setProperty("--rightpane-width", `${next}px`);
  localStorage.setItem("commandblock.rightPaneWidth", String(next));
}
```

Add a focusable, labelled vertical handle before `#rightpane`. Use pointer capture while dragging; Arrow keys adjust by 20 pixels, Home and End apply the clamp bounds. Add CSS cursor, focus, and mobile-hide styles.

- [x] **Step 4: Run the focused test to verify it passes**

Run: `node --test tests/preview-plugin-contract.test.cjs`

Expected: PASS.

### Task 3: Verify, build, and publish

**Files:**
- Modify: `Cargo.toml` only if a version increment is required by the existing release convention.

- [x] **Step 1: Run static and unit verification**

Run: `git diff --check; node --test tests/*.test.cjs; cargo test`

Expected: zero whitespace errors, all Node tests passing, all Rust tests passing.

- [x] **Step 2: Build the release executable**

Run: `cargo build --release`

Expected: exit code 0 and `target/release/commandblock.exe` exists.

- [ ] **Step 3: Commit only feature files and push main**

```bash
git add src/tools.rs src/ui.html tests/preview-plugin-contract.test.cjs \
  docs/superpowers/plans/2026-08-16-in-app-preview-and-resizable-workspace.md
git commit -m "fix(preview): keep previews in app"
git push origin main
```

Do not stage user-owned changed or untracked files listed in Global Constraints.
