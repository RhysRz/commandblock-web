# Thai Release Summary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate Thai release bullets automatically and render them under the update change-list heading.

**Architecture:** A PowerShell helper converts runtime commit subjects to a Markdown release summary. The desktop updater parses the summary into normal Thai bullet elements while retaining the technical changelog and GitHub release link.

**Tech Stack:** GitHub Actions, PowerShell, HTML/CSS/browser JavaScript, Node built-in test runner, Rust/Cargo.

## Global Constraints

- Use local Git metadata and the existing GitHub release token only; no AI or paid translation service.
- Preserve update eligibility, ZIP assets, checksums, and release URL.
- Keep `src/config.rs`, `src/diagnostics.rs`, and `buff_session.json.bak` unstaged.
- Release version is `1.0.9`.

---

### Task 1: Add a failing release-summary contract

**Files:**

- Create: `tests/thai-release-summary-contract.test.cjs`
- Modify: `tests/session-version-contract.test.cjs`

**Interfaces:**

- Consumes: release helper, release workflow, and `src/ui.html`.
- Produces: a regression contract requiring Thai mappings, fallback text, a notes-file workflow call, and summary UI elements.

- [ ] Write assertions for `Convert-CommitSubjectToThai`, the Thai fallback, `render-release-notes.ps1`, `--notes-file release-notes.md`, `#updateNotesSummary`, and `splitReleaseNotes`.
- [ ] Run `node --test tests/thai-release-summary-contract.test.cjs`; it must fail before production code exists.
- [ ] Change the desktop package expectation to `1.0.8`.

### Task 2: Create deterministic Thai release notes

**Files:**

- Create: `tools/render-release-notes.ps1`
- Modify: `.github/workflows/release-windows.yml`

**Interfaces:**

- Consumes: `-PreviousTag`, `-BuildId`, `-Repository`, and an optional `-Subject` string array.
- Produces: Markdown starting with `## สรุปการอัปเดต`, Thai bullets, `## รายละเอียดการเปลี่ยนแปลง`, and a Full Changelog URL.

- [ ] Implement `Convert-CommitSubjectToThai` for confirmations, update/download/install, SESSION/pin, plugins, Remote PC, Desktop Connector, mobile UI, and generic feature/fix/performance/refactor subjects.
- [ ] Emit `- ปรับปรุงความเสถียรและประสิทธิภาพของ CommandBlock` if no subject maps to a bullet.
- [ ] Set checkout `fetch-depth: 0`, find the prior `build-*` tag, call the helper to write `release-notes.md`, and replace `--generate-notes` with `--notes-file release-notes.md`.
- [ ] Run the helper with a `-Subject` array containing `feat(ui): replace native confirmations` and `fix(update): retry stalled downloads`; it must output Thai bullets about confirmation and update download retry.

### Task 3: Render the Thai summary in the updater

**Files:**

- Modify: `src/ui.html`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

**Interfaces:**

- Consumes: release body text in `s.release_notes`.
- Produces: `splitReleaseNotes(notes)` returning `{ summary: string[], details: string }` to `renderUpdate`.

- [ ] Add `#updateNotesSummary` and `#updateNotesSummaryList` before the current technical `pre`, with Obsidian styling and hidden state when no bullets exist.
- [ ] Parse only bullets between `## สรุปการอัปเดต` and `## รายละเอียดการเปลี่ยนแปลง`.
- [ ] Create list elements through `textContent`, retain Full Changelog in technical text, and preserve the existing GitHub release link.
- [ ] Set Cargo version to `1.0.8`; let the release build update the lockfile.

### Task 4: Verify and publish

**Files:**

- Modify: workflow, helper, UI, Cargo files, tests, and this plan.

**Interfaces:**

- Consumes: the feature commit on `main`.
- Produces: a Windows release with Thai summary bullets and downloadable assets.

- [ ] Run focused Node contracts, all Node tests, Rust tests, release build, and `git diff --check`.
- [ ] Commit only `.github/workflows/release-windows.yml`, `tools/render-release-notes.ps1`, `src/ui.html`, Cargo files, feature tests, and this plan using `feat(updater): add Thai release summaries`.
- [ ] Push `main` and verify the latest release starts with Thai summary bullets, has Full Changelog, ZIP, and checksum assets.
