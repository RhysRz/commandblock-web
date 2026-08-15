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

## Tool-work messages

- Content emitted before a tool-call round is classified as work narration, not the final answer.
- The UI moves that narration into the existing expandable work area so messages such as command-parsing remarks do not appear as separate chat answers.
- Content from the final no-tool response remains in the primary assistant answer.

## Mobile navigation

- On screens at or below 900 pixels, remove the fixed bottom icon rail.
- A single right-hand slidebar contains the tool tabs (Queue, Files, Changes, Preview, Terminal, Notes) and account/status controls.
- The right slidebar opens from a circular hamburger button at the upper right of the chat. The circular settings button sits directly below it and opens the existing settings modal.
- A circular chat-history button at the upper left opens the history drawer from the left.
- The floating buttons stay above the chat header, use the existing purple-glass visual language, and have accessible labels.

## Failure handling

- A pending Thinking render timer is cleared when the chat turn finishes or errors, then the final buffered Thinking state is rendered once.
- Empty or malformed stream events leave the prior UI state unchanged.

## Tests

- UI contract tests assert that auto-scroll is conditional on the reader being near the bottom and restores after returning to the bottom.
- UI contract tests assert that Thinking rendering is throttled, capped to the visible tail, and starts closed.
- UI contract tests assert that mobile navigation has no bottom rail and exposes the right slidebar, settings button, and left history button.
- UI contract tests assert that tool-work narration is rendered in the expandable work area rather than as final-answer text.
