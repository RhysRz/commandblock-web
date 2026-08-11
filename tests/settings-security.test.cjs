const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
test('cloud key settings are never written to browser storage', () => {
  const source = fs.readFileSync(path.join(__dirname, '..', 'web', 'js', 'settings.js'), 'utf8');
  assert.doesNotMatch(source, /localStorage|sessionStorage|indexedDB/);
  assert.match(source, /apiKey: ''/);
});
