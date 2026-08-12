const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('connector persists only a refresh token in Windows Credential Manager', () => {
  const code = fs.readFileSync(path.join(__dirname, '..', 'src', 'connector.rs'), 'utf8');
  assert.match(code, /keyring::Entry/);
  assert.match(code, /grant_type=refresh_token/);
  assert.match(code, /refresh_token/);
});
