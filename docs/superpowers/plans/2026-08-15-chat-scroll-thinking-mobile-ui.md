# Chat Scroll, Thinking, and Mobile UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep streaming chat readable and responsive while consolidating CommandBlock's mobile controls into drawers and floating actions in both the web and desktop EXE UI.

**Architecture:** `src/ui.html` remains the canonical shared UI embedded into the EXE and used to build the web shell. Browser-only behavior lives in its existing script, while `src/gui.rs` emits a small explicit SSE boundary event so the UI can distinguish tool-work narration from a final model answer.

**Tech Stack:** Vanilla HTML/CSS/JavaScript, Rust SSE server, Node.js `node:test`, Cargo.

## Global Constraints

- Apply all UI behavior in `src/ui.html` so CommandBlock Web and `Commandblock.exe` match.
- Follow live output only when the reader is within exactly 96 pixels of the bottom.
- Throttle Thinking DOM updates to 200 ms and render only the latest 3,000 characters.
- Keep Thinking, Todos, and work narration expandable and separate from the final answer.
- On widths at or below 900 pixels, use one right slidebar, a floating history action at upper left, and stacked menu/settings actions at upper right.

---

### Task 1: Protect reader-controlled scrolling

**Files:**
- Modify: `src/ui.html:1484-1490, 1700-1764`
- Modify: `tests/chat-scroll-layout.test.cjs`

**Interfaces:**
- Produces: `isNearChatBottom(): boolean`, `scrollBottom(options?: { force?: boolean }): void`, and `followLiveOutput: boolean`.
- Consumes: Existing `chat`, `wrap`, `addMsg`, and streaming event handlers.

- [ ] **Step 1: Write the failing test**

```js
test('streaming output follows only a reader already near the bottom', () => {
  assert.match(html, /const AUTO_SCROLL_BOTTOM_GAP\s*=\s*96/);
  assert.match(html, /function isNearChatBottom\(\)/);
  assert.match(html, /chat\.addEventListener\("scroll",\s*\(\)\s*=>\s*\{\s*followLiveOutput\s*=\s*isNearChatBottom\(\)/);
  assert.match(html, /function scrollBottom\(\{force=false\}=\{\}\)/);
  assert.match(html, /if\(!force && !followLiveOutput\) return;/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/chat-scroll-layout.test.cjs`

Expected: FAIL because the 96-pixel follow-live functions do not exist.

- [ ] **Step 3: Write minimal implementation**

```js
const AUTO_SCROLL_BOTTOM_GAP = 96;
let followLiveOutput = true;
function isNearChatBottom(){
  return chat.scrollHeight - chat.scrollTop - chat.clientHeight <= AUTO_SCROLL_BOTTOM_GAP;
}
function scrollBottom({force=false}={}){
  if(!force && !followLiveOutput) return;
  chat.scrollTop = chat.scrollHeight;
}
chat.addEventListener("scroll",()=>{ followLiveOutput = isNearChatBottom(); });
```

