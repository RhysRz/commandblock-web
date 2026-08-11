# Commandblock Rename and Windows Installer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver Commandblock as `Commandblock.exe` and `Commandblock-Setup.exe` without breaking existing Buff user data.

**Architecture:** Rename public product strings and the Cargo package output while retaining all data-contract identifiers. Use an Inno Setup script to package the release EXE, shortcut definitions, and uninstaller; a PowerShell wrapper makes the release repeatable.

**Tech Stack:** Rust 2021, Cargo, HTML, PowerShell, Inno Setup 6, Node built-in test runner.

## Global Constraints

- Rename all visible application labels, desktop window title, CLI banner, documentation title, and launch script from `Buff` to `Commandblock`.
- Change the Cargo package name so release builds produce `commandblock.exe`; distribute it as `Commandblock.exe`.
- Keep `.freebuff`, `buff_session.json`, `BUFF_*` environment variables, local WebView data paths, and existing configuration schema unchanged.
- Build `dist/Commandblock-Setup.exe` with Start Menu, optional Desktop shortcut, and uninstaller under the installing user's local application-data area.
- Make installer-created shortcuts run from `{userappdata}\Commandblock` so existing relative user-data files are writable and survive updates.
- Do not package `config.json`, API keys, or existing session data.
- `C:\Codex` is not a Git repository; do not create commits.

---

## File structure

- Create: `tests/commandblock-branding.test.cjs` — public-name and compatibility contract test.
- Create: `installer/Commandblock.iss` — Inno Setup configuration.
- Create: `installer/build-installer.ps1` — release + staging + installer compiler wrapper.
- Modify: `Cargo.toml`, `src/main.rs`, `src/gui.rs`, `src/ui.html`, `README.md`, `start.bat`, `build.rs` — public branding and output name.
- Create: `dist/Commandblock-Setup.exe` — installer artifact.
- Create: `Commandblock.exe` — delivered portable artifact.

### Task 1: Lock down the branding and compatibility contract

**Files:**
- Create: `tests/commandblock-branding.test.cjs`
- Test: `tests/commandblock-branding.test.cjs`

**Interfaces:**
- Produces: checks for the public Commandblock identifiers and the preserved configuration identifiers.

- [x] **Step 1: Write the failing static branding test**

Create `tests/commandblock-branding.test.cjs`:

```js
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const read = (...parts) => fs.readFileSync(path.join(__dirname, '..', ...parts), 'utf8');

test('uses Commandblock publicly while preserving existing data identifiers', () => {
  const cargo = read('Cargo.toml');
  const ui = read('src', 'ui.html');
  const main = read('src', 'main.rs');
  const start = read('start.bat');
  assert.match(cargo, /^name = "commandblock"$/m);
  assert.match(ui, /<title>Commandblock —/);
  assert.match(ui, /<h1>Commandblock /);
  assert.match(main, /Commandblock v\{VERSION\}/);
  assert.match(start, /Commandblock\.exe/i);
  assert.match(main, /const SESSION_FILE: &str = "buff_session\.json"/);
  assert.match(read('src', 'config.rs'), /BUFF_API_KEY/);
});
```

- [x] **Step 2: Run it to confirm the old product name fails the new contract**

```powershell
node --test tests\commandblock-branding.test.cjs
```

Expected: failure because the package, UI, CLI, and batch launcher still use Buff.

### Task 2: Rename public product surfaces without changing data contracts

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Modify: `src/gui.rs`
- Modify: `src/ui.html`
- Modify: `README.md`
- Modify: `start.bat`
- Modify: `build.rs`
- Test: `tests/commandblock-branding.test.cjs`

**Interfaces:**
- Consumes: public name `Commandblock` and legacy internal identifiers from Task 1.
- Produces: Cargo release output `target\release\commandblock.exe` and a `start.bat` that launches `Commandblock.exe`.

- [x] **Step 1: Rename the Cargo package and public descriptions**

Set `Cargo.toml` package fields to:

```toml
name = "commandblock"
description = "Commandblock — ผู้ช่วยพัฒนาโค้ด AI แบบตัวแทน (agentic coding assistant CLI)"
```

- [x] **Step 2: Replace visible UI and desktop title labels**

Change UI title/header/help copy and `WindowBuilder::with_title` from Buff to Commandblock. Keep URLs, icon asset paths, IDs, and current layout unchanged.

- [x] **Step 3: Replace CLI labels and comments that identify the product**

Change user-visible CLI banners, prompts, notes, help copy, and application documentation to Commandblock. Do not replace `.freebuff`, `buff_session.json`, `BUFF_*`, or config keys.

