const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('the faster updater release increments the desktop package version', () => {
  const cargo = fs.readFileSync(path.join(__dirname, '..', 'Cargo.toml'), 'utf8');
  assert.match(cargo, /^version = "1\.0\.3"$/m);
});

test('SESSION list uses a low-glare obsidian surface instead of white cards', () => {
  const ui = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
  assert.match(ui, /\.session-item\s*\{[^}]*background:\s*rgba\(30,\s*20,\s*48/);
  assert.doesNotMatch(ui, /\.session-item\s*\{[^}]*background:\s*#fff/i);
});
