# Desktop Reliability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add automatic update restart, local diagnostics, settings backup/restore, and visible release notes.

**Architecture:** Keep update acquisition in `src/update.rs`; expose release metadata through its status JSON. Add a focused `src/diagnostics.rs` module for safe report and backup files. `src/gui.rs` owns local HTTP endpoints and graceful delayed process exit after sending an install response. `src/ui.html` renders the four controls without external services.

**Tech Stack:** Rust, Wry/WebView2 local HTTP server, serde JSON, Node built-in tests.

## Global Constraints

- Backups and diagnostics stay on the user’s machine.
- Do not copy API keys, chat content, or a panic payload into diagnostic reports.
- Retain no more than five settings backups.
- The updater must receive the HTTP response before the CommandBlock process exits.

---

### Task 1: Update metadata and automatic restart

**Files:**

- Modify: `src/update.rs`
- Modify: `src/gui.rs`
- Modify: `src/ui.html`
- Test: `tests/desktop-update-controls.test.cjs`

- [ ] **Step 1: Write failing contract assertions**

```js
assert.match(gui, /schedule_process_exit/);
assert.match(gui, /"release_notes"/);
assert.match(ui, /id="updateNotes"/);
```

- [ ] **Step 2: Run `node --test tests/desktop-update-controls.test.cjs` and verify the assertions fail.**

- [ ] **Step 3: Add release metadata, a release-note display, and a delayed exit scheduled after HTTP response flush.**

- [ ] **Step 4: Run `node --test tests/desktop-update-controls.test.cjs` and verify it passes.**

### Task 2: Safe diagnostics and settings snapshots

**Files:**

- Create: `src/diagnostics.rs`
- Modify: `src/main.rs`
- Modify: `src/gui.rs`
- Test: `tests/desktop-reliability-controls.test.cjs`

- [ ] **Step 1: Write failing contract assertions for the panic hook, local report path, five-backup retention, and backup endpoints.**

- [ ] **Step 2: Run `node --test tests/desktop-reliability-controls.test.cjs` and verify it fails.**

- [ ] **Step 3: Implement generic report writing, snapshot create/list/restore, and the local HTTP endpoints.**

- [ ] **Step 4: Run the focused test and `cargo test` to verify the implementation passes.**

### Task 3: Desktop controls

**Files:**

- Modify: `src/ui.html`
- Test: `tests/desktop-reliability-controls.test.cjs`

- [ ] **Step 1: Extend the failing contract with report-copy and backup-restore control IDs.**

- [ ] **Step 2: Run the focused test and verify it fails.**

- [ ] **Step 3: Add accessible controls and small responsive panels that call the local endpoints.**

- [ ] **Step 4: Run the focused test and verify it passes.**

### Task 4: Full verification and release

**Files:**

- Modify: none unless verification identifies a fault.

- [ ] **Step 1: Run `node --test tests/*.test.cjs`.**
- [ ] **Step 2: Run `cargo fmt --check; cargo test; cargo build --release`.**
- [ ] **Step 3: Commit and push only after all commands succeed.**
