# CommandBlock Work Strip, Mobile Drawer, and Conversation Sync

## Goal

Make the desktop EXE and hosted web app present the same clear working-state UI:

- a compact, expandable work strip while the agent is working;
- a checklist of current Todos inside that strip;
- a compact per-turn file-change summary;
- the existing Thinking card unchanged;
- a mobile-first right-hand drawer that replaces the crowded bottom status bar; and
- conversation mirroring for the same signed-in account between desktop and mobile.

## Product decisions

### Work Strip

The work strip is a single-line `details` control inserted in the active assistant bubble. Its closed summary includes a working status, the current Todo progress, the current action label, a compact file-change delta, and a chevron. Example:

`กำลังแก้ UI มือถือ · 2/5 · 4 files changed +169 −44`

Expanding it reveals the Todo checklist and the changed-file list. The checklist uses clear pending, active, and completed states. It is only created for a task that has a plan or file changes, and remains tied to that assistant turn so unrelated old work never changes after a new prompt begins.

The existing `details.think` component remains separate, visually and behaviorally unchanged. Thinking is never folded into the work strip.

### File changes

The current standalone file-change card becomes the file-change section of the work strip. Its headline uses the compact visual requested by the user: `N files changed`, green additions, and red deletions. Expanding the strip retains the individual filenames and their deltas.

### Shared Todo protocol

Desktop already accepts the `update_plan` tool. The Cloud agent will receive the same tool contract and system instruction: for a multi-step task it creates a concise numbered plan, updates that plan as steps complete, and returns success without requesting a Desktop Connector operation. The UI parses the existing string plan and stores it as per-turn Todo state.

Tool activity updates the one-line action label. A normal tool call completes the next pending Todo only when the plan has not been replaced by a newer explicit `update_plan` call. This avoids falsely marking all work done just because the agent inspected files.

### Responsive mobile UI

Desktop keeps the normal bottom status bar. On screens 900px wide or narrower, that bar is visually replaced by one fixed hamburger button next to the composer. The hamburger opens an accessible right-hand drawer, without covering the message editor.

The drawer contains the previously visible model selector, account, balance/token usage, top-up action, selected folder, update status, and connection indicator. It supports close button, backdrop click, Escape, and focus return to the hamburger. The existing history and right work panels remain independent mobile overlays.

### Same-account conversation mirroring

The existing Supabase `conversations` and `messages` tables are the canonical cloud transcript. A signed-in web client and the signed-in EXE select the account's active conversation from the server rather than relying solely on a device-local conversation id.

Messages are written with a stable client message id so a sender does not render its own saved message twice. Clients poll the account's active conversation while it is visible and immediately refresh after sending or receiving a response. This intentionally uses a lightweight REST polling path rather than assuming Supabase Realtime is enabled on the free project.

The local EXE will expose authenticated sync endpoints backed by the same user account. It will use the account access token only for the user-scoped Supabase requests, never expose the DeepSeek key, and will degrade safely to its existing local transcript when offline. Messages created from a phone therefore appear on the signed-in desktop within the polling interval; desktop messages appear on the phone the same way. A task in flight remains owned by the originating client, so the other screen renders it as an updating transcript rather than starting a duplicate agent run.

## Scope and safety

- The shared `src/ui.html` remains the canonical visual source, so rebuilding the web bundle makes the EXE and website visually consistent.
- Existing auth, Remote PC, Desktop Connector, Thinking display, token pricing, and update flows remain in place.
- The conversation sync code only reads and writes rows scoped to the signed-in user. It does not synchronize API keys, local folders, terminal output, or Remote PC credentials.
- No new paid service or TURN/Realtime requirement is introduced.

## Verification

Add focused tests for the plan/checklist parsing, compact file-change summary, mobile drawer state, Cloud `update_plan` tool registration, and duplicate-safe conversation synchronization. Verify the web build, JavaScript syntax, Rust tests/build, and a mobile viewport smoke test before release.
