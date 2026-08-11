# Commandblock Web Canonical UI Design

## Goal

Publish the existing Commandblock interface in `src/ui.html` as the web application, rather than maintaining a separate web chat shell.

## Decision

The desktop UI remains the single visual and interaction source of truth.  The web deployment is built from that same HTML file and injects a browser-only adapter before the original scripts execute.

## Architecture

1. A deterministic Node build script reads `src/ui.html`, injects the Supabase SDK and `cloud-adapter.js`, and writes a deployable static site directory.
2. `cloud-adapter.js` runs only in the hosted build.  It supplies the existing `/api/*` UI contract in the browser:
   - `state`, `models`, and `model` use browser session state.
   - `chat` authenticates through Supabase and calls the existing `chat` Edge Function using the user-provided DeepSeek key held in session storage only.
   - `history` and `notes` use the current user’s Supabase data.
3. The adapter displays authentication as a full-page gate above the original Commandblock UI.  No second chat page or duplicate layout is rendered.
4. Browser-incompatible desktop features (native folder selection, filesystem, terminal, changes, queue, preview, project skills, and startup scripts) return explicit “Desktop Connector required” results. They must never simulate local access or claim success.
5. The GitHub Pages workflow builds the static site from the canonical desktop UI before uploading it.

## UX Requirements

- After sign-in, users see the same UI, tabs, model chooser, chat renderer, attachments, and purple Commandblock design as the desktop app.
- On first cloud chat, users are asked for a DeepSeek API key.  It stays only in `sessionStorage`; no key is written to Supabase, GitHub, or the page source.
- Cloud chat responses are converted to the SSE frame format expected by the existing UI so the original streaming-render code is reused unchanged.
- Unsupported tabs remain visible, but communicate the real limitation and how to continue on the desktop application.
- Authentication failures, missing keys, expired sessions, and upstream errors surface as Thai messages in the original UI.

## Safety Requirements

- The hosted client uses only the Supabase publishable key.
- The Edge Function remains the only component that contacts DeepSeek.
- No local filesystem, shell, or API secret is exposed to the browser.

## Verification

- A Node contract test builds the site and checks that it contains the canonical Commandblock layout and the cloud adapter before the original scripts.
- Tests check that the adapter uses `sessionStorage` for the key and calls only the configured Supabase project/Edge Function.
- Existing Node tests and `cargo test` remain green.
- GitHub Pages deploy is checked after the push to confirm that the public page is generated from the canonical UI.
