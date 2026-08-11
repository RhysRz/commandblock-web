# Buff Minecraft Overworld UI Implementation Plan

> **For agentic workers:** Execute this plan task-by-task with a red-green-refactor test cycle. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restyle every Buff UI pane as a CSS-only Minecraft Overworld interface that automatically uses day or night colors based on local time, without changing existing UI behavior.

**Architecture:** Keep `src/ui.html` as the only production file changed. Add CSS custom-property theme palettes and restyle the existing class names, then add a small DOM-free theme helper in the existing inline script. A Node built-in test extracts only that helper from `ui.html` and verifies the time boundary and DOM updates without new dependencies.

**Tech Stack:** Rust/Wry (existing desktop shell), one embedded HTML/CSS/JavaScript file, Node.js built-in `node:test` and `node:vm`, Cargo.

## Global Constraints

- Preserve every existing element `id`, `data-tab`, inline handler, endpoint, and fetch request.
- Preserve the current three-column layout and current small-screen collapse rules.
- Do not modify `src/gui.rs`, other Rust files, `config.json`, API settings, or backend behavior.
- Do not add images, web fonts, npm packages, or external assets.
- Use local system time: `day` from 06:00 through 17:59; `night` from 18:00 through 05:59.
- On an invalid hour, use `day`.
- The workspace has no `.git` directory, so no commit step can be performed.

---

### Task 1: Add a regression test for theme selection and DOM application

**Files:**
- Create: `tests/ui-theme.test.cjs`
- Modify: none

**Interfaces:**
- Consumes: the source bounded by `/* THEME_LOGIC_START */` and `/* THEME_LOGIC_END */` in `src/ui.html`.
- Produces: a repeatable `node --test tests/ui-theme.test.cjs` check for `themeForHour(hour)` and `applyThemeForHour(hour, document)`.

- [ ] **Step 1: Write the failing test**

Create `tests/ui-theme.test.cjs` with the following test. The source does not yet contain the markers or helpers, so this test must fail first.

```js
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');

function loadThemeApi() {
  const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
  const match = html.match(/\/\* THEME_LOGIC_START \*\/([\s\S]*?)\/\* THEME_LOGIC_END \*\//);
  assert.ok(match, 'ui.html must expose the bounded theme helper');
  const context = {};
  vm.runInNewContext(`${match[1]}; this.themeForHour = themeForHour; this.applyThemeForHour = applyThemeForHour;`, context);
  return context;
}

function fakeDocument() {
  const values = { themeIcon: { textContent: '' }, themeLabel: { textContent: '' } };
  return {
    body: { dataset: {} },
    getElementById(id) { return values[id] || null; },
  };
}

test('uses day from 06:00 through 17:59 and night otherwise', () => {
  const { themeForHour } = loadThemeApi();
  assert.equal(themeForHour(5), 'night');
  assert.equal(themeForHour(6), 'day');
  assert.equal(themeForHour(17), 'day');
  assert.equal(themeForHour(18), 'night');
  assert.equal(themeForHour(99), 'day');
});

test('applies the selected theme and updates the header indicator', () => {
  const { applyThemeForHour } = loadThemeApi();
  const doc = fakeDocument();
  assert.equal(applyThemeForHour(18, doc), 'night');
  assert.equal(doc.body.dataset.theme, 'night');
  assert.equal(doc.getElementById('themeIcon').textContent, '☾');
  assert.equal(doc.getElementById('themeLabel').textContent, 'NIGHT');
});
```

- [ ] **Step 2: Run the test to verify it fails for the intended reason**

Run: `node --test tests/ui-theme.test.cjs`

Expected: FAIL with `ui.html must expose the bounded theme helper`; this confirms the test is waiting for the new feature rather than failing from a syntax mistake.

- [ ] **Step 3: Do not add production code in this task**

Leave `src/ui.html` untouched until Task 2. The red test is the contract for its implementation.

- [ ] **Step 4: Record the test result**

Keep the failing command output in the implementation handoff or task notes. No commit is possible because `C:\Codex` is not a Git repository.

### Task 2: Implement the timed theme helper and header status

**Files:**
- Modify: `src/ui.html:586-615` for the header and initial script setup
- Modify: `src/ui.html:713-718` for the existing clock timer

**Interfaces:**
- Consumes: `themeForHour(hour)` and `applyThemeForHour(hour, document)` required by `tests/ui-theme.test.cjs`.
- Produces: `body[data-theme="day"]` or `body[data-theme="night"]`, plus a visible `#themeIcon` and `#themeLabel` in the chat header.

