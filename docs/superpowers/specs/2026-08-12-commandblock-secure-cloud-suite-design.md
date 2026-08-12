# CommandBlock Secure Cloud Suite Design

## Goal

Improve the installed CommandBlock application without adding a Lite installer or paid code signing: self-updating runtime helpers, contextual Cloud chat with usage reporting, account/device administration, and device-bound Remote PC approval.

## Constraints

- Keep the existing purple CommandBlock UI and GitHub Release distribution.
- Do not add a Lite installer.
- Do not add code signing or require paid external services.
- DeepSeek API keys remain in browser session storage only and are never stored in Supabase.
- Existing users keep their existing conversations, devices, and installed application.
- Remote commands remain scoped to the selected local Connector folder.

## Update lifecycle

The release ZIP contains `Commandblock.exe`, `commandblock-connector.exe`, and `commandblock-updater.exe`. The desktop app verifies the ZIP hash, stages all three files, and launches the updater. The updater writes a temporary command script, exits, and the script waits for CommandBlock to close before replacing all staged binaries and relaunching the app. This permits the updater binary to update itself without attempting to overwrite a running executable.

## Cloud chat and usage

The web adapter loads a bounded recent window for the active conversation before calling the `chat` Edge Function. The function accepts a validated message array rather than only one prompt, forwards it to DeepSeek, and returns content plus provider usage. A `usage_events` table persists exact or estimated input/output/total tokens per authenticated user, conversation, model, and calendar day. The web UI shows the latest request and daily/monthly totals in USD and THB using clearly labelled configurable local price/rate values.

## Account and device administration

The web status bar receives an Account button beside device controls. It opens a mobile-safe modal that reads and edits the profile display name, offers local logout and global logout, and groups Connector and Remote devices. Each device can be renamed or revoked. The modal shows recent security/audit events scoped by RLS to the authenticated owner.

## Device-bound Remote approval

Each Remote host creates a random device secret on first sign-in and stores it in Windows Credential Manager. The secret is never sent to the web page. A browser requesting Remote PC creates a pending session. The host displays a six-digit approval code and saves only a SHA-256 hash to the session. The browser must submit the displayed code; the host validates it locally before accepting. Sessions have a short expiry and are closed when the browser disconnects. Every request, approval, denial, timeout, and revoke is written to the owner audit log. The PIN is a protection against accidental use of an already authenticated account; it does not replace account security.

## Supabase changes

Migration `202608120005_secure_cloud_suite.sql` adds:

- `usage_events` and owner-only RLS/indexes;
- profile update policy if absent;
- remote session fields for approval hash, code submission, device proof, and security timestamps;
- audit action constraints for remote lifecycle events.

The existing `chat` Edge Function validates authenticated ownership and message limits, then writes no API key to the database or logs.

## Verification

- Rust unit tests cover release staging names, updater handoff, secret/code hashing, and expiry decisions.
- Node contract tests assert the release workflow includes all three binaries, UI controls exist, Cloud chat sends bounded history, usage is returned/persisted, and Remote approval never sends the device secret to the browser.
- Run `cargo test`, all relevant `node --test` contracts, web build, migration dry-run, then deploy migration/function after tests pass.
