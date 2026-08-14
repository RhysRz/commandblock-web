const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('cloud adapter keeps API keys in the browser and streams agent chat to DeepSeek', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');

  assert.match(source, /sessionStorage/);
  assert.match(source, /https:\/\/api\.deepseek\.com\/chat\/completions/);
  assert.match(source, /stream:\s*true/);
  assert.match(source, /Desktop Connector/);
  assert.match(source, /push\(name, payload\)/);
  assert.match(source, /CommandBlockChatRecovery/);
});

test('web adapter exposes a real logout action and keeps account controls scoped to the session', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');

  assert.match(source, /async function authLogout\(\)/);
  assert.match(source, /auth\.signOut\(\)/);
  assert.match(source, /cb-auth-pending/);
  assert.match(source, /commandblock\.active-device-id/);
});

test('web adapter uses a selected connector device for local pane operations', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');

  assert.match(source, /async function requestConnector\(action, payload\)/);
  assert.match(source, /from\('connector_commands'\)/);
  assert.match(source, /commandblock\.active-device-id/);
  assert.match(source, /path === '\/api\/files'.*connectorResult\('files'/s);
  assert.match(source, /path === '\/api\/exec'.*connectorResult\('exec'/s);
});
