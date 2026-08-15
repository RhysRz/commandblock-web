const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('plugin catalog opens from the left rail and labels provider states honestly', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(ui, /id="pluginsBtn"/);
  assert.match(ui, /id="pluginCatalog"/);
  assert.match(ui, /const PLUGIN_CATALOG = \[/);
  assert.match(ui, /connect-required/);
  assert.match(ui, /function renderPluginCatalog\(query\)/);
  assert.match(ui, /id="pluginSearch"/);
  assert.match(ui, /Built in/);
  assert.match(ui, /Connect required/);
});

test('plugin catalog lists broadly useful provider categories without a fake install API', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  for (const category of ['Development', 'Productivity', 'Storage', 'Communication', 'Design', 'Hosting', 'Billing']) {
    assert.match(ui, new RegExp(`category:"${category}"`));
  }
  assert.doesNotMatch(ui, /fetch\([^\n]*plugin/i);
});

test('plugin catalog includes every currently advertised connector provider', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  for (const provider of ['Base44', 'Codex Security', 'Google Calendar', 'Granola', 'HeyGen', 'HyperFrames', 'Lovable', 'Neon Postgres', 'OpenAI Developers', 'Outlook Calendar', 'Semrush', 'Superpowers']) {
    assert.match(ui, new RegExp(`name:"${provider}"`));
  }
});
