const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('desktop keeps safe local diagnostics and settings snapshots', () => {
  const main = fs.readFileSync(path.join(__dirname, '..', 'src', 'main.rs'), 'utf8');
  const gui = fs.readFileSync(path.join(__dirname, '..', 'src', 'gui.rs'), 'utf8');
  const ui = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
  const diagnostics = fs.readFileSync(path.join(__dirname, '..', 'src', 'diagnostics.rs'), 'utf8');

  assert.match(main, /diagnostics::install_panic_reporter/);
  assert.match(diagnostics, /join\("CommandBlock"\)/);
  assert.match(diagnostics, /join\("reports"\)/);
  assert.match(diagnostics, /BACKUP_LIMIT: usize = 5/);
  assert.doesNotMatch(diagnostics, /info_payload/);
  assert.match(gui, /\("GET", "\/api\/diagnostics"\)/);
  assert.match(gui, /\("POST", "\/api\/backup"\)/);
  assert.match(ui, /id="diagnosticCopy"/);
  assert.match(ui, /id="backupCreate"/);
  assert.match(ui, /id="backupRestore"/);
});
