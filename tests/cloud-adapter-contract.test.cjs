const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('cloud adapter keeps API keys in session storage and sends chat through Supabase', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');

  assert.match(source, /sessionStorage/);
  assert.match(source, /functions\/v1\/chat/);
  assert.match(source, /Desktop Connector/);
  assert.match(source, /event\('content'/);
  assert.match(source, /@media \(max-width: 760px\)/);
  assert.doesNotMatch(source, /localStorage\.setItem\([^)]*(api|key)/i);
});
