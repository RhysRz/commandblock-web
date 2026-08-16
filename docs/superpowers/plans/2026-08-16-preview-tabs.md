# Preview Tabs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add browser-style multi-tab Preview navigation to CommandBlock desktop and web UI.

**Architecture:** Keep preview tabs as UI-only state in `src/ui.html`, persisted in `sessionStorage`. The backend remains the source that validates and emits Preview URLs; UI events route URLs into the active preview tab. Each tab uses the existing iframe and toolbar, preserving local Preview behavior.

**Tech Stack:** Rust 2021, embedded HTML/CSS/JavaScript, Node built-in test runner, Playwright smoke test.

## Global Constraints

- Keep the Obsidian–purple visual system and existing mobile drawer layout.
- Do not add dependencies or bypass external frame restrictions.
- Treat local Preview URLs and validated HTTPS URLs as distinct `kind` values.
- Bump the desktop version to `1.0.14` and update `Cargo.lock`.
- Preserve unrelated user changes in `src/config.rs`, `src/diagnostics.rs`, `buff_session.json.bak`, `cbweb.html`, and `preview/`.

---

### Task 1: Define and test tab-state helpers

**Files:**
- Create: `web/preview-tabs.js`
- Create: `tests/preview-tabs-state.test.cjs`

**Interfaces:**
- Produces `window.CommandBlockPreviewTabs` with `restore(storage)`, `add(state, url)`, `select(state, id)`, `close(state, id)`, and `labelFor(url)`.
- State shape: `{ tabs: [{ id, url, title, kind }], activeId }`.

- [ ] **Step 1: Write the failing test**

```js
const state = tabs.add({ tabs: [], activeId: '' }, 'https://example.com/docs');
assert.equal(state.tabs.length, 1);
assert.equal(state.tabs[0].title, 'example.com');
assert.equal(state.activeId, state.tabs[0].id);
```

Add cases for selecting, closing the active tab, and malformed storage.

- [ ] **Step 2: Run the test and verify RED**

Run: `node --test tests/preview-tabs-state.test.cjs`

Expected: FAIL because `web/preview-tabs.js` does not exist.

- [ ] **Step 3: Write the minimal helper**

```js
function labelFor(url) {
  try { return new URL(url).hostname || 'New tab'; } catch { return 'New tab'; }
}
```

Export the documented APIs; use `crypto.randomUUID()` with a timestamp fallback.

- [ ] **Step 4: Run the test and verify GREEN**

Run: `node --test tests/preview-tabs-state.test.cjs`

Expected: PASS.

### Task 2: Render the Preview tab strip and wire controls

**Files:**
- Modify: `src/ui.html:544, 1244-1252, 2571-2588`
- Modify: `scripts/build-web.mjs:9-42`
- Modify: `tests/preview-plugin-contract.test.cjs`
- Modify: `tests/canonical-web-build.test.cjs`

**Interfaces:**
- Consumes `window.CommandBlockPreviewTabs` from `assets/preview-tabs.js`.
- Produces `#previewTabs`, `#previewTabAdd`, `#previewUrlInput`, and selected-tab iframe behavior.

- [ ] **Step 1: Write the failing UI contract**

```js
assert.match(ui, /id="previewTabs"/);
assert.match(ui, /id="previewTabAdd"/);
assert.match(ui, /id="previewUrlInput"/);
assert.match(ui, /function renderPreviewTabs/);
```

Assert that the web build copies `preview-tabs.js` into `site/assets`.

- [ ] **Step 2: Run the test and verify RED**

Run: `node --test tests/preview-plugin-contract.test.cjs tests/canonical-web-build.test.cjs`

Expected: FAIL because the tab controls and helper asset do not exist.

- [ ] **Step 3: Build the Obsidian tab strip**

Add horizontal tab buttons with an icon, label, close button, and fixed `+` control. Add toolbar URL input with Enter handling; render the selected URL in the existing iframe. Use the existing browser button for the active tab.

- [ ] **Step 4: Render a safe frame fallback**

When the iframe errors or times out, show an in-panel fallback with the URL and an explicit browser-open action. Do not bypass site frame policies.

- [ ] **Step 5: Run the UI contract and verify GREEN**

Run: `node --test tests/preview-plugin-contract.test.cjs tests/canonical-web-build.test.cjs`

Expected: PASS.

### Task 3: Route Preview events and publish v1.0.14

**Files:**
- Modify: `src/ui.html:2195, 2268, 2273`
- Modify: `Cargo.toml:3`
- Modify: `Cargo.lock` commandblock package entry
- Modify: `tests/session-version-contract.test.cjs`

**Interfaces:**
- Consumes `state.preview_url` and the `preview_ready` SSE event.
- Produces a new Preview tab for each emitted URL while preserving prior tabs.

- [ ] **Step 1: Write failing routing and version tests**

```js
assert.match(ui, /addPreviewTab\(state\.preview_url\)/);
assert.match(ui, /ev === "preview_ready"[\s\S]*addPreviewTab/);
assert.match(cargo, /^version = "1\.0\.14"$/m);
```

- [ ] **Step 2: Run the tests and verify RED**

Run: `node --test tests/preview-plugin-contract.test.cjs tests/session-version-contract.test.cjs`

Expected: FAIL because URL events replace a single Preview state and the version is `1.0.13`.

- [ ] **Step 3: Route URLs to tabs and bump version**

Replace direct iframe assignment with `addPreviewTab(url)` for new Preview URLs; do not duplicate the already-selected exact URL. Bump Cargo and lockfile package versions to `1.0.14`.

- [ ] **Step 4: Verify all deliverables**

Run:

```powershell
node --test tests/*.test.cjs
cargo test
cargo build --release
```

Run Playwright against generated `site/index.html`; assert add, select, close, and no page errors.

- [ ] **Step 5: Commit and push**

```powershell
git add Cargo.toml Cargo.lock src/ui.html scripts/build-web.mjs web/preview-tabs.js tests/preview-tabs-state.test.cjs tests/preview-plugin-contract.test.cjs tests/canonical-web-build.test.cjs tests/session-version-contract.test.cjs docs/superpowers/plans/2026-08-16-preview-tabs.md
git commit -m "feat(preview): add browser-style preview tabs"
git push origin main
```
