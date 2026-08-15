# Timestamped Chat Segments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show AI work as chronological, separately rendered messages while keeping active Thinking and Work Strip at the bottom.

**Architecture:** Preserve Supabase message identity and creation time through the Rust sync route. The shared UI turns each tool-bounded model response into a completed assistant segment, with a separate bottom work container. The cloud adapter mirrors the desktop protocol and persists each assistant phase separately.

**Tech Stack:** Rust, embedded HTML/JavaScript, Supabase REST, browser Fetch/SSE, Node built-in test runner.

## Global Constraints

- No database migration: `messages.id` and `messages.created_at` already exist.
- Sort ties deterministically by database ID after timestamp.
- Do not discard visible AI text when a tool call begins.
- Keep the current Work Strip and Thinking treatment, but only the active work UI occupies the bottom position during a turn.

---

### Task 1: Preserve cloud row metadata through the desktop sync boundary

**Files:**
- Modify: `src/cloud.rs`
- Modify: `src/gui.rs`
- Test: `tests/timestamped-chat-segments.test.cjs`

**Interfaces:**
- Produces `CloudMessage { id, role, content, created_at }` for the sync endpoint.
- Consumes the existing `messages` Supabase table fields.

- [ ] **Step 1: Write the failing test**

Assert that the desktop sync response exposes `id` and `created_at` for each returned message.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/timestamped-chat-segments.test.cjs`

Expected: FAIL because the current desktop handler removes those fields.

- [ ] **Step 3: Implement the metadata-preserving message type and route response**

Return `CloudMessage` rows from `cloud::pull`, use their role/content for model history, and serialize all four row fields from `/api/conversation/sync`.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/timestamped-chat-segments.test.cjs`

Expected: PASS.

### Task 2: Render live and synchronized messages in chronological segments

**Files:**
- Modify: `src/ui.html`
- Test: `tests/timestamped-chat-segments.test.cjs`

**Interfaces:**
- Consumes sync rows `{ id, role, content, created_at }`.
- Produces `appendConversationRow(row)` and separate active content/work containers.

- [ ] **Step 1: Extend the failing test**

Exercise a row fixture where an earlier timestamp arrives after a later timestamp, and assert the rendered order is earlier then later. Assert tool-bound content is finalized rather than cleared.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/timestamped-chat-segments.test.cjs`

Expected: FAIL because current UI appends arrival order and calls `moveNarrationToWorkStrip`.

- [ ] **Step 3: Implement sorted row insertion and tool-bounded bubbles**

Insert persisted rows by `(created_at, id)`. Finalize each text segment at `tools_begin`, create an active work bubble after it, and create a new text bubble when content resumes.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/timestamped-chat-segments.test.cjs`

Expected: PASS.

### Task 3: Make the web adapter persist and signal each AI phase

**Files:**
- Modify: `web/cloud-adapter.js`
- Test: `tests/timestamped-chat-segments.test.cjs`

**Interfaces:**
- Emits `tools_begin` before tool events.
- Persists one assistant row for every non-empty model content phase.

- [ ] **Step 1: Extend the failing test**

Assert that phase persistence saves `data.content` inside the loop and that a `tools_begin` event precedes tool execution.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/timestamped-chat-segments.test.cjs`

Expected: FAIL because current code persists `lastContent` once after the loop.

- [ ] **Step 3: Implement phase-by-phase persistence and event emission**

Save each non-empty model content phase after it is received, remove the aggregate save, and signal `tools_begin` before emitting tools.

- [ ] **Step 4: Run tests and build**

Run: `node --test tests/timestamped-chat-segments.test.cjs tests/chat-scroll-layout.test.cjs tests/thinking-rendering.test.cjs tests/tool-work-narration.test.cjs tests/canonical-web-build.test.cjs`

Run: `cargo build --release`

Expected: all tests pass and release build succeeds.
