# Commandblock Rename and Windows Installer Design

## Goal

Rename Buff's user-facing product identity to `Commandblock` and deliver a Windows installer named `Commandblock-Setup.exe`.

## Branding scope

- Rename all visible application labels, desktop window title, CLI banner, documentation title, and launch script from `Buff` to `Commandblock`.
- Change the Cargo package name so release builds produce `commandblock.exe`; distribute it as `Commandblock.exe`.
- Keep internal compatibility names unchanged: `.freebuff`, `buff_session.json`, `BUFF_*` environment variables, local WebView data paths, and existing configuration schema remain untouched so existing settings and history continue working.

## Installer

- Use Inno Setup to create `dist/Commandblock-Setup.exe` from the release executable.
- Install to `%LOCALAPPDATA%\Programs\Commandblock`, with Start Menu and optional Desktop shortcuts using the existing orange command-block icon; this avoids elevation and keeps installation and user data under the installing account.
- Include an uninstaller. It removes installed program files but leaves user-created configuration and session data in place.
- Have all installer-created shortcuts launch with `%APPDATA%\Commandblock` as their working directory. This writable folder holds `config.json`, `buff_session.json`, and `.freebuff` without placing credentials in Program Files or the installer payload.
- Do not package `config.json`, API keys, or existing session data. A new install creates its normal default configuration at first launch.

## Build and verification

- Add a repeatable PowerShell build script that compiles release Rust output, copies it to the installer staging name, and invokes Inno Setup.
- Add static tests that assert the UI name, Cargo package name, output EXE name, and installer safeguards.
- Build and verify both `Commandblock.exe` and `dist/Commandblock-Setup.exe`.
