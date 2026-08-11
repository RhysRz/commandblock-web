# Buff Obsidian Violet-Only Implementation Plan

> **For agentic workers:** Execute via `executing-plans`; this is a CSS cleanup and release-delivery task.

**Goal:** Remove all Minecraft and terminal legacy CSS so the application has only the Obsidian Liquid Glass violet palette.

**Architecture:** Delete the complete legacy override between the `MINECRAFT OVERWORLD THEME` comment and the `OBSIDIAN LIQUID GLASS` comment. The later Obsidian block remains the sole theme source; replace its multicolor status values and terminal dots with violet-family values.

**Tech Stack:** Embedded HTML/CSS, Rust/Wry, Cargo.

## Global Constraints

- Keep Liquid Glass, aurora, floating tabs, DOM and JavaScript unchanged.
- Active CSS must contain no brown, gold, green, red, yellow, orange or Minecraft palette names.
- No Git repository exists; do not commit.

---

### Task 1: Remove legacy styling and normalize violet status colors

**Files:**
- Modify: `src/ui.html:550-674`, `src/ui.html:690-718`

**Interfaces:**
- Consumes: the Obsidian Liquid Glass CSS block.
- Produces: one authoritative palette with only obsidian, violet, lavender, indigo and white-toned UI colors.

- [ ] **Step 1: Delete the legacy CSS block**

Delete every declaration from `/* ============ MINECRAFT OVERWORLD THEME ============ */` up to but not including `/* ============ OBSIDIAN LIQUID GLASS ============ */`.

- [ ] **Step 2: Normalize the remaining semantic colors**

Set the surviving status variables and terminal indicator colors to violet-family values:

```css
--ok: #b899ff;
--warn: #c7a7ff;
--err: #dfc6ff;
.term-head .dots i:nth-child(1) { background: #dfc6ff; }
.term-head .dots i:nth-child(2) { background: #c7a7ff; }
.term-head .dots i:nth-child(3) { background: #b899ff; }
```

- [ ] **Step 3: Verify palette cleanup**

Run: `rg -n -i 'minecraft|wood|dirt|grass|gold|brown|#(?:[0-9a-f]{0,2}(?:[89a-f][0-9a-f]|[0-9a-f][89a-f]))' src/ui.html`

Expected: no legacy theme block or active brown/gold/green/red/yellow/orange declaration remains; review remaining generic browser/preview white values manually.

### Task 2: Build and deliver

**Files:**
- Modify: `buff.exe` from release output

- [ ] **Step 1: Verify and build**

Run: `cargo test && cargo build --release`

Expected: test suite exits 0 and release executable is generated.

- [ ] **Step 2: Deliver**

Confirm no Buff process is running, copy `target\\release\\buff.exe` to `C:\\Codex\\buff.exe`, and compare SHA-256 hashes for equality.

## Plan Self-Review

- The deletion boundary removes all previously observed brown legacy declarations.
- The retained theme defines only the requested violet-family UI palette.
- Build and delivery commands are explicit.
