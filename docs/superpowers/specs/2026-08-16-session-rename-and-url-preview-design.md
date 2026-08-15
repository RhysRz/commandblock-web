# SESSION rename and HTTPS Preview URL design

## Goal

Let a signed-in CommandBlock user rename a SESSION from its existing Obsidian context menu, and let `/preview https://…` display a published HTTPS site inside the in-app Preview tab.

## Session rename

- The SESSION context menu gains `✏ Rename SESSION`, alongside the existing pin and delete actions.
- Selecting it opens a CommandBlock-styled modal, prefilled with the current title. It is not a browser `prompt()`.
- The title is trimmed and must contain 1–80 characters. Enter saves; Escape, Cancel, or clicking Cancel changes nothing.
- Save calls an owner-scoped `POST /api/conversations/:id/rename` endpoint with `{ title }`.
- The desktop handler delegates to a cloud function which patches `conversations.title` and `updated_at`; the browser adapter implements the identical virtual API with Supabase.
- The client refreshes its session list after a successful save, so the renamed row (and its current sort position) is shown immediately. Other web/desktop clients receive the title when their normal session refresh runs.

## HTTPS Preview URL

- `/preview` without arguments retains its current behavior: reopen the latest local preview.
- The desktop command parser recognizes `/preview <url>` only when `<url>` is a valid `https://` URL. It stores that URL as the active preview and returns a normal Preview result.
- The shared UI treats both forms as a Preview request, switches to the Preview tab, and refreshes `/api/state` after the response. The iframe then loads the current `preview_url`.
- HTTP, file, localhost, malformed URLs, and arbitrary command text are rejected with a concise Thai error. Local preview tools continue to use the existing local allow-list.
- The explicit `เปิดในเบราว์เซอร์` button remains an escape hatch and is unchanged.

## Error handling and tests

- Rename errors leave the current title unchanged and show the returned Thai error toast.
- URL validation errors do not alter the previously active preview.
- Contract tests cover the context action, modal and rename API in both desktop/web adapters, plus the exact `/preview https://` parsing and rejection of non-HTTPS input.
- Existing session pin/delete and local Preview behavior remain covered by the full Node and Rust test suites.
