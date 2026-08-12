# Remote Device Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give an account owner safe visibility and control over their Connector and Remote PCs without widening remote-control authority.

**Architecture:** A Supabase migration adds owner-scoped audit records and supporting indexes. CommandBlock Web uses existing authenticated Supabase client calls to list, rename, revoke, and audit devices. Sidecars keep the existing confirmation, expiry, and user ownership constraints.

**Tech Stack:** Supabase Postgres/RLS, static JavaScript adapter, Rust connector/remote sidecars.

## Global Constraints

- New tables, policies, and indexes are applied by migration `202608120004_device_management.sql`.
- All access predicates use `(select auth.uid()) = user_id`.
- Audit rows contain metadata only: device, mode, action, session id, and timestamp.
- Revocation cascades pending commands/sessions and requires fresh registration.

---

### Task 1: Define owner-scoped device audit schema

**Files:**
- Create: `supabase/migrations/202608120004_device_management.sql`
- Test: `tests/device-management-schema.test.cjs`

**Interfaces:**
- Produces `public.device_audit_events(id, user_id, device_kind, device_id, remote_session_id, action, mode, created_at)`.

- [ ] **Step 1: Write failing schema contract test**

```js
assert.match(sql, /create table public\.device_audit_events/);
assert.match(sql, /enable row level security/);
assert.match(sql, /\(select auth\.uid\(\)\) = user_id/);
assert.match(sql, /device_audit_events_owner_created_idx/);
```

- [ ] **Step 2: Run it**

Run: `node --test tests/device-management-schema.test.cjs`

Expected: FAIL because the migration does not exist.

- [ ] **Step 3: Implement migration**

Create the audit table with UUID primary key, `user_id uuid not null references auth.users(id) on delete cascade`, checked enum-like text values, and `created_at timestamptz not null default now()`. Enable RLS; grant only select/insert to owners; add `(user_id, created_at desc)` index. Add delete policies on existing device tables that remain owner scoped.

- [ ] **Step 4: Re-run test**

Run: `node --test tests/device-management-schema.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add supabase/migrations/202608120004_device_management.sql tests/device-management-schema.test.cjs && git commit -m "feat(remote): add owner device audit schema"`

### Task 2: Record minimal Remote lifecycle audit events

**Files:**
- Modify: `src/remote.rs`, `web/cloud-adapter.js`
- Test: `src/remote.rs`, `tests/remote-device-management.test.cjs`

**Interfaces:**
- Produces `record_audit(agent, token, AuditEvent) -> Result<(), String>` and web action events `requested`, `connected`, `closed`.

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn audit_payload_never_contains_screen_or_input_content() {
    let payload = AuditEvent::requested("device", "control", "session").json();
    assert!(payload.get("frame").is_none());
    assert!(payload.get("key").is_none());
}
```

```js
assert.match(adapter, /device_audit_events/);
assert.match(adapter, /action: 'requested'/);
```

- [ ] **Step 2: Run tests**

Run: `cargo test remote::tests::audit_payload_never_contains_screen_or_input_content; node --test tests/remote-device-management.test.cjs`

Expected: FAIL because audit types/events do not exist.

- [ ] **Step 3: Implement audit inserts**

Write one owner-authenticated audit row at request, approval/denial, and close. Error to audit must not abort a confirmed session; log only a generic local warning. Web adapter records request/close actions for browser-owned lifecycle points.

- [ ] **Step 4: Re-run tests**

Run: `cargo test remote::tests; node --test tests/remote-device-management.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add src/remote.rs web/cloud-adapter.js tests/remote-device-management.test.cjs && git commit -m "feat(remote): audit session lifecycle"`

### Task 3: Build My devices controls

**Files:**
- Modify: `web/cloud-adapter.js`, `web/index.html`
- Test: `tests/remote-device-management.test.cjs`

**Interfaces:**
- Produces controls `#cb-devices-open`, `#cb-device-rename`, `#cb-device-revoke` and owner-filtered queries.

- [ ] **Step 1: Write failing UI contract test**

```js
assert.match(adapter, /id = 'cb-devices-open'/);
assert.match(adapter, /cb-device-rename/);
assert.match(adapter, /cb-device-revoke/);
assert.match(adapter, /\.eq\('user_id', session\.user\.id\)/);
```

- [ ] **Step 2: Run it**

Run: `node --test tests/remote-device-management.test.cjs`

Expected: FAIL because the controls do not exist.

- [ ] **Step 3: Implement the modal**

List Connector and Remote devices separately with `last_seen_at`. Rename accepts 1–80 trimmed characters, then updates only the selected owner row. Revoke requires a confirmation dialog, deletes only that owner’s device row, and updates pending associated sessions to `closed` before deletion. Render the last 50 owner audit events.

- [ ] **Step 4: Re-run UI and security contracts**

Run: `node --test tests/remote-device-management.test.cjs tests/remote-session-contract.test.cjs tests/connector-schema-contract.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add web/cloud-adapter.js web/index.html tests/remote-device-management.test.cjs && git commit -m "feat(remote): manage owner devices from web"`
