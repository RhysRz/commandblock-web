const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('destructive actions use the in-app confirmation dialog', () => {
  const source = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');

  assert.match(source, /function confirmAction\(options\)/);
  assert.match(source, /id="confirmModal"/);
  assert.match(source, /id="confirmCancel"/);
  assert.match(source, /id="confirmApprove"/);
  assert.doesNotMatch(source, /\b(?:window\.)?confirm\(/);
  assert.match(source, /await confirmAction\([\s\S]*ลบ SESSION/);
  assert.match(source, /await confirmAction\([\s\S]*กู้คืน/);
  assert.match(source, /await confirmAction\([\s\S]*ออกจากระบบ/);
});
