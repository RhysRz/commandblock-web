# CommandBlock Secure Cloud Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver self-updating CommandBlock binaries, contextual Cloud chat and usage reporting, account/device administration, and device-bound Remote PC approval.

**Architecture:** Keep the Rust desktop executable as the local authority for files, updater staging, and Remote host approval. Keep the static web app as the authenticated client through Supabase. Supabase stores only owner-scoped metadata, history, usage, audit records, and temporary Remote negotiation state; it never stores DeepSeek keys or Remote device secrets.

**Tech Stack:** Rust 2021, ureq, keyring, Wry UI, static JavaScript, Supabase Postgres/RLS, Supabase Edge Functions (Deno), GitHub Actions, Inno Setup.

## Global Constraints

- Do not add a Lite installer or code-signing certificate.
- Keep API keys and Remote device secrets out of Git and Supabase tables.
- All new database rows must be owner-scoped by RLS.
- Add a failing test before every production behavior change.
- Preserve Windows-only desktop support and the existing purple UI.

---

### Task 1: Add secure Cloud/Remote database schema

**Files:**
- Create: `supabase/migrations/202608120005_secure_cloud_suite.sql`
- Test: `tests/secure-cloud-schema.test.cjs`

**Interfaces:**
- Produces `public.usage_events` and Remote session approval fields used by web adapter, Edge Function, and `src/remote.rs`.

- [ ] Write a Node contract test asserting `usage_events`, `approval_code_hash`, `approval_code_input`, owner RLS, and daily usage index occur in migration.
- [ ] Run `node --test tests/secure-cloud-schema.test.cjs` and confirm it fails because the migration is absent.
- [ ] Add idempotent SQL creating the table, indexes, security policies, remote columns, and audit-safe constraints.
- [ ] Re-run the test and commit `feat(db): add secure cloud suite schema`.

### Task 2: Make Cloud chat contextual and persist usage

**Files:**
- Modify: `web/cloud-adapter.js`
- Modify: `supabase/functions/chat/index.ts`
- Test: `tests/cloud-chat-context.test.cjs`

**Interfaces:**
- `cloudChat()` sends `{model, baseUrl, apiKey, messages, conversationId}`.
- Edge Function returns `{content, usage}` and inserts an owner `usage_events` row.

- [ ] Write a failing contract test requiring a bounded `messages` query, no API-key persistence, usage event insertion, and `data.usage` SSE.
- [ ] Run its Node test and confirm failure.
- [ ] Implement bounded message loading, input validation (roles/length/count), DeepSeek call with history, usage insert, and explicit estimated fallback.
- [ ] Re-run test and commit `feat(cloud): retain chat context and usage`.

### Task 3: Add account, device, and usage dashboards

**Files:**
- Modify: `web/cloud-adapter.js`
- Modify: `src/ui.html`
- Test: `tests/account-device-dashboard.test.cjs`

**Interfaces:**
- `mountAccount()` exposes profile update, local/global logout, owner device list and usage day/month query.

- [ ] Write a failing contract test for `mountAccount`, `signOut({ scope: 'global' })`, profiles update, and daily/monthly usage queries.
- [ ] Run test and confirm failure.
- [ ] Implement responsive Account modal and integrate it with existing device controls and token display.
- [ ] Re-run test and commit `feat(web): add account and usage dashboard`.

### Task 4: Update staged updater binary itself

**Files:**
- Modify: `src/update.rs`
- Modify: `src/bin/commandblock-updater.rs`
- Test: `tests/updater-self-replace.test.cjs`
- Test: `src/update.rs` unit tests

**Interfaces:**
- `stage_release()` extracts all three binaries.
- `launch_staged_update()` launches updater which writes and starts a temporary replacement script then exits.

- [ ] Write failing Rust/contract tests requiring the updater file in staging and a deferred self-replacement command.
- [ ] Run target tests and confirm failure.
- [ ] Implement staging and a quote-safe temporary script that waits for the main PID, copies all three binaries, deletes itself, and relaunches CommandBlock.
- [ ] Re-run tests and commit `feat(update): self-update updater sidecar`.

### Task 5: Add device-bound Remote approval

**Files:**
- Modify: `src/remote.rs`
- Modify: `web/cloud-adapter.js`
- Test: `src/remote.rs` unit tests
- Test: `tests/remote-device-approval.test.cjs`

**Interfaces:**
- `remote::device_secret()` stores per-host secret in Windows Credential Manager.
- A Remote host saves a SHA-256 approval-code hash and accepts only a matching browser input before answering WebRTC.

- [ ] Write failing Rust tests for six-digit code validation/hash matching/expiry and a contract test proving no device secret is in browser code.
- [ ] Run tests and confirm failure.
- [ ] Implement local secret generation, code-hash exchange, acceptance/denial/timeout audit events, and browser PIN prompt/state.
- [ ] Re-run tests and commit `feat(remote): require device-bound approval`.

### Task 6: Package and deploy verification

**Files:**
- Modify: `.github/workflows/deploy-pages.yml` only if release verification needs updates
- Modify: `installer/build-installer.ps1` only if binary completeness check needs updates
- Test: `tests/release-package-completeness.test.cjs`

- [ ] Extend the release contract test to require all three binaries and installer script parity.
- [ ] Run test and confirm failure if an artifact is missing.
- [ ] Make only required package changes.
- [ ] Run `cargo test`, all Node contract tests, `node scripts/build-web.mjs`, `supabase db push --linked --dry-run`, deploy the Edge Function/migration, commit, push, then create a verified Setup artifact.
