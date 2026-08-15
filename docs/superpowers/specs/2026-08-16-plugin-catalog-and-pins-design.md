# Plugin catalog, pinned messages, and session header

## Goal

Add a CommandBlock plugin catalog that looks and behaves like an Obsidian-style
marketplace, keep pinned chat messages visible while reading a long transcript,
and align the New session control at the right side of the SESSION header.

## Scope

### Plugin catalog

- Add a Plugins icon to the existing left navigation.
- Open a first-party CommandBlock dialog/page with an Obsidian dark-purple
  interface: search, Installed, Public, category sections, and connector cards.
- Populate the catalog with broadly useful integrations including development,
  productivity, storage, design, communication, hosting, and billing tools.
- Every card has an explicit capability state:
  - **Built in**: CommandBlock already has the related capability.
  - **Connect required**: a real external integration needs that provider's
    account/OAuth/API configuration before it can be used.
  - **Planned**: the catalog entry is informative only and cannot be installed.
- The UI must never claim an external ChatGPT plugin is installed, connected, or
  usable unless CommandBlock has a working integration and user authorization.

### Pinned messages

- Keep the existing owner-scoped durable message pin state.
- Render pinned messages in a sticky, compact tray immediately below the chat
  header while a conversation contains pins.
- Hide the tray when no pinned messages exist.
- A tray entry scrolls the matching message into view and gives it a temporary
  visual focus treatment.  The tray itself does not duplicate, mutate, or
  reorder messages in the transcript.
- This behavior is shared by the browser build and the Windows EXE because both
  use `src/ui.html`.

### SESSION header

- Keep the title `SESSION` at the left of its existing header.
- Place `+ New session` at the far right of the same header row.
- Preserve the mobile close button and retain usable touch targets.

## Data flow and safety

The catalog is local UI metadata only.  It sends no credentials, creates no
external accounts, and does not change provider permissions.  Existing
message-pin API endpoints and Supabase RLS remain the source of truth for the
pinned tray.  The feature changes only presentation and navigation of messages
already visible to the signed-in owner.

## Testing and release

- Add contract tests for catalog entry/status rendering, the left navigation
  trigger, sticky pin tray, tray-to-message navigation, and session-header
  alignment.
- Run the complete Node and Rust test suites plus a Windows release build.
- Bump the desktop version, commit only feature files, push `main`, and verify
  the uploaded release ZIP before reporting completion.

## Out of scope

- Automatically installing every ChatGPT connector.
- OAuth/API implementation for third-party providers.
- Changing the user's existing provider keys, account settings, or permissions.
