const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('terminal exposes one-click desktop connector and remote actions', () => {
  const ui = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
  assert.match(ui, /id="openConnector"/);
  assert.match(ui, /id="openRemote"/);
  assert.match(ui, /\/api\/desktop-mode/);
});

test('mobile remote dialog has touch actions and a disconnect control', () => {
  const adapter = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');
  assert.match(adapter, /cb-remote-actions/);
  assert.match(adapter, /cb-remote-disconnect/);
  assert.match(adapter, /min-height:44px/);
});