- [ ] **Step 1: Write the minimal production code required for the red test**

Inside the chat header, add a non-interactive status element after the existing `/preview` pill. It must not replace the existing `#clock` element:

```html
<span class="theme-status" aria-label="ธีมตามเวลาท้องถิ่น">
  <span id="themeIcon" aria-hidden="true">☀</span>
  <span id="themeLabel">DAY</span>
</span>
```

Immediately before the current `tick()` function, add the bounded helper exactly as testable source:

```js
/* THEME_LOGIC_START */
function themeForHour(hour) {
  if (!Number.isInteger(hour) || hour < 0 || hour > 23) return 'day';
  return hour >= 6 && hour < 18 ? 'day' : 'night';
}
function applyThemeForHour(hour, doc) {
  const theme = themeForHour(hour);
  doc.body.dataset.theme = theme;
  const icon = doc.getElementById('themeIcon');
  const label = doc.getElementById('themeLabel');
  if (icon) icon.textContent = theme === 'day' ? '☀' : '☾';
  if (label) label.textContent = theme === 'day' ? 'DAY' : 'NIGHT';
  return theme;
}
/* THEME_LOGIC_END */
function refreshTheme() {
  try { applyThemeForHour(new Date().getHours(), document); }
  catch (_) { applyThemeForHour(-1, document); }
}
```

Replace the one-line `tick(); setInterval(tick, 30000);` initialization with:

```js
tick();
refreshTheme();
setInterval(() => { tick(); refreshTheme(); }, 60000);
```

- [ ] **Step 2: Run the focused test to verify it passes**

Run: `node --test tests/ui-theme.test.cjs`

Expected: both tests PASS. This proves the hour boundaries, fallback, `data-theme`, icon, and label behavior.

- [ ] **Step 3: Refactor only if needed**

Keep the helper between its two required comments and do not duplicate day/night boundary logic elsewhere. Re-run the same test after any cleanup.

- [ ] **Step 4: Record the passing result**

No commit is possible because `C:\Codex` is not a Git repository.

### Task 3: Apply the Overworld CSS system to every existing pane

**Files:**
- Modify: `src/ui.html:7-549`

**Interfaces:**
- Consumes: `body[data-theme]` set by Task 2 and all existing class names/IDs.
- Produces: pixel-block day and night palettes across activity bar, chat, history, tab panels, terminal, preview, notes, settings modal, message states, code blocks, form controls, and responsive rules.

- [ ] **Step 1: Extend the root palette with semantic tokens**

Replace the current dark-only values in `:root` with default day values, then add the night override. Use these names so existing selectors can be updated uniformly:

```css
:root {
  --sky: #82c8ec; --grass: #6eaa3c; --dirt: #805536; --wood: #59452f;
  --stone: #46513e; --panel: #d9e7c5; --panel2: #bed79d; --border: #354629;
  --text: #202b1a; --muted: #53634a; --accent: #d3a332; --accent2: #4d7f2f;
  --code-bg: #1d241a; --code-text: #e9f0d8; --shadow: #26301d;
}
body[data-theme="night"] {
  --sky: #14254c; --grass: #375d31; --dirt: #503825; --wood: #3a3022;
  --stone: #303d38; --panel: #aeb8a8; --panel2: #82978a; --border: #26392d;
  --text: #172117; --muted: #536358; --accent: #d1bb68; --accent2: #83b5c9;
  --code-bg: #111a1a; --code-text: #d8e3d0; --shadow: #0b1220;
}
```

- [ ] **Step 2: Convert global frame and navigation to pixel blocks**

Restyle `body`, `#iconrail`, `.irail-btn`, `.chat-head`, `.logo`, `.pill`, `.tabbar`, `.tab`, `.pane`, `#histpane`, and `#rightpane` to use square corners, 2–4px solid borders, an offset `box-shadow`, and semantic tokens. Preserve grid columns and existing `body.nohist` selectors. Use `border-radius: 0` for Minecraft-like controls; retain a thin visible focus outline:

```css
button:focus-visible, textarea:focus-visible, input:focus-visible {
  outline: 3px solid #f6dc67;
  outline-offset: 2px;
}
.irail-btn.active, .tab.active {
  color: var(--text);
  background: #d2b057;
  box-shadow: inset 3px 3px #eed47b, inset -3px -3px #896b28;
}
```

