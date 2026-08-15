# SESSION Rename and HTTPS Preview URL Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add durable SESSION renaming and render published HTTPS URLs passed to `/preview` inside CommandBlock.

**Architecture:** The shared HTML UI calls the existing conversation API. Rust implements that API for the EXE; the cloud adapter implements the same virtual route for the browser. Rust validates and stores a preview URL, and the existing Preview tab reads it from `/api/state`.

**Tech Stack:** Rust, Supabase REST, JavaScript, Node test runner, Cargo.

## Global Constraints

- Keep local `/preview` and the browser-open escape hatch unchanged.
- Only valid public `https://` URLs may set an external preview; failed validation must preserve the current preview.
- Trim SESSION titles and allow 1–80 characters only.
- Scope every cloud update to `user_id`.
- Leave `src/config.rs`, `src/diagnostics.rs`, `buff_session.json.bak`, and `cbweb.html` untouched.
- Release a new desktop version after tests and the release build pass.

---

### Task 1: Add the rename API

**Files:**
- Modify: `src/cloud.rs`, `src/gui.rs`, `web/cloud-adapter.js`
- Test: `tests/session-desktop-contract.test.cjs`, `tests/session-web-contract.test.cjs`

**Interfaces:**
- `cloud::rename_conversation(agent, conversation_id, title) -> Result<CloudConversation, String>` validates UUID and title, patches `conversations` with `id` and `user_id`, and returns the changed row.
- `POST /api/conversations/:id/rename` accepts `{ "title": string }` and returns `{ "conversation": { "id", "title", "updated_at" } }`.
- `renameConversation(session, id, title)` in the cloud adapter returns the same shape.

- [ ] **Step 1: Add failing contracts**

```js
assert.match(gui, /ends_with\("\/rename"\)/);
assert.match(cloud, /pub fn rename_conversation/);
assert.match(adapter, /async function renameConversation/);
assert.match(adapter, /\/api\/conversations\/\[\^\/\]\+\/rename/);
```

- [ ] **Step 2: Confirm they fail**

Run `node --test tests/session-desktop-contract.test.cjs tests/session-web-contract.test.cjs`; expect failure because no rename symbols exist.

- [ ] **Step 3: Implement minimal transport**

Use `PATCH conversations?id=eq.<id>&user_id=eq.<user>` with `{ title, updated_at: now }`, require one returned row, and surface Thai validation/errors. Implement the matching route in both the Rust server and cloud adapter.

- [ ] **Step 4: Confirm contracts pass**

Run `node --test tests/session-desktop-contract.test.cjs tests/session-web-contract.test.cjs`; expect pass.

- [ ] **Step 5: Commit the API**

Run `git add src/cloud.rs src/gui.rs web/cloud-adapter.js tests/session-desktop-contract.test.cjs tests/session-web-contract.test.cjs` then `git commit -m "feat(session): add durable rename API"`.

### Task 2: Add Rename SESSION UI

**Files:**
- Modify: `src/ui.html`
- Test: `tests/session-web-contract.test.cjs`

**Interfaces:**
- Add `renameSession` context action, `openRenameSessionDialog({ id, title })`, and `renameSession(id, title)`.
- The dialog is an in-app Obsidian modal, not `window.prompt()`.

- [ ] **Step 1: Add failing UI contracts**

```js
assert.match(ui, /id="renameSession"/);
assert.match(ui, /id="renameSessionDialog"/);
assert.match(ui, /function openRenameSessionDialog/);
assert.match(ui, /function renameSession/);
assert.match(ui, /\/rename/);
```

- [ ] **Step 2: Confirm the UI contract fails**

Run `node --test tests/session-web-contract.test.cjs`; expect failure because the menu action/modal are absent.

- [ ] **Step 3: Implement modal behavior**

Add the menu item after Pin. Prefill it with the selected title, disable Save for invalid trimmed input, submit on Enter, close on Escape/Cancel, call the route, refresh sessions, toast errors, and restore focus to the selected row.

- [ ] **Step 4: Confirm the UI contract passes**

Run `node --test tests/session-web-contract.test.cjs`; expect pass.

- [ ] **Step 5: Commit the UI**

Run `git add src/ui.html tests/session-web-contract.test.cjs` then `git commit -m "feat(session): rename sessions from context menu"`.

### Task 3: Support `/preview https://…`

**Files:**
- Modify: `Cargo.toml`, `Cargo.lock`, `src/tools.rs`, `src/gui.rs`, `src/ui.html`
- Test: `tests/preview-plugin-contract.test.cjs`

**Interfaces:**
- Add `tools::set_https_preview_url(raw_url: &str) -> Result<String, String>` and `tools::preview_command(argument: &str) -> String`.
- `try_command` passes the text following `/preview` to `preview_command`.
- UI recognizes `/preview` plus optional arguments and switches to Preview before waiting for a response.

- [ ] **Step 1: Add failing preview contracts**

```js
assert.match(tools, /pub fn set_https_preview_url/);
assert.match(gui, /"\/preview"\s*=>\s*tools::preview_command/);
assert.match(ui, /requestedPreview\s*=\s*\/\^\\\/preview\\b/);
```

- [ ] **Step 2: Confirm they fail**

Run `node --test tests/preview-plugin-contract.test.cjs`; expect failure because URL parsing is absent.

- [ ] **Step 3: Implement strict preview validation**

Add the Rust `url` dependency. Parse with `url::Url`; require HTTPS and a non-empty public host; reject malformed input, HTTP, localhost, loopback/private IP hosts, file URLs, and user-info URLs. Do not mutate the active preview on errors. Retain no-argument local reopen behavior.

- [ ] **Step 4: Confirm contracts pass**

Run `node --test tests/preview-plugin-contract.test.cjs`; expect pass.

- [ ] **Step 5: Commit preview support**

Run `git add Cargo.toml Cargo.lock src/tools.rs src/gui.rs src/ui.html tests/preview-plugin-contract.test.cjs` then `git commit -m "feat(preview): open HTTPS URLs in app"`.

### Task 4: Verify and publish

**Files:**
- Modify: `Cargo.toml`, `Cargo.lock`, `tests/session-version-contract.test.cjs`

- [ ] **Step 1: Set release version**

Set Cargo package version to `1.0.12` and change the contract to match `^version = "1\\.0\\.12"$`.

- [ ] **Step 2: Run full verification**

Run `node --test tests/*.test.cjs`, `cargo test`, and `cargo build --release`; expect all pass.

- [ ] **Step 3: Inspect and commit release metadata**

Run `git diff --check` and `git status --short`. Stage only Cargo metadata and the version contract, then commit `chore: release commandblock 1.0.12`. Verify the four user-owned local files remain unstaged.

- [ ] **Step 4: Push and monitor release**

Run `git push origin main` and verify the GitHub workflow publishes after package assets are ready.
