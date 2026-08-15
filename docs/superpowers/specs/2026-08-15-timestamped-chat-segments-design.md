# Timestamped Chat Segments Design

## Goal

Render each AI response phase as an individual chat message, while keeping the active Thinking and Work Strip at the bottom of the conversation. Keep browser, mobile, and desktop history in one deterministic timestamp order.

## Root cause

The desktop UI moves text emitted before a tool call into the Work Strip and clears the visible response. The desktop sync route drops message IDs and timestamps, so the UI can only append arrivals. The web adapter collects content from every model/tool phase and persists that collection as one assistant row.

## Design

- A cloud message carries `id`, `role`, `content`, and `created_at` from the database to the UI.
- The UI keeps a timestamp-sorted set of persisted message rows. A late polling response is inserted before a newer row rather than appended.
- The live turn has a temporary assistant segment for content. On `tools_begin`, its text stays visible as a completed assistant message. Tool activity begins in a new active Work Strip below it. The next content creates another assistant segment.
- The web adapter emits `tools_begin` and persists each non-empty assistant model phase separately. The final phase is therefore not merged with earlier progress text.
- Thinking and the active Work Strip remain at the bottom while the turn runs. Finished text segments retain their chronological position above them.

## Error handling and compatibility

- Rows lacking a timestamp use a stable lowest-priority fallback and remain visible.
- Duplicate rows are identified by the database message ID, with the existing role/content fallback retained for older responses.
- Existing conversation rows remain readable without migration because the database already has `id` and `created_at`.

## Verification

- Regression tests cover timestamp ordering, preserving pre-tool text, the active work area placement, and separate cloud persistence phases.
- Run the focused test set and the release build before publishing.