- [x] **Step 4: Rename the portable executable and launcher target**

After the release build, copy `target\release\commandblock.exe` to `Commandblock.exe`; update `start.bat` to start `Commandblock.exe`.

- [x] **Step 5: Run static and Rust verification**

```powershell
node --test tests\commandblock-branding.test.cjs tests\header-command-block-icon.test.cjs tests\input-autogrow.test.cjs
cargo test
```

Expected: all Node tests and Rust tests pass.

### Task 3: Add a repeatable Inno Setup installer

**Files:**
- Create: `installer/Commandblock.iss`
- Create: `installer/build-installer.ps1`
- Test: `tests/commandblock-branding.test.cjs`

**Interfaces:**
- Consumes: `target\release\commandblock.exe`, `assets\buff-command-block.ico`.
- Produces: `dist\Commandblock-Setup.exe` and an installer that does not ship user configuration.

- [x] **Step 1: Extend the static test for installer safeguards**

Append to the branding test:

```js
test('installer provides shortcuts and leaves user configuration out of its payload', () => {
  const iss = read('installer', 'Commandblock.iss');
  assert.match(iss, /OutputBaseFilename=Commandblock-Setup/);
  assert.match(iss, /SetupIconFile=\.\.\\assets\\buff-command-block\.ico/);
  assert.match(iss, /Name: "\{autoprograms\}\\Commandblock"/);
  assert.match(iss, /Name: "\{autodesktop\}\\Commandblock"/);
  assert.doesNotMatch(iss, /config\.json|buff_session\.json|\.freebuff/i);
});
```

- [x] **Step 2: Add the Inno Setup script**

Create `installer/Commandblock.iss`:

```ini
#define AppName "Commandblock"
#define AppVersion "1.0.0"
#define AppExeName "Commandblock.exe"

[Setup]
AppId={{A5721B0D-80D0-466B-8B8B-7E43D0678721}
AppName={#AppName}
AppVersion={#AppVersion}
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\{#AppExeName}
SetupIconFile=..\assets\buff-command-block.ico
OutputDir=..\dist
OutputBaseFilename=Commandblock-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern

[Dirs]
Name: "{userappdata}\Commandblock"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "..\Commandblock.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Commandblock"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{userappdata}\Commandblock"
Name: "{autodesktop}\Commandblock"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{userappdata}\Commandblock"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; WorkingDir: "{userappdata}\Commandblock"; Description: "Launch Commandblock"; Flags: nowait postinstall skipifsilent
```

- [x] **Step 3: Add the installer build wrapper**

Create `installer/build-installer.ps1`:

```powershell
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
cargo build --release
Copy-Item -LiteralPath 'target\release\commandblock.exe' -Destination 'Commandblock.exe' -Force
$iscc = (Get-Command iscc.exe -ErrorAction SilentlyContinue).Source
if (-not $iscc) {
    $iscc = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
}
if (-not (Test-Path -LiteralPath $iscc)) {
    throw 'Inno Setup 6 is required. Install JRSoftware.InnoSetup, then rerun this script.'
}
& $iscc 'installer\Commandblock.iss'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
```

- [x] **Step 4: Install the Inno Setup compiler through winget**

```powershell
winget install --id JRSoftware.InnoSetup --exact --accept-package-agreements --accept-source-agreements
```

Expected: `ISCC.exe` is available at `C:\Program Files (x86)\Inno Setup 6\ISCC.exe` or on `PATH`.

- [x] **Step 5: Run the installer static test**

```powershell
node --test tests\commandblock-branding.test.cjs
```

Expected: all tests pass.

### Task 4: Build and verify both deliverables

**Files:**
- Create: `Commandblock.exe`
- Create: `dist/Commandblock-Setup.exe`

**Interfaces:**
- Consumes: Tasks 1–3.
- Produces: portable and installer artifacts.

- [x] **Step 1: Build the portable executable and installer**

```powershell
powershell -ExecutionPolicy Bypass -File installer\build-installer.ps1
```

- [x] **Step 2: Verify artifacts, tests, and checksums**

```powershell
node --test tests\commandblock-branding.test.cjs tests\header-command-block-icon.test.cjs tests\input-autogrow.test.cjs
cargo test
Get-Item Commandblock.exe, dist\Commandblock-Setup.exe | Select-Object Name, Length
Get-FileHash Commandblock.exe, dist\Commandblock-Setup.exe -Algorithm SHA256
```

Expected: all tests pass and both artifact files have nonzero length and SHA-256 values.
