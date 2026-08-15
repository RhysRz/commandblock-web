# Checkpoint Resume and Preview Plugin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Resume interrupted CommandBlock jobs without duplicated work, and make the local project Preview a built-in, auditable AI capability.

**Architecture:** Extend the existing per-account recovery cache with project identity, timestamp, plan, and interruption reason. The shared UI renders a continuation action from EXE or cloud events. Preview capability definitions build on `open_preview`; Web forwards allowed Preview actions through Desktop Connector and EXE executes them locally.

**Tech Stack:** Rust, vanilla JavaScript, WebView2 bridge, Node test runner, Desktop Connector.

## Global Constraints

- Checkpoints are account-and-project scoped and contain no API key, password, or secret.
- Preview actions may target only CommandBlock local `127.0.0.1` previews.
- Preserve `src/config.rs`, `src/diagnostics.rs`, and `buff_session.json.bak` because they are user changes.

---

### Task 1: Project-scoped recovery contract

**Files:** Modify `web/chat-recovery.js`; modify `tests/chat-recovery.test.cjs`.

**Interfaces:** `saveRunState(storage, userId, state)` persists `{conversationId, messages, projectKey, plan, savedAt, reason}`. `loadRunState(storage, userId, projectKey)` returns matching state only.

- [x] Write a failing test:

```js
recovery.saveRunState(storage, 'user-1', { conversationId:'conv-1', messages:[], projectKey:'C:/demo', plan:'- [ ] build', savedAt:100, reason:'step_limit' });
assert.equal(recovery.loadRunState(storage, 'user-1', 'C:/other'), null);
assert.equal(recovery.loadRunState(storage, 'user-1', 'C:/demo').reason, 'step_limit');
```

- [x] Run `node --test tests/chat-recovery.test.cjs` and confirm it fails.
- [x] Add the fields, validation, and timestamped load result to `chat-recovery.js`.
- [x] Run `node --test tests/chat-recovery.test.cjs` and confirm it passes.
- [ ] Commit with `feat: scope interrupted chat checkpoints`.

### Task 2: Cloud checkpoint and reusable resume control

**Files:** Modify `web/cloud-adapter.js`, `src/ui.html`, `tests/cloud-chat-context.test.cjs`.

**Interfaces:** Cloud emits `{t, reason, project_key, plan}` as `resume`; `resumeFromCheckpoint()` calls the canonical continuation phrase exactly once.

- [x] Write a failing contract test for `projectKey`, `reason: 'step_limit'`, `resumeFromCheckpoint`, and `ทำต่อจาก Checkpoint`.
- [x] Run `node --test tests/cloud-chat-context.test.cjs` and confirm it fails.
- [x] Use `connector:<device-id>` as the web project key, persist step-limit/recoverable-error reasons, and replace the inline button callback with `resumeFromCheckpoint()`.
- [x] Run `node --test tests/cloud-chat-context.test.cjs tests/chat-recovery.test.cjs` and confirm it passes.
- [ ] Commit with `feat: show resumable cloud checkpoints`.

### Task 3: EXE recovery event

**Files:** Modify `src/gui.rs`, `src/ui.html`; create `tests/exe-resume-contract.test.cjs`.

**Interfaces:** The desktop tool loop saves its bounded history and emits `resume` with `reason`, `project_key`, `plan`, and a Thai label when it reaches the agent-round limit.

- [x] Write failing source-contract tests for `"resume"`, round-limit handling, and UI resume event consumption.
- [x] Run `node --test tests/exe-resume-contract.test.cjs` and confirm it fails.
- [x] Save the active root/plan/history on interruption, emit the resume event, and clear state only after normal completion.
- [x] Run `node --test tests/exe-resume-contract.test.cjs; cargo test` and confirm both pass.
- [ ] Commit with `feat: resume interrupted desktop runs`.

### Task 4: Local Preview Browser capability

**Files:** Modify `src/tools.rs`, `src/main.rs`, `src/ui.html`, `web/cloud-adapter.js`; create `tests/preview-plugin-contract.test.cjs`.

**Interfaces:** Tools `preview_open`, `preview_inspect`, `preview_click`, `preview_fill` validate `last_preview_url()` starts with `http://127.0.0.1:`. Web uses allowlisted Connector action `preview_action` and returns `{ok, action, detail}`.

- [x] Write failing tests asserting tool names, localhost validation, Connector forwarding, and `Preview:` Work Strip activity.
- [x] Run `node --test tests/preview-plugin-contract.test.cjs` and confirm it fails.
- [x] Add tool definitions and dispatcher branches; have each action open the Preview tab and emit structured audit/result text. Return a clear unavailable-Connector error in web mode.
- [x] Run `node --test tests/preview-plugin-contract.test.cjs; cargo test` and confirm both pass.
- [ ] Commit with `feat: add local preview browser capability`.

### Task 5: Regression and release build

**Files:** Modify `Cargo.toml`, `Cargo.lock`; test `tests/*.test.cjs`.

- [x] Add a failing contract expectation for `version = "1.0.2"`.
- [x] Run `node --test tests/*.test.cjs` before changing release metadata.
- [x] Set Cargo package version to `1.0.2`.
- [x] Run `node --test tests/*.test.cjs; cargo test; cargo build --release`.
- [ ] Confirm `target/release/commandblock.exe` exists and commit with `release: prepare commandblock v1.0.2`.
