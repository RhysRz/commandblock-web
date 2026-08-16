# Native Preview Browser and AI Browser Actions

## Purpose

CommandBlock Desktop must open public websites that reject iframe embedding (for example Google, YouTube, Facebook, and Instagram) without launching an external browser. It must also give the agent controlled browser actions rather than merely telling the user to click a local preview.

This feature applies to the Windows desktop executable. The GitHub Pages/mobile UI remains an iframe-based preview because a normal web page cannot bypass another site's frame policy.

## User experience

The existing Preview tab bar and URL input remain the single entry point.

- Local project previews continue to render in the existing iframe.
- Public `https://` pages open in a native WebView2 child view positioned over the Preview content area. The CommandBlock shell, chat, tabs, and controls remain visible.
- The Preview controls gain Back, Forward, Refresh, and an indicator showing whether the active view is Local Preview or Browser.
- Browser tabs retain their current URL and title. A blocked iframe is never presented as an error when the native browser is available.
- If WebView2 cannot initialize, CommandBlock leaves the local UI usable and explains how to open the URL externally.

## Architecture

`gui.rs` owns a new desktop-browser controller on the winit UI thread. It creates one Wry child WebView using the same persistent WebContext as the CommandBlock UI, so website cookies and logins persist in CommandBlock's WebView profile.

The UI uses the existing local HTTP server to request browser operations. A typed request channel carries requests from chat/tool worker threads to the winit event loop; the event loop exclusively creates, moves, shows, hides, navigates, and evaluates the child WebView. This avoids sending native WebView objects across threads.

The UI reports the precise Preview pane rectangle and device scale factor whenever its layout changes. The native controller updates the child WebView bounds on resize and hides it whenever a local preview or a non-Preview tab is active.

## Agent tools

The current `preview_open`, `preview_inspect`, `preview_click`, and `preview_fill` tools remain compatible for local previews. New `browser_*` tools target the active native browser page:

- `browser_open(url)` navigates to a validated public `https://` URL.
- `browser_inspect()` returns a compact, sanitized accessibility/DOM snapshot: title, URL, visible text, and actionable controls.
- `browser_click(selector)` clicks an inspected element and waits for navigation or DOM feedback.
- `browser_fill(selector, value)` fills an inspected editable control.
- `browser_press(key)` supports keyboard actions such as Enter, Escape, and Tab.
- `browser_scroll(direction)` scrolls the current page.

Browser script execution is limited to the currently visible WebView. Tool results carry the final URL and a concise status. The agent must inspect before it can act on a selector; arbitrary JavaScript is not exposed as an agent tool.

## Safety and privacy

Only `https://` public URLs are accepted. Credentials embedded in URLs, localhost, and private-network targets are rejected. The native browser uses CommandBlock's existing application profile; no browser data is uploaded by the feature.

The agent may navigate, inspect, scroll, and interact with ordinary page controls. Before any action that can submit a form, post/send a message, purchase, delete, or otherwise change an external account, CommandBlock shows an in-app confirmation naming the website and intended action. The user may cancel without affecting the current page.

Websites can still impose their own login, CAPTCHA, regional, account, or bot-detection restrictions. This design does not bypass them.

## Failure handling

- Navigation failures keep the current browser state and return a readable error.
- Unsupported selectors return an inspection-guided error instead of pretending the click worked.
- Timeouts return the last known URL and an option to inspect again.
- If the native view cannot be shown, the Preview pane explains the limitation and offers the existing external-browser fallback.

## Testing and acceptance

Tests will be written first for URL validation, request routing, tool contracts, confirmation classification, and UI/native-bridge contract markers. Existing preview-tab and plugin tests must remain green.

Manual desktop verification will cover local Preview, Google navigation in native browser mode, tab switching, resize/hide behavior, safe click/fill actions, and a confirmation-required external action. The final Windows release is built only after Node tests, Rust tests, release compilation, and a whitespace diff check pass.

## Scope boundaries

This feature does not attempt a general remote browser service, browser extensions, CAPTCHA solving, login credential storage, or bypassing iframe/CSP protections in the web/mobile edition.
