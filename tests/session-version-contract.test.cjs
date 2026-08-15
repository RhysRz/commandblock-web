const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('the checkpoint and preview update increments the desktop package version', () => {
  const cargo = fs.readFileSync(path.join(__dirname, '..', 'Cargo.toml'), 'utf8');
  assert.match(cargo, /^version = "1\.0\.2"$/m);
});
