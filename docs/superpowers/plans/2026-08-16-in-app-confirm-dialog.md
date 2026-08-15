# In-app Confirmation Dialog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace native browser confirmation prompts with a reusable CommandBlock confirmation modal.

**Architecture:** `src/ui.html` owns one shared modal and the small Promise-based `confirmAction(options)` interface. Existing deletion, backup restore, and logout handlers await this interface before making their current requests; no service, API, or storage contract changes.

**Tech Stack:** Static HTML, CSS and browser JavaScript; Node built-in test runner; Rust/Cargo release build.

## Global Constraints

- Keep the Obsidian-purple CommandBlock visual language.
- Cancel is initially focused; backdrop, Escape and Cancel do not perform the action.
- Do not change backend routes, Supabase tables, credentials or persisted message data.
- Keep unrelated local edits in `src/config.rs`, `src/diagnostics.rs`, and `buff_session.json.bak` unstaged.

---

### Task 1: Specify the UI contract with a failing test

**Files:**
- Create: `tests/in-app-confirm-contract.test.cjs`
- Modify: `tests/session-version-contract.test.cjs`

**Interfaces:**
- Consumes: the `src/ui.html` source text.
- Produces: a regression contract requiring `confirmAction(options)`, the three modal controls, and no native browser confirmation calls.

- [ ] **Step 1: Write the failing test**

```js
test('destructive actions use the in-app confirmation dialog', () => {
  const source = fs.readFileSync(uiPath, 'utf8');
  assert.match(source, /function confirmAction\(options\)/);
  assert.match(source, /id="confirmModal"/);
  assert.match(source, /id="confirmCancel"/);
  assert.match(source, /id="confirmApprove"/);
  assert.doesNotMatch(source, /\b(?:window\.)?confirm\(/);
  assert.match(source, /await confirmAction\([\s\S]*ลบ SESSION/);
  assert.match(source, /await confirmAction\([\s\S]*กู้คืน/);
  assert.match(source, /await confirmAction\([\s\S]*ออกจากระบบ/);
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `node --test tests/in-app-confirm-contract.test.cjs`

Expected: failure because `confirmAction(options)` and the custom controls are not present yet.

- [ ] **Step 3: Update the version contract**

Change the expected Cargo package version from `1.0.6` to `1.0.7` so this change produces a downloadable update.

- [ ] **Step 4: Commit the test contract**

```powershell
git add tests/in-app-confirm-contract.test.cjs tests/session-version-contract.test.cjs
git commit -m "test: cover in-app confirmation dialog"
```

### Task 2: Build the shared confirmation modal

**Files:**
- Modify: `src/ui.html` near existing modal markup, modal CSS, and keyboard handlers.

**Interfaces:**
- Consumes: `confirmAction(options: { title, message, confirmLabel, danger })` from callers.
- Produces: `Promise<boolean>`; resolves `true` only from the explicit Confirm button.

- [ ] **Step 1: Add the modal markup and Obsidian CSS**

Create `#confirmModal` containing `#confirmTitle`, `#confirmBody`, `#confirmCancel`, and `#confirmApprove`; keep it hidden until used. Style the card to fit `min(430px, calc(100vw - 28px))` and use a danger treatment only when the options request it.

- [ ] **Step 2: Add the minimal shared behavior**

```js
function confirmAction(options) {
  const { title, message, confirmLabel, danger } = options;
  // Populate the modal, focus Cancel, then resolve true only on Confirm.
  return new Promise((resolve) => { pendingConfirmation = { resolve }; });
}
```

Wire Cancel, backdrop and Escape to resolve `false`, and close a prior unresolved dialog as `false` before opening another.

- [ ] **Step 3: Re-run the focused test**

Run: `node --test tests/in-app-confirm-contract.test.cjs`

Expected: it still fails only because the three callers have not switched to the new API.

### Task 3: Move each destructive caller to the shared modal

**Files:**
- Modify: `src/ui.html` in `deleteSession`, backup restore, and logout handlers.
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

**Interfaces:**
- Consumes: `await confirmAction(options)` from Task 2.
- Produces: requests proceed only when the caller receives `true`.

- [ ] **Step 1: Replace delete SESSION confirmation**

```js
const confirmed = await confirmAction({
  title: 'ลบ SESSION?',
  message: 'ลบ SESSION นี้และข้อความทั้งหมดในนั้นหรือไม่? การลบไม่สามารถย้อนกลับได้',
  confirmLabel: 'ลบ SESSION',
  danger: true,
});
if (!confirmed) return;
```

- [ ] **Step 2: Replace restore and logout confirmations**

Use explicit Thai consequence text and labels `กู้คืน` and `ออกจากระบบ`; leave both actions non-danger accent confirmations.

- [ ] **Step 3: Increment package version**

Set the root Cargo package version to `1.0.7` and allow Cargo to synchronize `Cargo.lock` during the build.

- [ ] **Step 4: Verify focused tests pass**

Run: `node --test tests/in-app-confirm-contract.test.cjs tests/session-version-contract.test.cjs`

Expected: both tests pass with no native `confirm()` remaining.

### Task 4: Verify and publish the desktop update

**Files:**
- Modify: `src/ui.html`, `Cargo.toml`, `Cargo.lock`, and the tests from Task 1.

**Interfaces:**
- Consumes: the feature commit.
- Produces: a GitHub release asset and GitHub Pages update users can download.

- [ ] **Step 1: Run all verification**

```powershell
node --test tests/*.test.cjs
cargo test
cargo build --release
git diff --check
```

- [ ] **Step 2: Commit only feature files**

```powershell
git add Cargo.toml Cargo.lock src/ui.html tests/in-app-confirm-contract.test.cjs tests/session-version-contract.test.cjs docs/superpowers/plans/2026-08-16-in-app-confirm-dialog.md
git commit -m "feat(ui): replace native confirmations"
```

- [ ] **Step 3: Push and verify release automation**

```powershell
git push origin main
gh run list --workflow release-windows.yml --limit 1
```

Wait for the release workflow to finish successfully, then inspect the release for the `CommandBlock-Windows-x64.zip` asset and checksum.
