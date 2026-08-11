# Buff Obsidian Liquid Glass Implementation Plan

> **For agentic workers:** Execute with `executing-plans` task-by-task. This CSS-only redesign has no new runtime behavior that merits a unit test; verification protects existing app behavior through build, JavaScript parsing, and desktop smoke checks.

**Goal:** Replace the visible Minecraft/Terminal appearance with a premium dark Obsidian Liquid Glass interface using animated purple-blue aurora.

**Architecture:** Keep one production file, `src/ui.html`. Remove the recently added timed-theme markup and JavaScript, then add one final CSS override layer that wins over existing legacy selectors while preserving the HTML, IDs, event handlers, and backend interface. All motion is a decorative background pseudo-element and respects reduced-motion preferences.

**Tech Stack:** Embedded HTML/CSS/JavaScript, Rust/Wry, Cargo, Node.js built-in JavaScript parser.

## Global Constraints

- Preserve layout, all existing interaction IDs, events, routes, fetch calls and responsive behavior.
- Use no external images, fonts or dependencies.
- Use glass on application surfaces; use opaque near-black surfaces for terminal, code, file preview, notes and composer.
- Use 18-second purple-blue aurora animation; disable it with `prefers-reduced-motion: reduce`.
- Remove the timed theme indicator and its obsolete regression test.
- No Git repository exists; do not commit.

---

### Task 1: Remove the temporary timed-theme behavior

**Files:**
- Modify: `src/ui.html:721-724`, `src/ui.html:841-865`
- Delete: `tests/ui-theme.test.cjs`

**Interfaces:**
- Consumes: the original `tick()` clock function and `#clock` element.
- Produces: the original independent 30-second clock update with no `data-theme` or theme status dependency.

- [ ] **Step 1: Remove obsolete UI markup and helpers**

Delete the `span.theme-status` with `#themeIcon` and `#themeLabel`. Delete the source from `/* THEME_LOGIC_START */` through `/* THEME_LOGIC_END */` and delete `refreshTheme()`. Restore this clock initializer:

```js
tick();
setInterval(tick, 30000);
```

- [ ] **Step 2: Remove the obsolete test**

Delete `tests/ui-theme.test.cjs`; it exclusively tests the removed `themeForHour` and `applyThemeForHour` behavior.

- [ ] **Step 3: Parse all embedded scripts**

Run:

```powershell
node -e "const fs=require('fs'); const html=fs.readFileSync('src/ui.html','utf8'); const scripts=[...html.matchAll(/<script[^>]*>([\s\S]*?)<\/script>/g)].map(m=>m[1]); scripts.forEach((script,index)=>new Function(script)); console.log('Embedded UI JavaScript syntax: OK ('+scripts.length+' scripts)');"
```

Expected: the two embedded scripts parse successfully.

### Task 2: Apply the Obsidian Liquid Glass CSS layer

**Files:**
- Modify: `src/ui.html:550-672`

**Interfaces:**
- Consumes: every existing UI selector and current app DOM.
- Produces: Obsidian background, decorative reduced-motion-safe aurora, glass surfaces, violet active states, and opaque editing/code surfaces.

- [ ] **Step 1: Add global surface tokens and the aurora layer**

Append a final CSS override block defining `--obsidian`, `--glass`, `--glass-strong`, `--violet`, `--violet-bright`, `--ink`, `--text`, and `--muted`. Apply the obsidian background to `body`; use `body::before` for absolute, pointer-events-none radial gradients and `@keyframes aurora-drift` with an 18-second animation. Add:

```css
@media (prefers-reduced-motion: reduce) {
  body::before { animation: none; }
}
```

- [ ] **Step 2: Restyle panels as glass**

Override `#iconrail`, `.chat-head`, `#histpane`, `.tabbar`, `.pane`, `.modelmenu`, `.modal-card`, `.modal-head`, `.modal-foot`, and `#toast` with a translucent `var(--glass)`, `backdrop-filter: blur(20px) saturate(130%)`, thin white-transparent borders and restrained shadows. Keep all content above the aurora with `position: relative; z-index: 1`.

- [ ] **Step 3: Restyle conversation and controls**

Make assistant bubbles dark lavender glass and user bubbles a violet gradient. Apply violet glow only to active rail/tab states and primary send/save actions. Keep `.term`, `.term-out`, `.bubble pre`, `.bubble code`, `.fileview`, `textarea#notesBox`, `.inputbox`, and `.term-in` opaque `var(--ink)` with readable text.

- [ ] **Step 4: Preserve accessibility and narrow layout**

Keep focus outlines visible in `:focus-visible`, retain all existing narrow-layout media rules, and ensure the decorative aurora cannot cover pointer targets.

### Task 3: Build, inspect and deliver the executable

**Files:**
- Modify: `buff.exe` from the release output

**Interfaces:**
- Consumes: `target\\release\\buff.exe` produced by Cargo.
- Produces: `C:\\Codex\\buff.exe` with the new visual theme.

- [ ] **Step 1: Build and run the full Rust test suite**

Run: `cargo test && cargo build --release`

Expected: test suite exits 0 and release output exists at `target\\release\\buff.exe`.

- [ ] **Step 2: Smoke-check interface coverage**

Launch the release executable, verify the chat, history, each right-side tab, model menu and settings modal remain usable, then resize the window to its narrow layout. Confirm glass surfaces appear over the animated aurora and Terminal/code/composer remain opaque.

- [ ] **Step 3: Copy the verified release executable**

Confirm no `buff` process is running, then run:

```powershell
Copy-Item -LiteralPath 'C:\Codex\target\release\buff.exe' -Destination 'C:\Codex\buff.exe' -Force
```

Compare SHA-256 hashes of both files and require equality.

## Plan Self-Review

- Scope coverage: Task 1 removes the unwanted prior behavior; Task 2 applies every visual rule; Task 3 verifies and delivers the desktop app.
- Component integrity: no IDs, event handlers, endpoints or DOM roles are changed.
- Completeness: each modification and verification command is explicit.
