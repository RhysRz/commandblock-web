# In-app confirmation dialog

## Goal

Replace browser/WebView confirmation dialogs with one CommandBlock-styled confirmation modal so destructive actions never show a `127.0.0.1 says` prompt.

## Behavior

- Provide a reusable `confirmAction(options)` Promise API in `src/ui.html`.
- Use an Obsidian-purple surface and make **Cancel** the default focused action.
- Escape, clicking the backdrop, and Cancel resolve to `false`; Confirm resolves to `true`.
- Replace native confirmations only for deleting a SESSION, restoring a backup, and logging out.

## Safety

- No backend routes, stored data, or credentials change.
- The destructive request starts only after the popup resolves `true`.
- Each caller provides a clear title, consequence text, and confirm label.

## Verification

- Add a contract test proving the reusable dialog controls exist, the three actions call `confirmAction`, and no browser `confirm()` calls remain.
- Run the focused Node test, all Node tests, Rust tests, release build, and whitespace validation.
