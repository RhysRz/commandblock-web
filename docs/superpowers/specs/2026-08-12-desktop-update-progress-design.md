# CommandBlock desktop update progress

## Goal

Make a new CommandBlock desktop release visible and user-controlled instead of silently downloading it.

## Behaviour

1. On launch, the desktop app checks the latest GitHub release in the background.
2. If the release differs from the embedded build id, the chat footer displays an update notice and a **Download** button.
3. After the user clicks download, the notice stays visible and renders a progress bar, percentage, and byte counts whenever the server supplies a content length.
4. The downloaded ZIP is SHA-256 checked, then only the two expected EXEs are staged locally.
5. On success, the notice tells the user to close and reopen CommandBlock. The existing updater helper applies the staged version during the next launch.
6. Failed checks or downloads surface a safe Thai error and do not replace the installed EXE.

## Boundaries

- Releases are read only from the existing GitHub release endpoint.
- The local UI invokes only a fixed `download` action; it accepts no URL or path from the browser.
- No automatic overwrite occurs while the app is running.
