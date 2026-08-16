# Native Preview Browser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show public websites that block iframes in CommandBlock's desktop Preview pane and let the AI inspect and act on that native browser safely.

**Architecture:** Keep local project previews in the existing iframe. Add a typed, thread-safe browser command bridge that forwards UI and AI requests to the winit UI thread, where a Wry child WebView2 is created and positioned over the Preview content rectangle. The browser tools receive compact DOM/action snapshots through WebView evaluation callbacks and require a confirmation round-trip for external state-changing actions.

**Tech Stack:** Rust 2021, Wry 0.56/WebView2, winit 0.30, serde_json, existing local HTTP/SSE GUI, Node built-in test runner.

## Global Constraints

- Windows EXE is the only native-browser target; GitHub Pages/mobile stays iframe-only.
- Public browser navigation accepts only credential-free public `https://` URLs.
- Preserve user-owned changes in `src/config.rs`, `src/diagnostics.rs`, `buff_session.json.bak`, `cbweb.html`, and `preview/`.
- Native WebView objects stay on the winit UI thread; tools may only use a `Send + Sync` request bridge.
- State-changing web actions require an in-app user confirmation; no CAPTCHA, login, or frame-policy bypass exists.
- Add tests before production code and run the full Node/Rust verification suite before release.

---

## File Structure

| File | Responsibility |
| --- | --- |
| `src/browser.rs` | Pure URL/action validation, browser command/request types, JavaScript snapshot/action builders, and the thread-safe bridge registry. |
| `src/lib.rs` | Exposes the `browser` module to `tools.rs` and the GUI binary. |
| `src/tools.rs` | Declares `browser_*` schemas and dispatches agent actions through the bridge instead of pretending that the user clicked. |
| `src/gui.rs` | Registers the bridge, owns the Wry child WebView and window events, serves browser UI endpoints, and forwards confirmation/state events to the page. |
| `src/ui.html` | Reports Preview bounds, switches iframe/native-browser visibility, provides navigation controls and confirmation modal. |
| `tests/native-browser-contract.test.cjs` | Source-level integration contracts for tools, GUI routes, native WebView, and UI bridge markers. |
| `src/browser.rs` tests | Behavioural tests for public URL validation, action validation, and confirmation classification. |

### Task 1: Browser domain model and bridge contract ✅

**Files:**
- Create: `src/browser.rs`
- Modify: `src/lib.rs:1-7`
- Test: `src/browser.rs` inline `#[cfg(test)]` module

**Interfaces:**
- Produces `pub enum BrowserCommand`, `pub struct BrowserReply`, `pub trait BrowserBridge`, `pub fn register_bridge`, and `pub fn dispatch`.
- Consumes `serde_json::Value` and `url::Url`.
- Later tasks call `browser::dispatch(BrowserCommand)` only; they never own `wry::WebView`.

- [ ] **Step 1: Write failing Rust tests for browser validation and confirmation classification**

```rust
#[test]
fn public_https_url_rejects_private_hosts_and_credentials() {
    assert!(validate_public_https("https://www.google.com/").is_ok());
    assert!(validate_public_https("http://www.google.com/").is_err());
    assert!(validate_public_https("https://user:secret@example.com/").is_err());
    assert!(validate_public_https("https://localhost/").is_err());
    assert!(validate_public_https("https://192.168.1.5/").is_err());
}

#[test]
fn submit_like_controls_require_confirmation() {
    assert!(requires_confirmation("button", "Send message", "submit"));
    assert!(requires_confirmation("a", "Delete account", ""));
    assert!(!requires_confirmation("a", "Next page", ""));
}
```

- [ ] **Step 2: Run the focused Rust test and verify it fails because the module/function does not exist**

Run: `cargo test browser::tests --lib`

Expected: compilation failure mentioning missing `browser` module or `validate_public_https`.

- [ ] **Step 3: Implement the minimal pure browser model and bridge**

```rust
#[derive(Debug, Clone)]
pub enum BrowserCommand {
    Show { url: String, bounds: BrowserBounds },
    Hide,
    Navigate { url: String },
    Back,
    Forward,
    Reload,
    Inspect,
    Click { selector: String, confirmed: bool },
    Fill { selector: String, value: String },
    Press { key: String },
    Scroll { direction: ScrollDirection },
}

pub trait BrowserBridge: Send + Sync {
    fn dispatch(&self, command: BrowserCommand) -> BrowserReply;
}
```

Implement `validate_public_https`, `requires_confirmation`, a bounded `BrowserReply`, and an `OnceLock<Mutex<Option<Arc<dyn BrowserBridge>>>>` registry. Return `Browser unavailable: open CommandBlock Desktop first` when no bridge is registered.

- [ ] **Step 4: Run the focused Rust test and verify it passes**

Run: `cargo test browser::tests --lib`

Expected: PASS for URL and confirmation tests.

