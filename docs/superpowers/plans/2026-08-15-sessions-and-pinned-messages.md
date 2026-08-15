# Sessions and Pinned Messages Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (\`- [ ]\`) syntax for tracking.

**Goal:** Add cloud-synchronized multi-session chat and pinned messages to CommandBlock web and desktop UI, then prepare a versioned update build without replacing the user's installed EXE.

**Architecture:** Supabase remains the source of truth: \`conversations\` identifies a Session and \`messages.is_pinned\` stores the pin state. A small browser helper owns deterministic session/message ordering and pin toggling. \`web/cloud-adapter.js\` accesses Supabase directly, while \`src/cloud.rs\` plus \`src/gui.rs\` expose equivalent data through the embedded desktop HTTP API; \`src/ui.html\` renders both clients from that common contract.

**Tech Stack:** Rust 2021, ureq, serde_json, embedded HTML/JavaScript, Supabase Postgres/RLS, Node built-in test runner, Cargo.

## Global Constraints

- Preserve all existing conversations and messages; a new session never deletes an older session.
- All database reads/writes remain scoped by \`user_id = auth.uid()\` through the existing RLS policies.
- Use \`created_at\` plus \`id\` as a stable ordering tie-breaker for messages and \`updated_at\` plus \`id\` for Session ordering.
- Only signed-in cloud accounts receive multi-session synchronization and shared pins; offline desktop behavior remains unchanged.
- Build version \`1.0.1\` to \`C:\\Codex\\target\\release\\commandblock.exe\`; do not overwrite \`C:\\Codex\\Commandblock.exe\`.
- Do not stage or alter existing user-owned changes in \`src/config.rs\`, \`src/diagnostics.rs\`, or \`buff_session.json.bak\`.

---

### Task 1: Add durable pin storage and efficient Session reads

**Files:**
- Create: \`supabase/migrations/202608150001_sessions_and_pins.sql\`
- Create: \`tests/session-pin-schema-contract.test.cjs\`

**Interfaces:**
- Produces \`public.messages.is_pinned boolean not null default false\`.
- Produces index \`messages_conversation_pin_created_idx (conversation_id, is_pinned desc, created_at asc, id asc)\`.
- Existing \`users_manage_own_messages\` policy remains the authorization boundary.

- [ ] **Step 1: Write the failing schema contract test**

\`\`\`js
test('session and pin migration keeps pins durable and owner-scoped', () => {
  const sql = fs.readFileSync(migration, 'utf8');
  assert.match(sql, /alter table public\.messages add column if not exists is_pinned boolean not null default false/i);
  assert.match(sql, /messages_conversation_pin_created_idx/i);
  assert.match(sql, /auth\.uid\(\).*user_id/i);
});
\`\`\`

- [ ] **Step 2: Run the test to verify it fails**

Run: \`node --test tests/session-pin-schema-contract.test.cjs\`

Expected: FAIL because the migration file does not yet exist.

- [ ] **Step 3: Add the migration**

\`\`\`sql
alter table public.messages
  add column if not exists is_pinned boolean not null default false;

create index if not exists messages_conversation_pin_created_idx
  on public.messages (conversation_id, is_pinned desc, created_at asc, id asc);
\`\`\`

Leave the existing RLS policy unchanged; add a SQL comment naming \`auth.uid() = user_id\` as the enforced ownership rule so the migration documents its dependency.

- [ ] **Step 4: Run the test to verify it passes**

Run: \`node --test tests/session-pin-schema-contract.test.cjs\`

Expected: PASS with one test and zero failures.

- [ ] **Step 5: Commit the migration and contract test**

\`\`\`powershell
git add supabase/migrations/202608150001_sessions_and_pins.sql tests/session-pin-schema-contract.test.cjs
git commit -m "feat: persist pinned chat messages"
\`\`\`

### Task 2: Add deterministic client-side Session and pin helpers

**Files:**
- Create: \`web/session-store.js\`
- Create: \`tests/session-store.test.cjs\`
- Modify: \`scripts/build-web.mjs\`

**Interfaces:**
- Produces \`CommandBlockSessions.compareSessions(a, b)\` returning newest-first Session order.
- Produces \`CommandBlockSessions.sortMessages(rows)\` returning \`created_at\`, then \`id\`, ascending order.
- Produces \`CommandBlockSessions.togglePinned(row)\` returning a copy with \`is_pinned\` inverted.
- \`scripts/build-web.mjs\` copies \`web/session-store.js\` to \`site/assets/session-store.js\`.

- [ ] **Step 1: Write the failing helper tests**

\`\`\`js
test('sessions order newest update first with id as a stable tie breaker', () => {
  const ordered = rows.sort(api.compareSessions).map((row) => row.id);
  assert.deepEqual(ordered, ['c', 'b', 'a']);
});

test('pinning returns a new row and keeps the original row unchanged', () => {
  const before = { id: 'm1', is_pinned: false };
  assert.deepEqual(api.togglePinned(before), { id: 'm1', is_pinned: true });
  assert.equal(before.is_pinned, false);
});
\`\`\`

- [ ] **Step 2: Run the test to verify it fails**

Run: \`node --test tests/session-store.test.cjs\`

Expected: FAIL because \`web/session-store.js\` is absent.

- [ ] **Step 3: Implement the helper and web build copy**

\`\`\`js
function compareSessions(a, b) {
  const time = String(b.updated_at || '').localeCompare(String(a.updated_at || ''));
  return time || String(b.id || '').localeCompare(String(a.id || ''));
}
function togglePinned(row) { return { ...row, is_pinned: !row.is_pinned }; }
\`\`\`

Use the same \`created_at\`/\`id\` ascending comparator currently supplied by \`web/chat-timeline.js\` for messages. Export only \`compareSessions\`, \`sortMessages\`, and \`togglePinned\` on \`globalThis.CommandBlockSessions\`.

- [ ] **Step 4: Run the test to verify it passes**

Run: \`node --test tests/session-store.test.cjs\`

Expected: PASS with both ordering and immutable-toggle behavior green.

- [ ] **Step 5: Commit the helper**

\`\`\`powershell
git add web/session-store.js scripts/build-web.mjs tests/session-store.test.cjs
git commit -m "feat: add session ordering helpers"
\`\`\`

### Task 3: Implement browser Session switching and contextual pins

**Files:**
- Modify: \`web/cloud-adapter.js\`
- Modify: \`src/ui.html\`
- Modify: \`web/index.html\` if it owns standalone browser markup
- Create: \`tests/session-web-contract.test.cjs\`

**Interfaces:**
- Consumes \`CommandBlockSessions\` from Task 2.
- \`cloud-adapter\` exposes cloud HTTP adapter operations \`listConversations()\`, \`createConversation()\`, \`selectConversation(id)\`, \`toggleMessagePin(id, nextPinned)\`, and \`cloudConversationSync()\` using the selected conversation.
- The UI invokes \`GET /api/conversations\`, \`POST /api/conversations\`, \`POST /api/messages/:id/pin\`, and \`GET /api/conversation/sync?conversation_id=:id\` for desktop; the standalone adapter performs equivalent Supabase calls.

- [ ] **Step 1: Write failing contract tests**

\`\`\`js
test('browser adapter creates and selects explicit sessions instead of always selecting the newest', () => {
  assert.match(adapter, /createConversation\(session\)/);
  assert.match(adapter, /conversationId = id/);
  assert.doesNotMatch(adapter, /conversationId = await activeConversationForUser\(session\) \|\| conversationId/);
});

test('desktop UI labels its history panel SESSION and provides new-session and context pin controls', () => {
  assert.match(ui, />SESSION</);
  assert.match(ui, /id="newSession"/);
  assert.match(ui, /Pin message/);
  assert.match(ui, /contextmenu/);
});
\`\`\`

- [ ] **Step 2: Run the tests to verify they fail**

Run: \`node --test tests/session-web-contract.test.cjs\`

Expected: FAIL because explicit session methods and the Session UI do not exist.

- [ ] **Step 3: Implement selected-session behavior in \`web/cloud-adapter.js\`**

Replace the implicit newest-conversation lookup with a selected \`conversationId\`, restored per user only when it still belongs to that user. Query Session rows as \`id,title,model_id,created_at,updated_at\`; insert a new \`{ user_id, title: 'แชทใหม่', model_id: MODEL }\` row on request; query messages as \`id,role,content,created_at,is_pinned\`; and patch the selected owned message with \`{ is_pinned: nextPinned }\`. Each save updates only the active conversation's \`updated_at\`.

- [ ] **Step 4: Implement UI controls in \`src/ui.html\`**

Load \`/assets/session-store.js\`. Rename the center panel title to \`SESSION\`, add button \`#newSession\`, render selectable Session entries, then rebuild the transcript from the selected sync response. Attach \`contextmenu\` only to completed message elements, position a \`#messageContextMenu\` inside the viewport, and use a pin/unpin action. Render a compact \`#pinnedMessages\` area above ordinary chronological transcript entries without removing any original entry. Close the menu on \`Escape\`, document pointer-down, or action completion. On errors, preserve the current transcript and call the existing Thai \`toast\` helper.

- [ ] **Step 5: Run the tests to verify they pass**

Run: \`node --test tests/session-web-contract.test.cjs\`

Expected: PASS with explicit selection, Session UI, and pin context-menu contracts green.

- [ ] **Step 6: Commit the browser/UI work**

\`\`\`powershell
git add web/cloud-adapter.js src/ui.html web/index.html tests/session-web-contract.test.cjs
git commit -m "feat: add sessions and pinned messages UI"
\`\`\`

### Task 4: Expose the Session contract through the desktop Rust API

**Files:**
- Modify: \`src/cloud.rs\`
- Modify: \`src/gui.rs\`
- Create: \`tests/session-desktop-contract.test.cjs\`

**Interfaces:**
- \`CloudMessage\` gains \`is_pinned: bool\` and continues to carry \`id\`, \`role\`, \`content\`, and \`created_at\`.
- \`cloud::list_conversations(agent) -> Result<Vec<CloudConversation>, String>\` returns the signed-in user's rows in \`updated_at.desc,id.desc\` order.
- \`cloud::create_conversation(agent, model) -> Result<CloudConversation, String>\` inserts a fresh user-owned session.
- \`cloud::pull_conversation(agent, conversation_id) -> Result<Vec<CloudMessage>, String>\` scopes the request to that conversation.
- \`cloud::set_message_pin(agent, message_id, is_pinned) -> Result<(), String>\` patches an owner-scoped message.

- [ ] **Step 1: Write failing API contract tests**

\`\`\`js
test('desktop API exposes explicit Session and pin endpoints', () => {
  assert.match(gui, /\("GET", "\/api\/conversations"\)/);
  assert.match(gui, /\("POST", "\/api\/conversations"\)/);
  assert.match(gui, /\/api\/messages\/.*\/pin/);
  assert.match(cloud, /is_pinned/);
});

test('cloud reads pin state and scopes selected conversation requests', () => {
  assert.match(cloud, /select=id,role,content,created_at,is_pinned/);
  assert.match(cloud, /conversation_id=eq\.\{conversation_id\}/);
});
\`\`\`

- [ ] **Step 2: Run the tests to verify they fail**

Run: \`node --test tests/session-desktop-contract.test.cjs\`

Expected: FAIL because the endpoints and \`is_pinned\` field do not exist.

- [ ] **Step 3: Implement the cloud functions**

Add serializable \`CloudConversation\` and extend \`CloudMessage\`. Use the current \`auth::refresh_token\` and \`authed\` request helpers. For message patches use \`PATCH /rest/v1/messages?id=eq:{message_id}&user_id=eq:{pair.user_id}\` with JSON \`{ \"is_pinned\": is_pinned }\`; reject non-success responses with Thai errors. Do not change \`push\` behavior except to use the active session ID held by GUI state.

- [ ] **Step 4: Implement HTTP routing and active-session state**

In \`src/gui.rs\`, add request handlers for list/create/select/pin. Parse \`conversation_id\` from the query string using existing request parsing conventions; validate a UUID-shaped nonempty id before calling cloud. Return exactly the rows required by Task 3 and do not leak provider keys or another user's records. When a session is selected, replace only in-memory chat history with its user/assistant messages and persist it under the session/account cache path.

- [ ] **Step 5: Run desktop contract tests and Rust test suite**

Run: \`node --test tests/session-desktop-contract.test.cjs; cargo test\`

Expected: all Node assertions pass and Cargo exits 0.

- [ ] **Step 6: Commit the Rust API work**

\`\`\`powershell
git add src/cloud.rs src/gui.rs tests/session-desktop-contract.test.cjs
git commit -m "feat: synchronize sessions and pins in desktop app"
\`\`\`

### Task 5: Bump version, build, verify, and publish source without replacing installed EXE

**Files:**
- Modify: \`Cargo.toml\`
- Modify: \`Cargo.lock\` if Cargo updates it
- Modify: release/update metadata files discovered by \`rg -n "1\\.0\\.0|build-" .github scripts src\`
- Create: \`tests/session-version-contract.test.cjs\`

**Interfaces:**
- \`env!(\"CARGO_PKG_VERSION\")\` resolves to \`1.0.1\`.
- Built artifact is \`C:\\Codex\\target\\release\\commandblock.exe\`.
- Existing installed file \`C:\\Codex\\Commandblock.exe\` remains unmodified.

- [ ] **Step 1: Write the failing version/build contract test**

\`\`\`js
test('the update increments the desktop package version', () => {
  const cargo = fs.readFileSync('Cargo.toml', 'utf8');
  assert.match(cargo, /^version = "1\.0\.1"$/m);
});
\`\`\`

- [ ] **Step 2: Run the test to verify it fails**

Run: \`node --test tests/session-version-contract.test.cjs\`

Expected: FAIL while Cargo still declares \`1.0.0\`.

- [ ] **Step 3: Bump version and update only release metadata that declares an application version**

Set Cargo's package version to \`1.0.1\`. Keep the version label sourced from \`crate::VERSION\`; do not hard-code a second UI version. Preserve the current timestamp/build comparison logic in \`src/update.rs\`.

- [ ] **Step 4: Run the full fresh verification set**

Run: \`node --test tests/*.test.cjs; cargo test; $env:CARGO_TARGET_DIR='C:\\Codex\\target'; cargo build --release; (Get-Item 'C:\\Codex\\Commandblock.exe').LastWriteTimeUtc; (Get-Item 'C:\\Codex\\target\\release\\commandblock.exe').VersionInfo.FileVersion\`

Expected: all Node and Cargo tests exit 0; release build exits 0; installed EXE timestamp remains unchanged; target EXE reports \`1.0.1\` or is verified from Cargo metadata when Windows file metadata is unavailable.

- [ ] **Step 5: Commit and push only owned feature files**

\`\`\`powershell
git add Cargo.toml Cargo.lock tests/session-version-contract.test.cjs .github scripts
git commit -m "release: prepare commandblock v1.0.1"
git push origin main
\`\`\`

Before staging, inspect \`git status --short\` and omit \`src/config.rs\`, \`src/diagnostics.rs\`, and \`buff_session.json.bak\` unless their owner explicitly asks to include them.

## Plan self-review

- Spec coverage: Task 1 stores pins; Task 2 provides deterministic reusable helpers; Task 3 delivers the browser and embedded UI; Task 4 makes the desktop API use the same source of truth; Task 5 bumps, builds, and leaves the installed EXE untouched.
- Placeholder scan: no unresolved implementation markers are present; every task has concrete files, test commands, expected red/green outcome, and a commit command.
- Interface consistency: \`CloudMessage.is_pinned\`, \`conversation_id\`, and \`toggleMessagePin(id, nextPinned)\` use identical names in browser, desktop API, and UI tasks.

