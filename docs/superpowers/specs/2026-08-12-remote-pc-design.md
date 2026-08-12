# CommandBlock Remote PC Design

## Goal

Allow the signed-in owner of a CommandBlock Desktop Connector to request a remote session from CommandBlock Web, view the desktop, and send mouse/keyboard input after the person at the remote PC explicitly accepts the request.

## Security boundary

- A session belongs to one Supabase user; RLS prohibits any other user from reading or writing it.
- The remote machine shows the requester, mode, and an allow/deny dialog for every new session.
- Sessions expire after ten minutes and are terminated by the remote agent when the dialog is denied, the user presses disconnect, or the owner signs out.
- Remote input is disabled until the remote side grants control, and the agent exposes a visible local stop shortcut.

## Architecture

CommandBlock Web is the controller UI. Supabase stores a small, owner-scoped signaling record. A dedicated Desktop Remote Agent receives signaling and establishes WebRTC with the browser. Screen frames travel peer-to-peer; pointer/keyboard commands travel over a WebRTC data channel. Files and Terminal continue using the existing Desktop Connector relay.

## Connectivity

The first version uses public STUN only, so it has no operating cost but may not connect through restrictive NAT/firewall networks. It reports this clearly and does not silently fall back to an untrusted relay. A later optional TURN configuration can add reliability without changing account or permission rules.

## User flow

1. The desktop owner starts the Remote Agent and signs in.
2. Web shows the online device and a Remote button.
3. User requests view-only or control mode.
4. Remote PC displays an explicit allow/deny dialog and session code.
5. On accept, WebRTC connects; web shows live desktop and a Disconnect button.
6. Disconnect, timeout, or local stop closes the session and removes its signaling records.

## Tests

- Session schema verifies RLS ownership, allowed modes/statuses, expiry, and indexes.
- Desktop contracts prove remote agent is packaged separately and receives no session without an explicit request.
- Web contracts verify Remote controls are hidden before login and always show disconnect/permission state.
