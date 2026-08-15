# Thai Release Summary Design

## Goal

Show a short Thai, user-facing update summary under CommandBlock's update change list while retaining the existing Full Changelog and release link.

## Release generation

- The Windows release workflow compares runtime commits from the previous `build-*` release tag through the new runtime build.
- A checked-in PowerShell helper converts conventional-commit types to Thai action words: `feat` becomes `เพิ่ม`, `fix` becomes `แก้ไข`, and `perf` becomes `ปรับให้เร็วขึ้น`.
- The helper translates CommandBlock vocabulary, including update/download, SESSION, Remote PC, Desktop Connector, plugins, and in-app confirmations. Its test input accepts a single `-Subject` array rather than repeated parameter names, which remains compatible with the runtime Git-log input.
- The release body starts with `## สรุปการอัปเดต`, contains one Thai bullet for each relevant runtime commit, then includes the existing Full Changelog URL.
- When no relevant commit can be turned into a bullet, the helper emits `ปรับปรุงความเสถียรและประสิทธิภาพของ CommandBlock`.

## Desktop UI

- The existing update details area separates the Thai summary from the lower technical changelog text.
- The release URL continues to be exposed by the existing `เปิด release บน GitHub` link.
- User-visible bullets use normal proportional Thai text rather than the small monospace release body.

## Safety and scope

- The workflow uses only local Git metadata and GitHub's existing release token; no AI or paid translation service is introduced.
- It changes neither update eligibility nor package assets/checksums.
- Unrelated user-local changes in `src/config.rs`, `src/diagnostics.rs`, and `buff_session.json.bak` remain unstaged.

## Verification

- Add contract tests for the workflow summary helper and the update UI structure.
- Verify a known commit maps to a Thai bullet and the fallback is present.
- Run all Node tests, Rust tests, release build, whitespace validation, then confirm the published release body and assets.
