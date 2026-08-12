const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('desktop reports when a configured fallback model responds', () => {
  const main = fs.readFileSync(path.join(__dirname, '..', 'src', 'main.rs'), 'utf8');
  assert.match(main, /ใช้ fallback model/);
  assert.match(main, /config::fallback_models/);
});
