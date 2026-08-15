# CommandBlock Work Strip and Conversation Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give CommandBlock EXE and web the same expandable work/Todo interface, mobile status drawer, and same-account conversation mirroring.

**Architecture:** `src/ui.html` remains the canonical renderer used directly by the EXE and copied into GitHub Pages by `scripts/build-web.mjs`. A small UI state layer will join existing Todo, file-change, and tool events into one per-turn Work Strip. The Cloud adapter will expose `update_plan` and a server-backed active conversation; the desktop backend will add authenticated Supabase REST helpers and a `/api/conversation/sync` endpoint so each client reads and writes the same account-scoped transcript.

**Tech Stack:** Rust + ureq + serde_json, vanilla HTML/CSS/JavaScript, Supabase Auth/PostgREST, Node built-in test runner.

## Global Constraints

- `src/ui.html` is the only shared visual source; do not edit `site/index.html` manually.
- Preserve the current `details.think` markup, CSS, and interaction unchanged.
- Use the existing `messages` and `conversations` tables, scoped by `user_id`; never synchronize API keys, local folders, terminal output, or Remote PC credentials.
- Keep mobile media breakpoint at `max-width: 900px`; the composer stays visible while the drawer is open.
- Do not stage unrelated `src/config.rs`, `src/diagnostics.rs`, or `buff_session.json.bak` changes.

---

### Task 1: Establish Work Strip and mobile drawer contracts

**Files:**
- Create: `tests/work-strip-ui.test.cjs`
- Modify: `src/ui.html:223-254, 1020-1050, 1489-1538, 1634-1685`

**Interfaces:**
- Consumes: tool SSE `{ name, args }` and change SSE `{ path, status, added, deleted }`.
- Produces: `setTodos(planText, bubble)`, `renderWorkStrip(bubble)`, `recordTurnChange(change, bubble)`, and `setMobileStatusDrawer(open)`.

- [ ] **Step 1: Write the failing UI contract test**

```js
test('canonical UI renders an expandable work strip and a mobile status drawer', () => {
  assert.match(html, /class="workstrip"/);
  assert.match(html, /id="mobileStatusToggle"/);
  assert.match(html, /id="mobileStatusDrawer"/);
  assert.match(html, /function renderWorkStrip\(bub\)/);
  assert.match(html, /function setMobileStatusDrawer\(open\)/);
  assert.match(html, /\.workstrip summary/);
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `node --test tests/work-strip-ui.test.cjs`

Expected: FAIL because `workstrip` and mobile drawer elements do not exist.

- [ ] **Step 3: Implement the minimal shared UI**

```js
function renderWorkStrip(bub) {
  const files = [...turnChanges.values()];
  if (!todos.length && !files.length && !turnActivity) return;
  // Reuse one <details class="workstrip"> per assistant bubble.
  // Its summary contains activity, Todo progress, and + / - totals.
  // Its body contains the Todo checklist followed by changed-file details.
}

