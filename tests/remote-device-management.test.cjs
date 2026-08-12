const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('web exposes owner-scoped device management actions', () => {
  const adapter = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');
  assert.match(adapter, /id = 'cb-devices-open'/);
  assert.match(adapter, /cb-device-rename/);
  assert.match(adapter, /cb-device-revoke/);
  assert.match(adapter, /device_audit_events/);
  assert.match(adapter, /\.eq\('user_id', session\.user\.id\)/);
});
