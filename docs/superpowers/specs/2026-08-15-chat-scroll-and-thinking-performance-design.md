# Chat scroll and Thinking performance design

## Goal

Keep the CommandBlock chat usable while an agent streams a long response: reading older messages must never be interrupted by automatic scrolling, and a long Thinking stream must not cause frequent expensive DOM updates.

## Smart chat scrolling

- Treat the reader as following live output only when the chat is within 96 pixels of the bottom.
- `scrollBottom()` scrolls only while that follow-live state is true, unless the caller explicitly asks to reveal an item the user just created.
- A user scroll away from the bottom immediately disables follow-live. New streamed content then leaves the viewport in place.
- Returning to the bottom re-enables follow-live. This applies to desktop and mobile because both use the same chat container.

## Thinking presentation

- Thinking starts collapsed and its summary shows the live state plus a character count.
- Incoming thinking chunks are collected immediately, but the DOM is rendered no more than once every 200 ms.
- The expanded view renders only the most recent 3,000 characters, with a notice when earlier content has been omitted from the visual display.
- The existing Thinking card remains separate from tool and Todo cards. It must not force the chat to scroll when the reader has moved away from the bottom.

## Failure handling

- A pending Thinking render timer is cleared when the chat turn finishes or errors, then the final buffered Thinking state is rendered once.
- Empty or malformed stream events leave the prior UI state unchanged.

## Tests

- UI contract tests assert that auto-scroll is conditional on the reader being near the bottom and restores after returning to the bottom.
- UI contract tests assert that Thinking rendering is throttled, capped to the visible tail, and starts closed.
