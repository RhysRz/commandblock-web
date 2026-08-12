const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('web provides profile, global logout, and daily/monthly owner usage dashboard', () => {
  const adapter = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');
  assert.match(adapter, /mountAccount/);
  assert.match(adapter, /signOut\(\{ scope: 'global' \}\)/);
  assert.match(adapter, /from\('profiles'\)/);
  assert.match(adapter, /from\('usage_events'\)/);
  assert.match(adapter, /cb-account-open/);
});
