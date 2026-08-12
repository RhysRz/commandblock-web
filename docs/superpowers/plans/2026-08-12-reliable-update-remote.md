# Reliable Update and Remote Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recover update downloads from transient release CDN failures and provide actionable Remote PC network diagnostics.

**Architecture:** `src/update.rs` owns resumable release download state and preserves SHA-256 verification. `web/cloud-adapter.js` presents the existing P2P state and explains when a network requires TURN relay support.

**Tech Stack:** Rust, ureq, SHA-256, Node built-in test runner, browser JavaScript.

## Global Constraints

- Keep the GitHub release asset as the canonical source.
- Never bypass checksum verification.
- Do not bundle a paid TURN service or relay credentials.
- Keep Thai error text actionable.

---

### Task 1: Retry and resume update downloads

**Files:** Modify `src/update.rs` and `tests/updater-self-replace.test.cjs`.

- [ ] Write a failing static regression test that requires `for attempt in 0..3`, a `Range` request header, and `retry_delay` in `src/update.rs`.
- [ ] Run `node --test tests/updater-self-replace.test.cjs` and confirm it fails because the feature is absent.
- [ ] Implement the smallest retry loop around a partial-byte reader: retain bytes after a read failure, send `Range: bytes=<retained>-` on the next attempt, wait 1 then 2 seconds, and surface the final error only after attempt 3.
- [ ] Run `node --test tests/updater-self-replace.test.cjs` and `cargo test update::tests`.

### Task 2: Clarify Remote PC P2P failures

**Files:** Modify `web/cloud-adapter.js` and `tests/remote-device-approval.test.cjs`.

- [ ] Write a failing regression test requiring `TURN relay` and `เครือข่ายนี้อาจบล็อก P2P` in the web adapter.
- [ ] Run `node --test tests/remote-device-approval.test.cjs` and confirm it fails.
- [ ] Replace the generic ICE failure text with Thai guidance to retry another network and explain that a TURN relay is needed when the network blocks direct P2P.
- [ ] Run `node --test tests/remote-device-approval.test.cjs`.

### Task 3: Verify and publish

**Files:** Build output only.

- [ ] Run focused Node tests, `cargo test`, and `node scripts/build-web.mjs`.
- [ ] Commit the source, tests, specification, and plan, then push `main` to trigger the web and Windows release workflow.
