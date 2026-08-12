# CommandBlock Operations Suite

## Goal

Make CommandBlock practical for daily multi-device use: transparent DeepSeek usage and credit tracking, reliable local connectivity, safe cloud fallback, controlled Remote PC access, and a frictionless desktop update flow.

## Scope and delivery units

This work is deliberately divided into three independently releasable units. They share the existing CommandBlock identity and no unit stores a provider API key in Supabase or GitHub.

### A. Desktop lifecycle

- The existing update banner gains an **Install and restart** action after a verified package is staged. The app launches the existing updater helper, exits cleanly, and the helper replaces only `Commandblock.exe` and `commandblock-connector.exe`.
- Desktop Connector runs as a user-level background process after an explicit opt-in. A Windows system-tray menu shows online/offline state and provides: Open CommandBlock, Connect Desktop, Start Remote PC, Stop, and Exit. It does not run elevated and does not create a Windows service.
- A first-run settings toggle controls whether it starts with Windows. Disabling the toggle removes only the user’s autostart entry.
- A successful interactive Connector login stores only the Supabase refresh token in Windows Credential Manager. Autostart exchanges that token for an access token; it never stores the password. Explicit Connector logout deletes the credential and the autostart entry.

### B. Cloud model reliability and cost transparency

- Both desktop and CommandBlock Web show a compact usage strip next to the selected model: current configured DeepSeek credit in USD, equivalent THB, and a **+ เติมเงิน** button that opens `https://platform.deepseek.com/top_up` in the external browser.
- Credit is an explicitly user-entered local value, never fetched with or inferred from an API key. Users can edit credit USD and the USD/THB rate in a small local settings dialog. The default exchange rate is labelled as an estimate, not financial data.
- For each successful response, show input, output, and total tokens for the most recent request, plus session and daily totals. Use OpenAI-compatible `usage` fields from the terminal stream whenever the provider includes them. If absent, mark the display as an estimate and calculate with a deterministic local approximation.
- Desktop values live in a local settings file. Web values live in browser local storage scoped to the user’s browser. No conversation content, tokens, or keys are newly sent to Supabase for these counters.
- A configurable ordered fallback list applies only to cloud model/API failures that are retryable (429, 5xx, network timeout) or billing-related (402). The UI states which model handled the response. Authentication/configuration errors remain visible and do not silently switch models.

### C. Remote device control

- CommandBlock Web has a **My devices** view listing the signed-in user’s Connector and Remote devices with name, mode, online time, and last-seen timestamp.
- The owner can rename a device or revoke it. Revoke removes the device row and ends pending sessions. The desktop process must register again before it can receive commands.
- The view records an owner-visible, minimal audit event for each Remote request: device, requested mode, approval/denial, start, and end time. It does not record screen frames, keyboard contents, terminal output, or API keys.
- Existing rules remain: every remote session is owner-scoped, expires, and requires confirmation on the target for view/control access.

## API and data boundaries

| Area | Local API / storage | Cloud data |
| --- | --- | --- |
| Update | `GET/POST /api/update`; staged verified ZIP under LocalAppData | GitHub Release only |
| Usage / credit | Desktop settings or browser local storage | None |
| Model fallback | Existing model config plus ordered fallback names | Provider request only |
| Connector tray | Current signed-in connector session | Existing device heartbeats |
| Device management | CommandBlock Web adapter | Owner-scoped device/session/audit rows via Supabase RLS |

## Errors and safety

- An update may only be installed after checksum verification; failed downloads retain the running version.
- A missing token-usage field must not be displayed as an exact value.
- A fallback failure reports both the original and fallback model status without exposing keys.
- Device revoke and remote-control requests are idempotent and owner-scoped through RLS.
- The browser never receives unrestricted filesystem or command execution capability; it continues to send allowlisted commands through Desktop Connector.

## Acceptance criteria

1. A desktop user can see a new build, download with progress, click install/restart, and return to the updated app.
2. A user can opt into Connector autostart, see its tray state, and stop it without ending unrelated Windows processes.
3. Desktop and web show saved USD credit, approximate THB, an external DeepSeek top-up link, and per-message/session/day token counts with exact/estimated labeling.
4. A transient DeepSeek failure uses a configured fallback when available and reports the model used.
5. A signed-in user can list, rename, revoke, and audit only their own remote devices and sessions.

## Non-goals

- Reading DeepSeek account balances, billing history, or payment details through an API key.
- Shared provider API keys, central billing, or a paid service.
- Persistent video/keylogging data or unattended remote control.