- [ ] **Step 3: Convert chat and status elements into Overworld components**

Style `#chat` with a CSS-only sky/grass/dirt layered background, then style `.bubble`, `.msg.user .bubble`, `.avatar`, `.inputbox`, `#input`, `#send`, `.statusbar`, `.modelbtn`, `.theme-status`, and `.folderbar`. Keep overflow, max-width, attachment previews, disabled button styling, and `cursor` behavior working. Use an explicit night text color for code blocks:

```css
.bubble pre, .term-out, .fileview {
  background: var(--code-bg);
  color: var(--code-text);
  border: 3px solid var(--border);
}
.theme-status { font: 700 10px/1 monospace; color: var(--text); }
```

- [ ] **Step 4: Restyle every right-panel tab and modal without changing its behavior**

Apply matching inventory/quest styling to `.pane-toolbar`, `.listpane`, `.litem`, `.emptyhint`, `.term`, `.term-head`, `.term-in`, `#notesBox`, `.modal`, `.modal-card`, `.skill-card`, `.toast`, `.think`, `.todosbox`, `.changebox`, and `.toolchip`. Do not change tab `data-tab` values, `hidden` attributes, terminal input IDs, preview iframe, or settings form IDs.

- [ ] **Step 5: Preserve responsive behavior**

Update existing media rules only to use the new tokens and ensure no fixed pixel-art decoration overlays the composer or terminal input. At the existing narrow breakpoint, retain the current single chat-column layout and existing panes hidden by `body.nohist`.

- [ ] **Step 6: Run focused regression tests**

Run: `node --test tests/ui-theme.test.cjs`

Expected: PASS. The test protects the timed-theme behavior while CSS is changed.

- [ ] **Step 7: Record the passing result**

No commit is possible because `C:\Codex` is not a Git repository.

### Task 4: Verify the embedded application end-to-end

**Files:**
- Modify: none unless verification finds a defect
- Test: `tests/ui-theme.test.cjs`

**Interfaces:**
- Consumes: the existing Wry application embedding `include_str!("ui.html")` in `src/gui.rs`.
- Produces: evidence that the desktop app compiles and every existing interaction survives the visual redesign.

- [ ] **Step 1: Run the automated theme regression test**

Run: `node --test tests/ui-theme.test.cjs`

Expected: PASS with two passing tests.

- [ ] **Step 2: Build the release executable**

Run: `cargo build --release`

Expected: PASS and produce `target\\release\\buff.exe` with no compiler errors.

- [ ] **Step 3: Manually verify both time states**

Launch `target\\release\\buff.exe`, open developer tools if available, and run:

```js
applyThemeForHour(6, document);
applyThemeForHour(18, document);
```

Expected: each call changes only `body[data-theme]` and the header ☀/☾ status; the current chat and selected tab remain intact.

- [ ] **Step 4: Manually verify existing controls**

In the desktop app, click Queue, Files, Changes, Preview, Terminal, Notes, the history toggle, folder button, model selector, settings button, and preview pill. Send a harmless message and enter a harmless terminal command such as `echo theme-check`.

Expected: all controls use their existing endpoints and behavior; no click target is hidden by theme decoration.

- [ ] **Step 5: Verify narrow layout**

Resize the desktop window until the existing small-screen media query activates.

Expected: chat composer remains visible and usable; collapsed history/right panels follow the prior behavior; no horizontal overflow blocks the UI.

- [ ] **Step 6: Handle defects test-first**

For any failed condition, first add a focused failing assertion to `tests/ui-theme.test.cjs` when the defect concerns timed theming. For purely visual layout issues, record the exact viewport/state, make the smallest CSS change, then repeat Steps 1–5.

- [ ] **Step 7: Record final verification**

Capture the Node test and Cargo build output in the delivery summary. No commit is possible because `C:\Codex` is not a Git repository.

## Plan Self-Review

- Spec coverage: Tasks 2–3 cover timed day/night palettes and every named pane; Task 4 covers build, tabs, interactions, and narrow layout.
- Test-first coverage: Task 1 creates and runs a failing Node test before Task 2 creates the helper; every later code task reruns it.
- Naming consistency: the test, implementation, and manual checks consistently use `themeForHour`, `applyThemeForHour`, `refreshTheme`, `themeIcon`, and `themeLabel`.
- Completeness scan: every implementation and verification action has an explicit command or code sample.
