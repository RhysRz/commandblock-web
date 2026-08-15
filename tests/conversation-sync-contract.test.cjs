const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');
const cloud = fs.readFileSync(path.join(root, 'src', 'cloud.rs'), 'utf8');
const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');
const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');
const adapter = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');

test('desktop and cloud clients select the latest account conversation and poll the shared transcript', () => {
  assert.match(cloud, /fn latest_conversation\(/);
  assert.doesNotMatch(cloud, /a\.delete\(&del_url\)/);
  assert.match(gui, /"\/api\/conversation\/sync"/);
  assert.match(ui, /function startConversationSync\(\)/);
  assert.match(adapter, /async function activeConversationForUser\(/);
  assert.match(adapter, /path === '\/api\/conversation\/sync'/);
});
