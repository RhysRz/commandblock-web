# Buff Floating Glass Tabs Implementation Plan

> **For agentic workers:** Execute via `executing-plans`. This is a CSS-only visual change; verify through release build and the existing tab behavior.

**Goal:** Make the right-pane top tabs a floating glass pill tray with a violet active state.

**Architecture:** Modify only the final Obsidian Liquid Glass CSS block in `src/ui.html`. Existing tab buttons, `data-tab` values, `switchTab` JavaScript and horizontal overflow behavior remain unchanged.

**Tech Stack:** Embedded HTML/CSS, Rust/Wry, Cargo.

## Global Constraints

- Do not modify HTML or JavaScript.
- Preserve tab labels and click behavior.
- Keep horizontal scrolling when the right pane is narrow.
- No Git repository exists; do not commit.

---

### Task 1: Apply and deliver the floating tab tray

**Files:**
- Modify: `src/ui.html:807-810`

**Interfaces:**
- Consumes: existing `.tabbar`, `.tab`, and `.tab.active` elements.
- Produces: floating glass tray styling without behavior changes.

- [ ] **Step 1: Add the CSS override**

Append these declarations at the end of the Obsidian Liquid Glass block:

```css
.tabbar {
  margin: 10px 10px 0;
  padding: 5px;
  gap: 6px;
  border: 1px solid rgba(255, 255, 255, .11);
  border-radius: 14px;
  background: rgba(8, 8, 16, .58);
  box-shadow: inset 0 1px rgba(0, 0, 0, .45);
}
.tab { border: 1px solid transparent; border-radius: 10px; }
.tab.active {
  border-color: rgba(216, 194, 255, .30);
  background: linear-gradient(135deg, rgba(113, 65, 212, .86), rgba(177, 109, 243, .78));
  box-shadow: 0 7px 17px rgba(113, 59, 221, .36), inset 0 1px rgba(255, 255, 255, .25);
}
```

- [ ] **Step 2: Build and verify**

Run: `cargo test && cargo build --release`

Expected: Rust suite exits 0 and release executable is created.

- [ ] **Step 3: Deliver executable**

Confirm Buff is closed. Copy `target\\release\\buff.exe` to `C:\\Codex\\buff.exe`, then compare SHA-256 hashes and require equality.

## Plan Self-Review

- CSS targets only the existing three tab selectors.
- Functional tab behavior remains untouched.
- Build and delivery commands are explicit.