Call `scrollBottom({force:true})` only for the user message just added by `send`; leave streamed content, tool state, Thinking, and notes conditional.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/chat-scroll-layout.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add tests/chat-scroll-layout.test.cjs src/ui.html
git commit -m "fix: preserve reader scroll position during streams"
```

### Task 2: Throttle and cap Thinking rendering

**Files:**
- Modify: `src/ui.html:1719-1732, CSS for .think`
- Create: `tests/thinking-rendering.test.cjs`

**Interfaces:**
- Produces: `scheduleThinkingRender(bubble, thinkingText): void` and `flushThinkingRender(): void`.
- Consumes: SSE `think` events and `scrollBottom()` from Task 1.

- [ ] **Step 1: Write the failing test**

```js
test('Thinking uses a closed, throttled, tail-only renderer', () => {
  assert.match(html, /const THINK_RENDER_DELAY_MS\s*=\s*200/);
  assert.match(html, /const THINK_VISIBLE_CHAR_LIMIT\s*=\s*3000/);
  assert.match(html, /function scheduleThinkingRender\(bub, fullText\)/);
  assert.match(html, /setTimeout\(flushThinkingRender, THINK_RENDER_DELAY_MS\)/);
  assert.match(html, /details"\); th\.className="think"; th\.open=false/);
  assert.match(html, /fullText\.slice\(-THINK_VISIBLE_CHAR_LIMIT\)/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/thinking-rendering.test.cjs`

Expected: FAIL because the throttled renderer and visible-tail limit do not exist.

- [ ] **Step 3: Write minimal implementation**

Create a per-turn pending Thinking state containing `bubble`, full text, and one timer. On each SSE `think` chunk append to the state, update only the count in the summary, and schedule one render after 200 ms. The renderer writes `fullText.slice(-3000)` to `.thinkb`, prefixes an omission notice when required, and calls conditional `scrollBottom()`.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/thinking-rendering.test.cjs tests/chat-scroll-layout.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add tests/thinking-rendering.test.cjs src/ui.html
git commit -m "perf: throttle long thinking output"
```

### Task 3: Keep tool-work narration out of final answers

**Files:**
- Modify: `src/gui.rs:1568-1609`
- Modify: `src/ui.html:1548-1602, 1718-1753`
- Create: `tests/tool-work-narration.test.cjs`

**Interfaces:**
- Produces: an SSE event named `tools_begin` with `{}` payload; `moveNarrationToWorkStrip(bubble, text): void`.
- Consumes: `TurnSink::tools_begin`, existing `renderWorkStrip`, and streamed content accumulated before a tool round.

- [ ] **Step 1: Write the failing test**

```js
test('tool narration is moved into the expandable work strip', () => {
  assert.match(gui, /fn tools_begin\(&mut self\)\s*\{[\s\S]*sse\(self\.out, "tools_begin", json!\(\{\}\)\)/);
  assert.match(html, /function moveNarrationToWorkStrip\(bub, narration\)/);
  assert.match(html, /ev==="tools_begin"/);
  assert.match(html, /className="work-narration"/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/tool-work-narration.test.cjs`

Expected: FAIL because `tools_begin` is currently discarded and no work-narration UI exists.

- [ ] **Step 3: Write minimal implementation**

Make `SseSink::tools_begin` send `tools_begin`. In the UI, keep streamed pre-tool content in the active assistant bubble until this event arrives, then move it into a collapsed `.work-narration` detail within the Work Strip and clear the primary answer accumulator. Final no-tool content remains in `.txt` and continues to sync as the assistant response.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/tool-work-narration.test.cjs tests/work-strip-ui.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add tests/tool-work-narration.test.cjs src/gui.rs src/ui.html
git commit -m "fix: keep tool narration out of final answers"
```

### Task 4: Consolidate mobile controls into drawers and floating actions

**Files:**
- Modify: `src/ui.html:884-944, 945-1081, 1820-1865, 2091-2200`
- Create: `tests/mobile-navigation-layout.test.cjs`

**Interfaces:**
- Produces: `#mobileHistoryToggle`, `#mobileMenuToggle`, `#mobileSettingsToggle`, `setMobileMenuDrawer(open)`, and `setMobileHistoryDrawer(open)`.
- Consumes: existing `#histpane`, `#rightpane`, `#mobileStatusDrawer`, `openSettings()`, and mobile backdrop/Escape behavior.

- [ ] **Step 1: Write the failing test**

```js
test('mobile uses floating history, menu, and settings controls without a bottom rail', () => {
  assert.match(html, /id="mobileHistoryToggle"/);
  assert.match(html, /id="mobileMenuToggle"/);
  assert.match(html, /id="mobileSettingsToggle"/);
  assert.match(html, /function setMobileMenuDrawer\(open\)/);
  assert.match(html, /mobileSettingsToggle\.addEventListener\("click", openSettings\)/);
  assert.match(html, /#iconrail\s*\{[\s\S]*display:\s*none/);
  assert.match(html, /body\.mobmenu #rightpane\s*\{\s*transform:\s*translateX\(0\)/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/mobile-navigation-layout.test.cjs`

Expected: FAIL because the current mobile layout has a bottom icon rail and no floating controls.

- [ ] **Step 3: Write minimal implementation**

Add the three accessible floating buttons outside `#iconrail`. On mobile, hide `#iconrail`, make `#rightpane` the only right slidebar containing tabs plus status, and route the hamburger to `body.mobmenu`. Route the history button to `body.mobhist`; close either drawer from the shared backdrop and Escape. Wire the settings button directly to the existing `openSettings()`. Leave the existing desktop rail and desktop layout unchanged.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/mobile-navigation-layout.test.cjs tests/work-strip-ui.test.cjs tests/chat-scroll-layout.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add tests/mobile-navigation-layout.test.cjs src/ui.html
git commit -m "feat: simplify mobile command controls"
```

### Task 5: Build and verify the shared artifact

**Files:**
- Modify: generated `C:\Codex\Commandblock.exe` only after release build succeeds

**Interfaces:**
- Consumes: canonical `src/ui.html`, `src/gui.rs`, Node UI contract tests, and Cargo release build.

- [ ] **Step 1: Run the full relevant verification suite**

Run:

```powershell
node --test tests/chat-scroll-layout.test.cjs tests/thinking-rendering.test.cjs tests/tool-work-narration.test.cjs tests/mobile-navigation-layout.test.cjs tests/work-strip-ui.test.cjs tests/canonical-web-build.test.cjs
node scripts/build-web.mjs
$env:CARGO_TARGET_DIR='C:\Codex\target'; cargo check
$env:CARGO_TARGET_DIR='C:\Codex\target'; cargo build --release
```

Expected: all Node tests pass, web build completes, Cargo check and release build succeed.

- [ ] **Step 2: Replace the desktop EXE after it is closed**

```powershell
Copy-Item -LiteralPath 'C:\Codex\target\release\commandblock.exe' -Destination 'C:\Codex\Commandblock.exe' -Force
```

If the desktop app is open, invoke `commandblock-updater.exe --apply` with the running app PID so replacement happens after that process exits.

- [ ] **Step 3: Commit and push source changes**

```powershell
git add src/ui.html src/gui.rs tests
git commit -m "feat: improve streaming chat and mobile navigation"
git push origin main
```
