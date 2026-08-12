# Commandblock Desktop Connector Design

## Goal

Let Commandblock Web access an explicitly connected Windows Commandblock instance for project files, folder selection, terminal commands, changes, queue, and previews without exposing a local port or the user’s filesystem to the public internet.

## Architecture

The connector is a `Commandblock.exe --connector` mode. It signs in to the existing Supabase project with the user’s email and password for the current process, registers a local device ID, then long-polls Supabase over outbound HTTPS for commands addressed to that device. The web application, authenticated as the same user, inserts commands and polls their status. No inbound port, router configuration, service role key, or provider API key is used.

## Data model

- `connector_devices`: one row per connected desktop, owned by `user_id`, with a random local device ID, display name, current project root, and heartbeat timestamp.
- `connector_commands`: one row per requested action, owned by `user_id` and addressed to one device. It contains an action name, JSON payload, lifecycle state (`queued`, `running`, `completed`, `rejected`, `failed`), result JSON, and timestamps.
- Row-level security lets only the owner read or create these rows. The connector authenticates as that owner; it cannot see another account’s devices or commands.

## Connector behavior

1. The user starts `Commandblock.exe --connector`, signs in, and chooses a local project folder through the native Windows picker.
2. The connector stores the session only in memory, sends a heartbeat, and waits for commands.
3. It accepts only commands for its device and only paths resolving inside the selected project root.
4. `files`, `read`, `changes`, `queue`, and `preview` return data automatically. `exec` is shown to the desktop user and is rejected unless that user approves it.
5. Stopping the process clears its session and its heartbeat becomes stale; the web marks it offline.

## Web behavior

- The settings panel lists the user’s connected desktops and allows one active device.
- Existing `/api/files`, `/api/read`, `/api/changes`, `/api/queue`, `/api/pick-folder`, and `/api/exec` calls are adapted into a queued connector command when an active device is online.
- If no device is selected or connected, the UI gives a specific connector instruction instead of pretending the action ran.
- The Cloud chat remains separate from local execution; this phase enables the existing file/terminal panes, not autonomous cloud-agent tool execution.

## Safety requirements

- The browser never receives the desktop session or a filesystem path outside the selected root.
- The connector never stores the user password or DeepSeek key.
- The Supabase publishable key is the only key embedded in web or desktop source.
- Every command is user-scoped, device-scoped, auditable, and bounded by a timeout.

## Verification

- SQL contract tests verify RLS ownership and device-targeted commands.
- Rust tests verify path confinement and command lifecycle payloads.
- Web contract tests verify connector requests require a selected online device.
- A live test uses two browser/device sessions in the same account to confirm a folder request reaches only the selected connector.
