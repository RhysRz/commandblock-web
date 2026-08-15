# Checkpoint Resume and Preview Plugin Design

## Goal

Let CommandBlock continue an interrupted AI task without repeating completed work, and let the AI inspect and interact with a project's local Preview when that is necessary to verify a web task.

## Scope

This change applies to both the desktop EXE and the web client connected through Desktop Connector. It does not grant control over arbitrary websites, browser profiles, credentials, payments, or external desktop applications.

## Current state

The web adapter already saves a bounded run state per authenticated account and emits a resume event after a 12-step interruption. The web UI renders a continuation button for that event. The EXE needs the same explicit recovery affordance and the recovery data needs a stable, inspectable shape shared by both clients.

The existing `open_preview` tool starts/opens a local project preview but does not expose a controlled interaction surface to the model.

## Decision

Use two built-in capability modules rather than third-party executable plugins:

1. **Checkpoint Resume** stores the active conversation id, bounded message/tool evidence, Todo state, root folder, and interruption reason. It is scoped by account and project. A run is cleared only after a normal completion or when the user explicitly starts a new session. Both UI clients show a single `ทำต่อ` action only while a valid checkpoint exists.
2. **Preview Browser** is a built-in capability registry entry alongside the existing tools. It can open the current project's local preview, inspect a constrained page snapshot, and perform click/text actions only against the local preview origin. It emits every action to the existing Work Strip. The web client forwards actions through Desktop Connector; the EXE performs them locally.

## User experience

When a tool run reaches its step budget or a recoverable API error occurs, the UI shows: `งานยังไม่เสร็จ — checkpoint ถูกบันทึกแล้ว` and a `▶ ทำต่อ` button. Clicking it restores the checkpoint and sends the canonical continuation instruction. The original user request is not duplicated in the visible history.

When the model needs to validate an interactive UI, it calls `open_preview` and then Preview Browser actions. The UI shows concise activity such as `Preview: เปิด`, `Preview: คลิก “Login”`, and `Preview: กรอก email` in the Work Strip. The preview tab opens automatically when an interaction is requested.

## Safety and failure handling

- Preview interaction accepts only localhost or the locally hosted project preview URL created by CommandBlock.
- Interactions require an active Desktop Connector when called from the hosted web app; otherwise they return a clear actionable error without claiming success.
- No secrets are included in checkpoint payloads or displayed in the activity log.
- A stale or malformed checkpoint is discarded with an explanatory message and no continuation request is sent.
- Preview timeouts, unavailable page elements, and Connector disconnection are reported as tool results so the AI can adapt.

## Data and interfaces

`RunState` remains browser/local-cache based for detailed tool evidence; it contains `conversationId`, `messages`, `rootPath`, `plan`, `savedAt`, and `reason`. It is account/project namespaced and timestamped.

The capability registry exposes `preview_open`, `preview_inspect`, `preview_click`, and `preview_fill` definitions. Connector commands use an allowlisted `preview_*` action and return structured JSON only.

## Verification

- Unit tests prove checkpoints are saved, restored only for the matching account/project, and cleared after completion.
- Contract tests prove the EXE and web UI both expose the same continuation state and button labels.
- Tool tests reject external URLs and unavailable Connector actions.
- Existing Node and Cargo test suites remain green.