function setMobileStatusDrawer(open) {
  document.body.classList.toggle('mobstatus', Boolean(open));
  mobileStatusToggle.setAttribute('aria-expanded', String(Boolean(open)));
}
```

Move the existing Todo list and `renderChangeBox` content into `renderWorkStrip`; retain the `think` functions without changes. Add a fixed mobile hamburger adjacent to the composer and a right drawer containing the existing status controls. On desktop, keep the original status bar visible and hide the mobile trigger/drawer.

- [ ] **Step 4: Run the UI contract and existing scroll test**

Run: `node --test tests/work-strip-ui.test.cjs tests/chat-scroll-layout.test.cjs`

Expected: PASS with two tests.

- [ ] **Step 5: Commit the focused UI contract**

```bash
git add tests/work-strip-ui.test.cjs src/ui.html
git commit -m "feat: add expandable work strip and mobile status drawer"
```

### Task 2: Give Cloud tasks real Todo updates

**Files:**
- Create: `tests/cloud-work-plan-contract.test.cjs`
- Modify: `web/cloud-adapter.js:111-196, 265-281`
- Modify: `src/ui.html:1676-1684`

**Interfaces:**
- Consumes: OpenAI-compatible tool call named `update_plan` with `{ plan: string }`.
- Produces: a tool success object `{ ok: true, plan }` and Work Strip Todo updates from the matching tool event.

- [ ] **Step 1: Write the failing Cloud plan contract test**

```js
test('cloud agent exposes the same update_plan tool as the desktop agent', () => {
  assert.match(source, /name:\s*'update_plan'/);
  assert.match(source, /description:.*แผนงาน/s);
  assert.match(source, /if \(name === 'update_plan'\) return \{ ok: true, plan: args\.plan \|\| '' \}/);
  assert.match(source, /อัปเดต Todo เมื่อเริ่มงานและเมื่อขั้นตอนเสร็จ/);
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `node --test tests/cloud-work-plan-contract.test.cjs`

Expected: FAIL because Cloud `AGENT_TOOLS` does not contain `update_plan`.

- [ ] **Step 3: Add the no-connector plan tool and prompt rule**

```js
{
  type: 'function', function: {
    name: 'update_plan',
    description: 'บันทึกแผนงานเป็นข้อความลำดับขั้นเพื่อแสดง Todo ให้ผู้ใช้',
    parameters: { type: 'object', properties: { plan: { type: 'string' } }, required: ['plan'] },
  },
}

if (name === 'update_plan') return { ok: true, plan: args.plan || '' };
```

Append the exact Thai plan-use rule to `agentSystem`. When the UI receives the event, call `setTodos` and then `renderWorkStrip`; for any other tool, update the activity label and rerender rather than blindly marking multiple Todo items complete.

- [ ] **Step 4: Run Cloud contracts**

Run: `node --test tests/cloud-work-plan-contract.test.cjs tests/cloud-adapter-contract.test.cjs tests/cloud-chat-context.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit Cloud Todo support**

```bash
git add tests/cloud-work-plan-contract.test.cjs web/cloud-adapter.js src/ui.html
git commit -m "feat: show cloud agent plans in work strip"
```

### Task 3: Add safe, duplicate-free same-account transcript sync

**Files:**
- Create: `tests/conversation-sync-contract.test.cjs`
- Modify: `src/auth.rs:10-218`
- Modify: `src/gui.rs:1025-1100`
- Modify: `src/ui.html:1231-1253, 1624-1695`
- Modify: `web/cloud-adapter.js:84-108, 305-314`

**Interfaces:**
- Consumes: authenticated Supabase access token and user id from `auth::refresh_token`.
- Produces: `auth::sync_conversation(agent, request) -> Result<Value, String>` and JSON endpoint `GET /api/conversation/sync?after=<timestamp>` / `POST /api/conversation/sync`.
- Response: `{ conversation_id, messages: [{ id, role, content, created_at, client_message_id }], updated_at }`.

- [ ] **Step 1: Write the failing transcript sync contract test**

```js
test('desktop and cloud clients use an account-scoped active conversation sync path', () => {
  assert.match(gui, /"\/api\/conversation\/sync"/);
  assert.match(auth, /pub fn sync_conversation\(/);
  assert.match(adapter, /activeConversationForUser/);
  assert.match(adapter, /client_message_id/);
  assert.match(ui, /startConversationSync\(\)/);
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `node --test tests/conversation-sync-contract.test.cjs`

Expected: FAIL because the sync endpoint and helpers do not exist.

- [ ] **Step 3: Implement user-scoped PostgREST helpers and endpoints**

```rust
pub fn sync_conversation(agent: &ureq::Agent, request: &Value) -> Result<Value, String> {
    let token = refresh_token(agent)?;
    // Send apikey plus `Authorization: Bearer <access token>` to PostgREST.
    // Read the newest conversation for token.user_id, or insert one when POSTing the first message.
    // Always filter both tables with user_id=eq.<token.user_id>.
}
```

Implement `GET` to return the latest active account conversation plus messages newer than `after`, and `POST` to insert exactly one message using a supplied UUID-like `client_message_id`. If the same id already exists, return that row instead of inserting another. Add `client_message_id text unique` through a new idempotent migration and include it in the Cloud client inserts.

In `src/ui.html`, make `startConversationSync()` start only after authenticated login, poll no faster than every 3 seconds while visible, merge by `client_message_id`/row id, and stop on logout. A remote message creates a normal assistant/user bubble without calling `/api/chat`; it never causes a second model call.

In `web/cloud-adapter.js`, resolve the most recently updated account conversation before creating a new one, save the server conversation id locally only as a cache, and include a generated `client_message_id` when inserting messages.

- [ ] **Step 4: Run transcript contracts plus Rust format/check**

Run: `node --test tests/conversation-sync-contract.test.cjs tests/web-auth-contract.test.cjs && cargo fmt --check && cargo check`

Expected: PASS and cargo exits 0.

- [ ] **Step 5: Commit transcript synchronization**

```bash
git add tests/conversation-sync-contract.test.cjs src/auth.rs src/gui.rs src/ui.html web/cloud-adapter.js supabase/migrations
git commit -m "feat: sync same-account conversations across devices"
```

### Task 4: Build, smoke-test, and release the shared UI

**Files:**
- Modify: `tests/canonical-web-build.test.cjs`
- Generated: `site/index.html`, `site/cloud-adapter.js`

**Interfaces:**
- Consumes: canonical UI source and web adapter.
- Produces: a GitHub Pages bundle with the same Work Strip and mobile drawer as the EXE.

- [ ] **Step 1: Extend the failing canonical build assertion**

```js
assert.match(output, /class="workstrip"/);
assert.match(output, /id="mobileStatusDrawer"/);
assert.match(output, /function startConversationSync\(\)/);
```

- [ ] **Step 2: Run the canonical build test to verify it fails before the implementation is copied**

Run: `node --test tests/canonical-web-build.test.cjs`

Expected: FAIL until the canonical source contains all three contracts.

- [ ] **Step 3: Build the website and inspect generated assets**

Run: `node scripts/build-web.mjs && node --check web/cloud-adapter.js && node --check web/chat-recovery.js`

Expected: exit 0 and generated `site/index.html`, `site/cloud-adapter.js`, and `site/chat-recovery.js`.

- [ ] **Step 4: Run the complete relevant verification set**

Run: `node --test tests/work-strip-ui.test.cjs tests/cloud-work-plan-contract.test.cjs tests/conversation-sync-contract.test.cjs tests/cloud-adapter-contract.test.cjs tests/cloud-chat-context.test.cjs tests/chat-scroll-layout.test.cjs tests/canonical-web-build.test.cjs && cargo fmt --check && cargo check`

Expected: all Node tests pass and both Rust commands exit 0.

- [ ] **Step 5: Commit and push the release-ready bundle**

```bash
git add src/ui.html web/cloud-adapter.js src/auth.rs src/gui.rs supabase/migrations tests site
git commit -m "feat: unify work UI and account chat sync"
git push origin main
```

## Self-review

- Work Strip, compact change count, checklists, unchanged Thinking, and the mobile right drawer are covered by Tasks 1 and 4.
- Cloud Todo parity is covered by Task 2.
- Same-account conversation reading, idempotent insertion, active-conversation selection, polling, and no-duplicate rendering are covered by Task 3.
- Shared source and source-control safety are global constraints.
- No placeholder, inconsistent function name, or unassigned specification requirement remains.
