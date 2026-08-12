# Desktop update progress implementation plan

1. Add a failing contract test for update controls in the local desktop UI.
2. Refactor the update module into observable states: checking, available, downloading, ready, error, and up-to-date.
3. Add fixed local API endpoints for update status and starting a download, including byte progress and checksum validation.
4. Render an accessible update notice, progress bar, and action in the chat footer and poll status while it changes.
5. Run Rust and Node tests, build the release binary, then verify the release artifact flow before publishing.
