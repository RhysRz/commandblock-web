# Plugin branding and account menu Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give Plugin cards local full-color provider marks and readable spacing, and add an upward account-management menu for the signed-in user.

**Architecture:** Keep the shared browser/EXE UI self-contained in `src/ui.html`. Provider marks are trusted inline SVG constants chosen by catalog metadata, so neither build needs a new asset route or runtime network request. The account menu reuses the current Supabase session, usage control, device controls, the existing in-app confirmation dialog, and matched Desktop/Web password-recovery routes backed by Supabase Auth.

**Tech Stack:** Rust embedded HTML (`include_str!`), vanilla JavaScript, CSS grid, Node contract tests, Cargo.

## Global Constraints

- Plugin logos are local, colored, trusted markup; never fetch logos at runtime.
- External Plugin cards retain truthful `Built in`, `Connect required`, or `Planned` states.
- The current authenticated account remains the only account available through the menu.
- Sign out must use `confirmAction`; no browser-native confirmation dialogs.
- Do not stage `src/config.rs`, `src/diagnostics.rs`, or `buff_session.json.bak`.
- Bump the desktop version after tests pass and verify the GitHub release artifact.

---

### Task 1: Define layout and account-menu contracts

**Files:**
- Modify: `tests/plugin-catalog-contract.test.cjs`
- Create: `tests/account-menu-contract.test.cjs`

**Interfaces:** Consumes `PLUGIN_CATALOG`, `pluginCatalog`, `accChip`, and `confirmAction(options)` from `src/ui.html`. Produces source-level contracts for local brand marks, responsive card layout, menu actions, and safe logout.

- [ ] **Step 1: Write failing Plugin layout assertions**

```js
assert.match(ui, /const PLUGIN_BRAND_MARKS = \{/);
assert.match(ui, /brandMark\(item\)/);
assert.match(ui, /\.plugin-modal-card \{ width:min\(980px,94vw\)/);
assert.match(ui, /\.plugin-copy \{ min-width:0/);
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/plugin-catalog-contract.test.cjs`

Expected: FAIL because brand metadata and wider layout do not exist yet.

- [ ] **Step 3: Write failing account-menu assertions**

```js
assert.match(ui, /id="accountMenu"/);
assert.match(ui, /id="accountManageBtn"/);
assert.match(ui, /id="accountResetPasswordBtn"/);
assert.match(ui, /id="accountSignOutBtn"/);
assert.match(ui, /await confirmAction\([\s\S]*ออกจากระบบ/);
```

- [ ] **Step 4: Run test to verify it fails**

Run: `node --test tests/account-menu-contract.test.cjs`

Expected: FAIL because the account menu is absent.

- [ ] **Step 5: Commit test contracts**

```bash
git add tests/plugin-catalog-contract.test.cjs tests/account-menu-contract.test.cjs
git commit -m "test: define plugin branding and account menu contracts"
```

### Task 2: Implement local Plugin marks and readable cards

**Files:**
- Modify: `src/ui.html:430-451,2520-2610`
- Test: `tests/plugin-catalog-contract.test.cjs`

**Interfaces:** Consumes catalog entries containing a stable provider key. Produces `PLUGIN_BRAND_MARKS`, `brandMark(item)`, a 980px desktop catalog, and responsive cards without state-badge overlap.

- [ ] **Step 1: Add trusted local provider SVG mark mapping**

```js
const PLUGIN_BRAND_MARKS = { github: '<svg viewBox="0 0 24 24" aria-hidden="true">…</svg>' };
function brandMark(item) { return PLUGIN_BRAND_MARKS[item.brand] || '<span aria-hidden="true">'+esc(item.icon)+'</span>'; }
```

Every third-party item gets a `brand` key. First-party items keep their CommandBlock icon fallback.

- [ ] **Step 2: Change rendering and add containment styles**

```js
card.innerHTML = '<span class="plugin-icon">'+brandMark(item)+'</span><span class="plugin-copy">…</span><span class="plugin-state '+item.state+'">…</span>';
```

