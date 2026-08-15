# In-App Preview and Resizable Workspace Design

## Goal

Make CommandBlock show locally generated web previews in its existing in-app Preview tab by default, and let desktop users resize the right-hand workspace panel that contains Terminal, Preview, Files, Changes, Queue, and Notes.

## Scope

- `open_preview`, `/preview`, and Preview tool actions keep recording a local `127.0.0.1` URL but must not invoke the operating system browser.
- The UI observes the existing preview URL, activates the Preview tab when a preview action is recorded, and loads the URL in `#previewFrame`.
- The existing explicit **เปิดในเบราว์เซอร์** button remains the only browser-launch action.
- Desktop receives an accessible vertical drag handle immediately left of `#rightpane`.
- Dragging updates the third desktop grid column using a CSS custom property. The value is clamped to 240–720 pixels and stored in `localStorage` under `commandblock.rightPaneWidth`.
- On viewports at or below the existing mobile breakpoint, the handle is hidden and the existing slide-over right panel behavior remains unchanged.

## Data Flow

1. Rust Preview tools create or serve a local preview and save `PREVIEW_URL` without calling `open_browser`.
2. The GUI state endpoint already exposes `preview_url`.
3. The UI receives the state, selects the Preview tab for a preview activity, and `refreshPreview()` sets the iframe source.
4. A deliberate click on the browser button opens the URL in a new browser tab.
5. Pointer and keyboard events on the resize handle update the custom property and persist the approved width locally.

## Error Handling and Accessibility

- A missing or invalid preview URL preserves the existing Preview empty hint.
- The drag handle has an ARIA label, keyboard controls, focus styling, and uses ArrowLeft/ArrowRight with Home/End reset bounds.
- Width clamping prevents the chat, session panel, or right pane from becoming unusable.

## Testing

- A contract test must fail against the current browser-launch implementation, then assert Preview tools no longer call `open_browser` and the explicit browser button remains.
- A UI contract test must assert the resize handle, localStorage key, desktop grid variable, clamp bounds, and mobile hiding rule.
- Run Node contracts, Rust tests, and a release build before publishing.
