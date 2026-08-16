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