```css
.plugin-modal-card { width:min(980px,94vw); }
.plugin-copy { min-width:0; }
.plugin-desc { white-space:normal; display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; }
.plugin-state { flex-shrink:0; }
```

- [ ] **Step 3: Run test to verify it passes**

Run: `node --test tests/plugin-catalog-contract.test.cjs`

Expected: PASS.

- [ ] **Step 4: Commit the catalog implementation**

```bash
git add src/ui.html tests/plugin-catalog-contract.test.cjs
git commit -m "feat(plugins): add local colored provider marks"
```

### Task 3: Implement the upward account-management menu

**Files:**
- Modify: `src/ui.html:934-940,1141-1146,1239-1368,2819-2836`, `src/auth.rs`, `src/gui.rs`, `web/cloud-adapter.js`
- Test: `tests/account-menu-contract.test.cjs`, `tests/in-app-confirm-contract.test.cjs`

**Interfaces:** Consumes `showAccount(email)`, existing usage controls, device controls, and `/api/auth/logout`. Produces `setAccountMenuOpen(open)`, `openAccountProfile()`, `signOutCurrentAccount()`, `auth::send_password_recovery(agent, email)`, Desktop `POST /api/auth/recover`, and Web `authRecover()`.

- [ ] **Step 1: Add button and upward menu markup**

```html
<button id="accChip" class="accchip" type="button" aria-expanded="false">…</button>
<section id="accountMenu" class="account-menu" hidden>…</section>
```

The menu contains identity, Manage account, Connected devices, Usage & credit, password-reset guidance, and separated Sign out.

- [ ] **Step 2: Add account-menu behavior**

```js
function setAccountMenuOpen(open) {
  accountMenu.hidden = !open;
  accChip.setAttribute("aria-expanded", String(open));
}
document.addEventListener("pointerdown", event => {
  if (!accountMenu.contains(event.target) && !accChip.contains(event.target)) setAccountMenuOpen(false);
});
```

The menu opens upward and closes on Escape or outside click.

- [ ] **Step 3: Reuse safe existing actions**

```js
async function signOutCurrentAccount() {
  const confirmed = await confirmAction({ title:"ออกจากระบบ?", message:"…", approveLabel:"ออกจากระบบ", danger:true });
  if (!confirmed) return;
  await fetch("/api/auth/logout", { method:"POST", headers:{"Content-Type":"application/json"}, body:"{}" });
  location.reload();
}
```

Add `auth::send_password_recovery(agent, email)` using Supabase's `/auth/v1/recover` endpoint and route `POST /api/auth/recover`. Add the matching Web adapter `authRecover()` using `client.auth.resetPasswordForEmail`. Account name is stored locally per signed-in email, usage opens the existing credit controls, devices opens existing Connector/Remote controls, and the password entry reports success only after the matching local or Web route returns `{ok:true}`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `node --test tests/account-menu-contract.test.cjs tests/in-app-confirm-contract.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit the account menu**

```bash
git add src/ui.html tests/account-menu-contract.test.cjs tests/in-app-confirm-contract.test.cjs
git commit -m "feat(account): add account management menu"
```

### Task 4: Verify, version, release, and inspect

**Files:**
- Modify: `Cargo.toml`, `Cargo.lock`
- Test: all Node tests and Cargo test/build

**Interfaces:** Consumes Tasks 1-3. Produces a new release with the catalog and account-menu controls.

- [ ] **Step 1: Bump package version to `1.0.10`**

```toml
[package]
version = "1.0.10"
```

- [ ] **Step 2: Run static and automated verification**

Run: `git diff --check; node --test tests/*.test.cjs; cargo test; cargo build --release`

Expected: all Node/Rust tests pass and the release binary builds.

- [ ] **Step 3: Commit version and release work**

```bash
git add Cargo.toml Cargo.lock
git commit -m "build: release commandblock 1.0.10"
git push origin main
```

- [ ] **Step 4: Verify GitHub Actions release output**

```bash
gh run list --workflow release-windows.yml --limit 1
gh release view <new-tag> --json assets,url
```

Expected: successful workflow and a `CommandBlock-Windows-x64.zip` asset.