- [ ] **Step 5: Commit the independently testable domain layer**

```powershell
git add src/browser.rs src/lib.rs
git commit -m "feat(browser): add native browser command bridge"
```

### Task 2: Agent browser tools and contracts ✅

**Files:**
- Modify: `src/tools.rs:76-264, 1133-1215`
- Modify: `src/main.rs:790-804`
- Create: `tests/native-browser-contract.test.cjs`
- Test: `tests/native-browser-contract.test.cjs`

**Interfaces:**
- Consumes `browser::validate_public_https` and `browser::dispatch` from Task 1.
- Produces tool names `browser_open`, `browser_inspect`, `browser_click`, `browser_fill`, `browser_press`, and `browser_scroll`.
- Later GUI work accepts the corresponding `BrowserCommand` values.

- [ ] **Step 1: Write a failing contract test for declared and dispatched browser tools**

```js
test('agent browser tools use the native bridge and never claim a user click happened', () => {
  const tools = fs.readFileSync(path.join(root, 'src', 'tools.rs'), 'utf8');
  assert.match(tools, /"browser_open"/);
  assert.match(tools, /"browser_inspect"/);
  assert.match(tools, /BrowserCommand::Click/);
  assert.match(tools, /requires_confirmation/);
  assert.doesNotMatch(tools, /กรุณาคลิก.*ใน Preview/);
});
```

- [ ] **Step 2: Run the contract test and verify it fails for missing browser tool declarations**

Run: `node --test tests/native-browser-contract.test.cjs`

Expected: FAIL because `browser_open` and `BrowserCommand::Click` are absent.

- [ ] **Step 3: Implement tool schemas, validation, and exact result handling**

Add the six tool schemas to `TOOL_NAMES` and `tool_schemas`. For each handler, validate arguments before dispatching a typed command. `browser_click` first requests a click analysis; if `BrowserReply::ConfirmationRequired` is returned, expose the website/action and stop without clicking. Update rule 9 in `system_prompt()` so the agent must call `browser_inspect` before a selector action, may report real completed results only, and must request user confirmation when the tool returns a pending confirmation.

```rust
fn browser_open(args: &Value) -> String {
    let url = arg_str(args, "url").unwrap_or_default();
    match browser::validate_public_https(url) {
        Ok(url) => browser::dispatch(BrowserCommand::Navigate { url }),
        Err(error) => BrowserReply::error(error),
    }.to_tool_text()
}
```

- [ ] **Step 4: Run the focused contract test and Rust tests**

Run: `node --test tests/native-browser-contract.test.cjs; cargo test browser::tests --lib`

Expected: both commands PASS.

- [ ] **Step 5: Commit agent browser tools**

```powershell
git add src/tools.rs src/main.rs tests/native-browser-contract.test.cjs
git commit -m "feat(browser): add agent browser actions"
```

### Task 3: Native child WebView controller and GUI bridge ✅

**Files:**
- Modify: `src/gui.rs:1-23, 540-625, 700-825, 1760-1800`
- Modify: `tests/native-browser-contract.test.cjs`
- Test: `tests/native-browser-contract.test.cjs`, `cargo test`

**Interfaces:**
- Consumes `BrowserBridge`, `BrowserCommand`, `BrowserReply`, and `BrowserBounds` from Task 1.
- Consumes the six tool actions from Task 2.
- Produces `NativeBrowserController` owned only by `run_desktop_window` and `POST /api/browser/*` routes for the UI.

- [ ] **Step 1: Extend the contract test with failing native-controller expectations**

```js
test('desktop GUI owns a child WebView and routes browser commands on its UI thread', () => {
  const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');
  assert.match(gui, /build_as_child\(&window\)/);
  assert.match(gui, /set_bounds\(/);
  assert.match(gui, /set_visible\(/);
  assert.match(gui, /register_bridge/);
  assert.match(gui, /"\/api\/browser"/);
});
```

- [ ] **Step 2: Run the contract test and verify it fails because no child view/controller exists**

Run: `node --test tests/native-browser-contract.test.cjs`

Expected: FAIL at `build_as_child(&window)`.

- [ ] **Step 3: Implement the GUI-thread native controller**

Refactor `run_desktop_window` to use a `winit::event_loop::EventLoop<DesktopEvent>` and retain the `EventLoopProxy`. Register an `MpscBrowserBridge` that sends `DesktopEvent::Browser { command, response }` and waits with a finite timeout. In the application handler, create a Wry child view with `WebViewBuilder::new_with_web_context` using the existing persistent `WebContext`, `with_bounds`, and `build_as_child(&window)` when the first Browser `Show`/`Navigate` event arrives.

Implement navigation (`load_url`, `go_back`, `go_forward`, `reload`), bounds updates (`set_bounds`), visibility (`set_visible`), and JavaScript evaluation callbacks for inspect/click/fill/press/scroll. Sanitize returned snapshots to title, URL, up to 80 visible controls, and at most 8,000 characters of visible text.

