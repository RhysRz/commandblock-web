const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('desktop UI exposes a downloadable update notice with visible progress', () => {
  const ui = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
  const gui = fs.readFileSync(path.join(__dirname, '..', 'src', 'gui.rs'), 'utf8');

  assert.match(ui, /id="updateNotice"/);
  assert.match(ui, /id="updateDownload"/);
  assert.match(ui, /id="updateInstall"/);
  assert.match(ui, /id="updateProgress"/);
  assert.match(ui, /\/api\/update/);
  assert.match(gui, /\("GET", "\/api\/update"\)/);
  assert.match(gui, /\("POST", "\/api\/update"\)/);
  assert.match(gui, /"install" => update::launch_staged_update/);
});
