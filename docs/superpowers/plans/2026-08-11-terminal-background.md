# Buff Terminal Background Implementation Plan

> **For agentic workers:** Use `executing-plans` to carry out this single CSS task and verify the embedded desktop artifact.

**Goal:** Give every Buff panel the Terminal background color while retaining Minecraft controls and readable colored chat bubbles.

**Architecture:** Change only the final Minecraft theme override in `src/ui.html`; it already wins over the preceding base styles. Replace panel and chat decorative backgrounds with `var(--code-bg)` and leave the day/night JavaScript untouched.

**Tech Stack:** Embedded HTML/CSS/JavaScript, Node built-in tests, Cargo.

## Global Constraints

- Every application background surface uses `var(--code-bg)`.
- Retain colored buttons, tabs, chat bubbles, pixel borders and day/night accents.
- Do not change HTML, JavaScript, Rust code, endpoints or theme timing.
- No Git repository is present; do not commit.

---

### Task 1: Replace visual panel backgrounds and verify the artifact

**Files:**
- Modify: `src/ui.html:583-662`
- Test: `tests/ui-theme.test.cjs`

**Interfaces:**
- Consumes: existing `--code-bg` variables and `body[data-theme]` values.
- Produces: a uniform Terminal-colored canvas for chat, history, tool tabs and settings modal.

- [ ] **Step 1: Preserve the existing timed-theme regression test**

Run: `node --test tests/ui-theme.test.cjs`

Expected: PASS. This establishes that the unrelated day/night behavior is green before a CSS-only change.

- [ ] **Step 2: Apply the minimal CSS override**

At the end of the Minecraft Overworld theme block, add this rule set:

```css
#chatpane, #rightpane, #histpane, #chat, .pane, .tabcontent,
.modal-card, .modal-head, .modal-foot {
  background: var(--code-bg);
}
#chat { background-image: none; }
#chat:before { display: none; }
```

Do not alter `.bubble`, `.msg.user .bubble`, `.tab`, `.pill`, button or border rules.

- [ ] **Step 3: Re-run automated checks**

Run: `node --test tests/ui-theme.test.cjs && cargo build --release`

Expected: both theme tests PASS and `target\\release\\buff.exe` is generated successfully.

- [ ] **Step 4: Deliver the release executable**

Run: `Copy-Item -LiteralPath 'C:\\Codex\\target\\release\\buff.exe' -Destination 'C:\\Codex\\buff.exe' -Force`

Expected: root `buff.exe` is replaced after confirming no `buff` process is running. Compare SHA-256 hashes of both files and require equality.

## Plan Self-Review

- The CSS rule covers each scoped background surface and removes the chat-only Overworld scenery.
- Existing day/night tests protect the logic that must remain unchanged.
- No deferred actions or unspecified code remain.
