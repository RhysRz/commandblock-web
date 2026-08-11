# Header Command-Block Icon Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the header robot emoji with the original orange command-block-inspired app image.

**Architecture:** Reuse one compile-time PNG byte constant for both the Winit icon and a local HTTP asset route. The header contains a semantic image element that loads this route; CSS sizes it inside the existing glass logo tile without changing header layout.

**Tech Stack:** Rust 2021, Winit, embedded PNG asset, existing local HTTP server, HTML/CSS, Node built-in test runner.

## Global Constraints

- Keep the existing compact rounded-square glass logo container, position, text, and header controls unchanged.
- Display the PNG with `object-fit: contain`, preserving its transparent background and a small clear inset.
- Serve the existing `assets/buff-command-block.png` from Buff's local HTTP server at `/assets/buff-command-block.png`.
- Do not duplicate or base64-embed the image in `src/ui.html`.
- `C:\Codex` is not a Git repository; do not create commits for this work.

---

## File structure

- Create: `tests/header-command-block-icon.test.cjs` — checks the header's image markup and CSS contract.
- Modify: `src/gui.rs:15,503-511,594` — define and reuse the embedded PNG bytes; expose the PNG local route.
- Modify: `src/ui.html:632,772` — render and style the image in the existing logo tile.
- Modify: `Cargo.lock` — unchanged; no dependency is needed.

### Task 1: Add a regression test for the header asset contract

**Files:**
- Create: `tests/header-command-block-icon.test.cjs`
- Test: `tests/header-command-block-icon.test.cjs`

**Interfaces:**
- Produces: a Node test that reads `src/ui.html` and requires the logo image to use `/assets/buff-command-block.png` with accessible alt text and contain sizing.

- [x] **Step 1: Write the failing test**

Create `tests/header-command-block-icon.test.cjs` with this complete content:

```js
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');

test('header logo uses the shared orange command-block image', () => {
  assert.match(
    html,
    /<div class="logo">\s*<img src="\/assets\/buff-command-block\.png" alt="Buff Command Block">\s*<\/div>/,
  );
  assert.match(html, /\.logo img\s*\{[^}]*object-fit:\s*contain/);
});
```

- [x] **Step 2: Run the test to verify it fails with the current robot emoji**

```powershell
node --test tests\header-command-block-icon.test.cjs
```

Expected: one failing test because `.logo` still contains `🤖`.

### Task 2: Serve and render the shared image

**Files:**
- Modify: `src/gui.rs:15,503-511,594`
- Modify: `src/ui.html:632,772`
- Test: `tests/header-command-block-icon.test.cjs`

**Interfaces:**
- Consumes: `assets/buff-command-block.png` already embedded in the executable.
- Produces: `const COMMAND_BLOCK_ICON_PNG: &[u8]` and `GET /assets/buff-command-block.png` with content type `image/png`.

- [x] **Step 1: Define the shared embedded icon bytes**

Directly below `UI_HTML`, add:

```rust
const COMMAND_BLOCK_ICON_PNG: &[u8] = include_bytes!("../assets/buff-command-block.png");
```

- [x] **Step 2: Reuse the bytes for the Winit icon**

In `build_icon`, replace the `include_bytes!` expression with the shared constant:

```rust
        COMMAND_BLOCK_ICON_PNG,
```

- [x] **Step 3: Add the local PNG route**

Add this arm directly after the root HTML route in `handle`:

```rust
        ("GET", "/assets/buff-command-block.png") => {
            respond(&mut out, 200, "image/png", COMMAND_BLOCK_ICON_PNG)
        }
```

- [x] **Step 4: Replace the header emoji with semantic image markup**

Replace the header logo element with:

```html
    <div class="logo"><img src="/assets/buff-command-block.png" alt="Buff Command Block"></div>
```

- [x] **Step 5: Add contained icon styling**

Add this CSS immediately after the existing final `.logo` rule:

```css
  .logo { overflow: hidden; padding: 3px; }
  .logo img {
    width: 100%; height: 100%; display: block;
    object-fit: contain;
    filter: drop-shadow(0 1px 3px rgba(52, 18, 0, .38));
  }
```

- [x] **Step 6: Run the focused regression test and full Rust suite**

```powershell
node --test tests\header-command-block-icon.test.cjs
cargo test
```

Expected: both commands pass.

### Task 3: Deliver the refreshed executable

**Files:**
- Modify: `target\release\buff.exe` (build output)
- Modify: `buff.exe` (delivered executable)

**Interfaces:**
- Consumes: Tasks 1–2.
- Produces: a current `buff.exe` whose SHA-256 matches `target\release\buff.exe`.

- [x] **Step 1: Build the release executable**

```powershell
cargo build --release
```

Expected: the optimized build succeeds.

- [x] **Step 2: Copy the built executable after confirming Buff is closed**

```powershell
Copy-Item -LiteralPath target\release\buff.exe -Destination buff.exe -Force
```

If the copy is blocked by a running instance, ask the user to close Buff and rerun only this step.

- [x] **Step 3: Verify the delivered binary**

```powershell
$releaseHash = (Get-FileHash target\release\buff.exe -Algorithm SHA256).Hash
$deliveredHash = (Get-FileHash buff.exe -Algorithm SHA256).Hash
if ($releaseHash -ne $deliveredHash) { throw 'Delivered EXE hash does not match release build.' }
"Verified matching executable SHA-256: $releaseHash"
```

Expected: a matching SHA-256 confirmation.
