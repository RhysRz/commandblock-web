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

test('web-only settings hide unavailable skills and expose a real logout action', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');

  assert.match(source, /#settingsModal \.set-sec:nth-of-type\(2\)\{display:none\}/);
  assert.match(source, /logout\.id = 'cb-cloud-logout'/);
  assert.match(source, /client\.auth\.signOut\(\)/);
  assert.match(source, /cb-auth-pending/);
  assert.match(source, /document\.querySelector\('\.statusbar'\)\?\.appendChild\(logout\)/);
  assert.match(source, /#modelPill\{max-width:none!important;overflow:visible!important\}/);
});

test('web adapter uses a selected connector device for local pane operations', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');

  assert.match(source, /async function requestConnector\(action, payload\)/);
  assert.match(source, /from\('connector_commands'\)/);
  assert.match(source, /commandblock\.active-device-id/);
  assert.match(source, /path === '\/api\/files'.*connectorResult\('files'/s);
  assert.match(source, /path === '\/api\/exec'.*connectorResult\('exec'/s);
});
