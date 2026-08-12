# Terminal Quick Actions and Mobile Remote Controls

## Goal

Let a desktop user start Desktop Connector or Remote PC from the existing Terminal tab without typing a command, and make the CommandBlock Web remote controls comfortable on a phone.

## Desktop terminal

The Terminal header gains two explicit buttons: `Desktop Connector` and `Remote PC`. Each button calls a local GUI endpoint, which launches the exact same sidecar process used by `Commandblock.exe --connector` or `Commandblock.exe --remote`. The terminal prints a clear success or failure line; it never accepts arbitrary process arguments from the browser UI.

## Mobile web

The signed-in web status bar exposes a full-width `Remote PC` entry point on narrow screens. The remote dialog has a sticky action row: `ดูหน้าจอ`, `ควบคุมเครื่อง`, and `ตัดการเชื่อมต่อ`. Buttons use a 44px minimum touch target, the device selector scrolls horizontally, and the canvas takes the remaining visible viewport without horizontal overflow.

## Safety and errors

The two launcher endpoints accept only the identifiers `connector` and `remote`. Remote control remains disabled until the desktop PC accepts the session. Mobile UI continues to show the existing network/P2P error text, and Disconnect always closes the peer connection and marks the session closed.

## Tests

- Rust contract test asserts the launcher accepts exactly two known modes and rejects anything else.
- Web contract asserts mobile Remote markup contains a touch action row and disconnect action.
- JavaScript syntax and the generated Pages bundle must parse successfully.
