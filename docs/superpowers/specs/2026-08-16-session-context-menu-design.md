# Session Context Menu Design

## Goal

Provide an Obsidian popup at the pointer position: pin or unpin a synchronized message, and delete a selected SESSION safely.

## Interaction

- Right-click a message to open **Pin message** or **Unpin message**.
- Right-click a SESSION to open **Delete SESSION**.
- A message without a Cloud id shows a disabled syncing action instead of the native browser menu.
- Clicking outside or Escape closes the popup; deletion requires confirmation.

## Safety and UI

- Deletion is authenticated and owner-scoped; existing database cascades remove the messages in that SESSION.
- Desktop HTTP server and web adapter share `POST /api/conversations/{id}/delete`.
- The popup is clamped inside the viewport and uses `#120b20` Obsidian purple with a contained destructive action.
