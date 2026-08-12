# Desktop Reliability Design

## Goal

Make CommandBlock updates finish without manual window closing, make local failures diagnosable, preserve settings, and show the user what a release changes.

## Update exit and release notes

`POST /api/update` with `install` will start the existing updater, flush its successful JSON response, then schedule a short process exit. The updater continues to wait for that exact PID and relaunches CommandBlock after replacement. The update status API will expose the release body, publication time, and release URL; the desktop card displays those notes before download.

## Diagnostics

The desktop process installs a panic hook that writes a generic, non-secret crash report to `%LOCALAPPDATA%\\CommandBlock\\reports`. It records only time, app version, build ID, operating system, and source location. The UI offers a copy button for the latest report so users can send a useful support report without exposing API keys or chat text.

## Backup and restore

Before a staged update is applied, CommandBlock saves a rolling local backup of `config.json` and `.freebuff/settings.json` in `%LOCALAPPDATA%\\CommandBlock\\backups`. The Project settings panel lists backups and supports restoring a selected snapshot. At most five snapshots are retained. Missing files are represented as absent and are removed on restore rather than fabricated.

## Verification

Regression tests assert that successful installation schedules a process exit only after responding, diagnostic reports omit panic payloads, backup snapshots retain only approved setting files, and release notes are surfaced from GitHub API data to the UI.
