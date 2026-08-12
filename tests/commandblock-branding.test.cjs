const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const read = (...parts) => fs.readFileSync(path.join(__dirname, '..', ...parts), 'utf8');

test('uses CommandBlock publicly while preserving existing data identifiers', () => {
  const cargo = read('Cargo.toml');
  const ui = read('src', 'ui.html');
  const main = read('src', 'main.rs');
  const start = read('start.bat');
  assert.match(cargo, /^name = "commandblock"$/m);
  assert.match(ui, /<title>CommandBlock —/);
  assert.match(ui, /<h1>CommandBlock /);
  assert.match(main, /CommandBlock v\{VERSION\}/);
  assert.match(start, /Commandblock\.exe/i);
  assert.match(main, /const SESSION_FILE: &str = "buff_session\.json"/);
  assert.match(read('src', 'config.rs'), /BUFF_API_KEY/);
});

test('installer provides shortcuts and preserves user configuration while seeding a key-free template', () => {
  const iss = read('installer', 'Commandblock.iss');
  assert.match(iss, /OutputBaseFilename=Commandblock-Setup/);
  assert.match(iss, /PrivilegesRequired=lowest/);
  assert.match(iss, /DefaultDirName=\{localappdata\}\\Programs\\\{#AppName\}/);
  assert.match(iss, /SetupIconFile=\.\.\\assets\\buff-command-block\.ico/);
  assert.match(iss, /Name: "\{autoprograms\}\\Commandblock"/);
  assert.match(iss, /Name: "\{autodesktop\}\\Commandblock"/);
  assert.match(iss, /Source: "config-template\.json"; DestDir: "\{userappdata\}\\Commandblock"; DestName: "config\.json"; Flags: onlyifdoesntexist/i);
  assert.doesNotMatch(iss, /Source: "\.\.\\config\.json"|buff_session\.json|\.freebuff/i);
  const template = read('installer', 'config-template.json');
  assert.doesNotMatch(template, /gsk_|sk-or-v1-|AIza|sk-[A-Za-z0-9]{16,}/i);
});

test('installer build script finds per-user Inno Setup installations', () => {
  const script = read('installer', 'build-installer.ps1');
  assert.match(script, /\$env:LOCALAPPDATA.*Inno Setup 6.*ISCC\.exe/);
});
