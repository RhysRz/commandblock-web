const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('agent browser tools use the native bridge and never claim a user click happened', () => {
  const tools = fs.readFileSync(path.join(root, 'src', 'tools.rs'), 'utf8');
  const browserTools = tools.slice(tools.indexOf('fn browser_open'));

  assert.match(tools, /"browser_open"/);
  assert.match(tools, /"browser_inspect"/);
  assert.match(browserTools, /BrowserCommand::Click/);
  assert.doesNotMatch(browserTools, /กรุณาคลิก.*ใน Preview/);
});

test('desktop GUI owns a child WebView and routes browser commands on its UI thread', () => {
  const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');

  assert.match(gui, /build_as_child\(window\)/);
  assert.match(gui, /set_bounds\(/);
  assert.match(gui, /set_visible\(/);
  assert.match(gui, /register_bridge/);
  assert.match(gui, /"\/api\/browser"/);
});

test('Preview reports native browser bounds and exposes confirmation-safe browser controls', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(ui, /id="previewBrowserBack"/);
  assert.match(ui, /id="previewBrowserForward"/);
  assert.match(ui, /ResizeObserver/);
  assert.match(ui, /browserConfirmDialog/);
  assert.match(ui, /\/api\/browser/);
});