`browser_click` must first evaluate the target element and return a confirmation-required reply for submit/delete/send/payment-like labels or form-submit controls. A confirmed retry evaluates `.click()` and returns the observed final URL/status.

Add `POST /api/browser` parsing for UI commands and route its reply as JSON. Send SSE event `browser_state` after AI browser tools complete so the UI can update active-tab state.

- [ ] **Step 4: Run controller contracts and full Rust tests**

Run: `node --test tests/native-browser-contract.test.cjs; cargo test`

Expected: contract test and all Rust tests PASS.

- [ ] **Step 5: Commit the native controller**

```powershell
git add src/gui.rs tests/native-browser-contract.test.cjs
git commit -m "feat(browser): embed native preview webview"
```

### Task 4: Preview UI, confirmation dialog, and release verification ✅

**Files:**
- Modify: `src/ui.html:520-570, 1270-1290, 1510-1590, event handlers`
- Modify: `tests/native-browser-contract.test.cjs`
- Modify: `Cargo.toml`, `Cargo.lock`, `tests/session-version-contract.test.cjs`
- Test: all Node tests, all Rust tests, release build

**Interfaces:**
- Consumes `/api/browser` from Task 3 and the existing preview tab state helper.
- Produces a layout payload `{ x, y, width, height, scale }`, user navigation controls, native-browser visibility transitions, and confirmation retries.

- [ ] **Step 1: Extend the contract test with failing UI bridge expectations**

```js
test('Preview reports native browser bounds and exposes confirmation-safe browser controls', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');
  assert.match(ui, /previewBrowserBack/);
  assert.match(ui, /previewBrowserForward/);
  assert.match(ui, /ResizeObserver/);
  assert.match(ui, /browserConfirmDialog/);
  assert.match(ui, /\/api\/browser/);
});
```

- [ ] **Step 2: Run the contract test and verify it fails because the UI controls do not exist**

Run: `node --test tests/native-browser-contract.test.cjs`

Expected: FAIL at `previewBrowserBack`.

- [ ] **Step 3: Implement the Preview Browser UI**

Add Back, Forward, Refresh, and mode label controls next to the existing Preview URL field. On a public URL, call `/api/browser` with `Navigate`, hide the iframe, and use `ResizeObserver` plus `getBoundingClientRect()` to report the browser viewport. On local preview or tab close/switch, call `Hide` and restore the iframe.

Add an in-app `browserConfirmDialog` styled with the existing Obsidian-purple dialogs. It must identify the site and exact action, offer Cancel/Confirm, and issue the confirmed action only after the user clicks Confirm. Wire `browser_state` SSE events to tab URL/title updates. Retain the external browser button as a WebView2 fallback only.

Increase the package/version contract from `1.0.15` to the next patch release consistently in `Cargo.toml`, `Cargo.lock`, and the version test.

- [ ] **Step 4: Run web build and all automated checks**

Run: `node scripts/build-web.mjs; node --test tests/*.test.cjs; cargo test; cargo build --release; git diff --check`

Expected: every Node and Rust test PASS, release EXE builds, and `git diff --check` is silent.

- [ ] **Step 5: Manually verify the desktop flow**

1. Launch `target/release/commandblock.exe`.
2. Open Preview and enter `https://www.google.com/`; verify it renders inside the Preview pane, not in Chrome and not as an iframe refusal.
3. Resize the CommandBlock window and switch Preview tabs; verify the native page tracks the pane and hides when a local Preview is active.
4. Ask the AI to inspect a public page then click a harmless inspected navigation control; verify the action and its result appear in chat.
5. Attempt an action labelled Send/Delete/Buy; verify the Obsidian confirmation dialog appears and Cancel preserves the page.

- [ ] **Step 6: Commit, push, and publish the verified release**

```powershell
git add src/ui.html src/gui.rs src/tools.rs src/browser.rs src/lib.rs Cargo.toml Cargo.lock tests
git commit -m "feat(browser): add native preview automation"
git push origin main
```

Wait for the Windows release workflow to upload both `CommandBlock-Windows-x64.zip` and its `.sha256` asset before announcing the update.

## Plan Self-Review

- Spec coverage: Tasks 1-4 cover native Preview display, UI-thread ownership, agent inspect/action tools, confirmation safety, fallback/error behavior, testing, and Windows release.
- Scope check: mobile/web frame limits and prohibited bypasses remain explicit scope boundaries; no remote browser service or extensions are included.
- Type consistency: all later tasks use the `BrowserCommand`/`BrowserReply`/`BrowserBridge` names introduced by Task 1.
- Placeholder scan: the plan contains no unresolved placeholders or deferred implementation steps; each task has a failing test, exact verification command, implementation interface, and commit boundary.
