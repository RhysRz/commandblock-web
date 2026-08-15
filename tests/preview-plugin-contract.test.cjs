const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('preview capability is allowlisted to CommandBlock local preview and recorded in work activity', () => {
  const tools = fs.readFileSync(path.join(root, 'src', 'tools.rs'), 'utf8');
  const connector = fs.readFileSync(path.join(root, 'src', 'connector.rs'), 'utf8');
  const adapter = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(tools, /"preview_open"/);
  assert.match(tools, /"preview_inspect"/);
  assert.match(tools, /"preview_click"/);
  assert.match(tools, /http:\/\/127\.0\.0\.1:/);
  assert.match(connector, /"preview_action"/);
  assert.match(adapter, /preview_action/);
  assert.match(ui, /Preview:/);
});
