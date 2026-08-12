const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('Windows release package includes every runtime sidecar required by the app', () => {
  const workflow = fs.readFileSync(path.join(__dirname, '..', '.github', 'workflows', 'deploy-pages.yml'), 'utf8');
  assert.match(workflow, /target\\release\\commandblock\.exe/);
  assert.match(workflow, /target\\release\\commandblock-connector\.exe/);
  assert.match(workflow, /target\\release\\commandblock-updater\.exe/);
});
