# Sessions and Pinned Messages Design

## Goal

Let each signed-in CommandBlock user create and switch between independent chat sessions, and pin useful messages from either the browser or the desktop EXE.

## Chosen approach

Persist the feature in the existing Supabase `conversations` and `messages` tables. This is the single shared source of truth used by the web application and the Rust desktop application. A browser-only implementation would not appear in the EXE, and a desktop-file-only implementation would not synchronize across devices.

## User interface

- Rename the current conversation-history heading to `SESSION`.
- Add `+ New session` beside the heading. Selecting it creates a new conversation titled `แชทใหม่`, makes it active, clears only the active transcript view, and leaves all prior sessions intact.
- The Session panel lists the signed-in user's conversations with the active item visibly selected. Selecting an item loads its messages.
- Right-clicking a completed user or assistant message opens a small contextual menu with `Pin message` or `Unpin message`.
- Pinned messages retain their chronological place in the transcript, receive a pin badge, and are additionally shown in a compact pinned area at the top of that session. No hidden replacement or text merging occurs.
- The same UI and behavior are delivered by `src/ui.html`, so the desktop EXE and its embedded web UI stay identical. The standalone web build receives the same asset set.

## Data model and synchronization

- Add `is_pinned boolean not null default false` to `public.messages`.
- Add an index supporting session reads by `conversation_id`, chronological message ordering, and pinned-message reads.
- Keep the existing ownership RLS policy: all reads and writes still require `auth.uid() = user_id`.
- `GET /api/conversations` returns session metadata ordered by most-recent update.
- `POST /api/conversations` creates a fresh session; `GET /api/conversation/sync?conversation_id=...` returns only that session's messages.
- `POST /api/messages/:id/pin` toggles the pin state after ownership validation.
- The browser calls equivalent Supabase queries. The EXE API calls its existing cloud layer, keeping data cross-device.
- A message is persisted before it is displayed as a completed message; optimistic pin UI reverts with a visible error if persistence fails.

## Session behavior

- Starting a new session does not delete or rename prior sessions.
- Sending the first message in a new session updates that session's timestamp and title using the existing title convention.
- Chat sends always include only the active session's relevant history. Switching sessions cancels no completed work and never moves messages across sessions.
- Anonymous/offline desktop use retains its existing local-session behavior; multi-session and shared pins require a signed-in cloud account.

## Versioning and update delivery

- Bump Cargo package version from `1.0.0` to `1.0.1` for this update. `crate::VERSION` and the UI version label inherit this automatically.
- Build the updated executable into `C:\Codex\target\release\commandblock.exe` only. Do not overwrite `C:\Codex\Commandblock.exe` in this task.
- Publish the source and release asset through the existing update workflow so the user's installed application can test the download/install path. The update service must continue to advertise an update only when the remote version/build timestamp is newer than the installed version.

## Failure handling

- Session creation, loading, switching, and pinning show a concise Thai error in the current UI and preserve the currently visible session.
- Context menus close on click-away, Escape, or after selecting an action.
- A stale/deleted session falls back to the newest available session or a fresh empty session without losing local draft text.

## Verification

- Add Node tests for the DOM-independent session ordering and pin-state helper behavior, including toggle and stale ordering cases.
- Extend cloud/desktop API contract tests to cover the pinned field, ownership scope, new-session creation, and active-session synchronization.
- Run the complete Node test suite and `cargo test`, then run `cargo build --release` to verify the new versioned executable is produced without replacing the installed EXE.
