const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('Remote approval uses a local Credential Manager secret and never exposes it to the web', () => {
  const remote = fs.readFileSync(path.join(__dirname, '..', 'src', 'remote.rs'), 'utf8');
  const web = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');
  assert.match(remote, /keyring::Entry/);
  assert.match(remote, /approval_code_hash/);
  assert.match(remote, /approval_code_input/);
  assert.match(web, /cb-remote-pin/);
  assert.doesNotMatch(web, /device_secret/);
});

test('Remote PC explains when a network blocks direct P2P connectivity', () => {
  const web = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');
  assert.match(web, /เครือข่ายนี้อาจบล็อก P2P/);
  assert.match(web, /TURN relay/);
});
