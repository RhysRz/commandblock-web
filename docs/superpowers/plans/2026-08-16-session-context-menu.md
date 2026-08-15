# Session Context Menu Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe right-click pinning and SESSION deletion with an Obsidian popup in desktop and web clients.

**Architecture:** Keep one UI context-menu element whose actions change with the clicked target. Add an authenticated owner-scoped delete route to the desktop server and mirror the route in the web adapter.

**Tech Stack:** Rust, ureq, Supabase PostgREST, HTML/CSS/JavaScript, Node contract tests.

## Global Constraints

- Never show the native context menu for CommandBlock messages or SESSION rows.
- Delete only a SESSION owned by the signed-in user.
- Keep pinning unavailable until a message has a Cloud row id.
- Preserve the Obsidian violet visual system.

---

### Task 1: Add the authenticated SESSION delete API

**Files:** `src/cloud.rs`, `src/gui.rs`, `web/cloud-adapter.js`, `tests/session-desktop-contract.test.cjs`

- [ ] Write a failing contract test for `delete_conversation` and `POST /api/conversations/{id}/delete`.
- [ ] Implement UUID validation, user-id-filtered delete, and clear the active desktop transcript when it is deleted.
- [ ] Run `node --test tests/session-desktop-contract.test.cjs tests/session-web-contract.test.cjs`.

### Task 2: Add target-aware Obsidian context popup

**Files:** `src/ui.html`, `tests/session-web-contract.test.cjs`

- [ ] Write a failing test for `deleteSession`, `openContextMenu`, the menu button, and `#120b20` surface.
- [ ] Bind right-click handlers to SESSION rows and all message rows, clamp the menu location, and close it on outside click or Escape.
- [ ] Run `node --test tests/session-web-contract.test.cjs`.

### Task 3: Package and verify

**Files:** `Cargo.toml`, `Cargo.lock`

- [ ] Bump the package version to `1.0.4`.
- [ ] Run `node --test tests/*.test.cjs`, `cargo test`, and `cargo build --release`.
- [ ] Commit only the feature files and tests.
