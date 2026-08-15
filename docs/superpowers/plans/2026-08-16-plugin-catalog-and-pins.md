# Plugin Catalog and Pinned Tray Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe local plugin catalog, a sticky interactive pinned-message tray, and right-aligned New session control to CommandBlock.

**Architecture:** The shared `src/ui.html` remains the single UI implementation for both browser and Windows EXE. It receives durable message pin state from existing owner-scoped APIs. The plugin catalog is local metadata only; it never authenticates, installs, or invokes third-party services.

**Tech Stack:** Vanilla HTML/CSS/JavaScript, Rust embedded web UI, Node built-in test runner, Cargo.

## Global Constraints

- Reuse the shared `src/ui.html` and its Obsidian purple tokens.
- Do not claim an external provider is installed or connected without an authorized implementation.
- Do not add OAuth, API-key storage, external calls, or provider permissions.
- Preserve existing message-pin persistence and Supabase RLS.
- Bump the crate from `1.0.5` to `1.0.6`.
- Never stage `src/config.rs`, `src/diagnostics.rs`, or `buff_session.json.bak`.

---

### Task 1: Sticky pinned-message tray and SESSION header

**Files:**
- Create: `tests/pinned-message-tray.test.cjs`
- Modify: `src/ui.html:418-422`, `src/ui.html:1070`, `src/ui.html:1625-1632`
- Modify: `Cargo.toml:1-6`
- Modify: `tests/session-version-contract.test.cjs:6-9`

**Interfaces:**
- Consumes synchronized message rows `{ id, role, content, created_at, is_pinned }`.
- Produces `renderPinnedMessages(rows)` and `scrollToPinnedMessage(messageId)`.

- [ ] **Step 1: Write the failing test**

```js
test('pinned message tray stays sticky and navigates to its source message', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');
  assert.match(ui, /#pinnedMessages\s*\{[^}]*position:\s*sticky/s);
  assert.match(ui, /function scrollToPinnedMessage\(messageId\)/);
  assert.match(ui, /item\.addEventListener\("click".*scrollToPinnedMessage/s);
  assert.match(ui, /message\.classList\.add\("pinned-focus"\)/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/pinned-message-tray.test.cjs`

Expected: FAIL because the sticky CSS and navigation helper do not exist.

- [ ] **Step 3: Write minimal implementation**

```js
function scrollToPinnedMessage(messageId){
  const message=wrap.querySelector('.msg[data-conversation-id="'+CSS.escape(messageId)+'"]');
  if(!message) return;
  message.scrollIntoView({behavior:"smooth",block:"center"});
  message.classList.add("pinned-focus");
  window.setTimeout(()=>message.classList.remove("pinned-focus"), 1400);
}
```

Make the pin tray sticky under the chat header, render each pin as a button with its source id, and add the session-header flex layout with `margin-left:auto` on `+ New session`.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/pinned-message-tray.test.cjs tests/session-web-contract.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/ui.html Cargo.toml tests/pinned-message-tray.test.cjs tests/session-version-contract.test.cjs
git commit -m "feat(chat): keep pinned messages visible"
```

### Task 2: Truthful local plugin catalog

**Files:**
- Create: `tests/plugin-catalog-contract.test.cjs`
- Modify: `src/ui.html:1000-1040`, `src/ui.html:1116-1118`, `src/ui.html:1587-1596`

**Interfaces:**
- Consumes `PLUGIN_CATALOG` items `{ name, category, state, description, icon }`.
- Produces `openPluginCatalog()` and `renderPluginCatalog(query)`.
- `state` is `built-in`, `connect-required`, or `planned`.

- [ ] **Step 1: Write the failing test**

```js
test('plugin catalog opens from the left rail and labels states honestly', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');
  assert.match(ui, /id="pluginsBtn"/);
  assert.match(ui, /id="pluginCatalog"/);
  assert.match(ui, /const PLUGIN_CATALOG = \[/);
  assert.match(ui, /connect-required/);
  assert.match(ui, /function renderPluginCatalog\(query\)/);
  assert.match(ui, /id="pluginSearch"/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/plugin-catalog-contract.test.cjs`

Expected: FAIL because the rail trigger and catalog do not exist.

- [ ] **Step 3: Write minimal implementation**

```js
const PLUGIN_CATALOG = [
  { name:"GitHub", category:"Development", state:"connect-required", description:"Repos, issues, and pull requests", icon:"⌘" },
  { name:"Local workspace", category:"Development", state:"built-in", description:"Read and edit the selected project folder", icon:"▣" },
  { name:"Google Drive", category:"Storage", state:"connect-required", description:"Files after account connection", icon:"△" },
];
```

Add the left-rail icon and an Obsidian modal with search, installed/public tabs, category headings, cards, and non-actionable state badges. Include available catalog providers across Development, Productivity, Storage, Communication, Design, Hosting, and Billing. A card must not fetch, persist credentials, or say installation succeeded.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/plugin-catalog-contract.test.cjs tests/canonical-web-build.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/ui.html tests/plugin-catalog-contract.test.cjs
git commit -m "feat(plugins): add local integration catalog"
```

### Task 3: Verify and release

**Files:**
- Modify: `Cargo.lock` after the package version update.
- Modify: files from Tasks 1 and 2 only.

**Interfaces:**
- Consumes finalized shared UI and test contracts.
- Produces a version `1.0.6` Windows EXE and uploaded release assets.

- [ ] **Step 1: Run focused tests**

Run: `node --test tests/pinned-message-tray.test.cjs tests/plugin-catalog-contract.test.cjs tests/session-web-contract.test.cjs tests/mobile-navigation-layout.test.cjs`

Expected: all PASS.

- [ ] **Step 2: Run full verification**

Run: `cargo test; node --test tests/*.test.cjs; cargo build --release; git diff --check`

Expected: all tests PASS, release build exits 0, and no whitespace errors.

- [ ] **Step 3: Inspect staging boundaries**

Run: `git status --short; git diff --cached --name-only`

Expected: only feature files, tests, `Cargo.toml`, and `Cargo.lock` are staged.

- [ ] **Step 4: Commit and push one release commit**

```bash
git add Cargo.toml Cargo.lock src/ui.html tests/pinned-message-tray.test.cjs tests/plugin-catalog-contract.test.cjs tests/session-version-contract.test.cjs
git commit -m "feat(ui): add plugin catalog and sticky pins"
git push origin main
```

- [ ] **Step 5: Verify release artifacts**

Run: `gh run list --workflow release-windows.yml --limit 1 --json status,conclusion,headSha,url && gh release view build-<commit-sha> --json url,assets`

Expected: successful workflow and both `CommandBlock-Windows-x64.zip` and `.sha256` assets.
