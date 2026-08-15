const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('pinned message tray stays sticky and navigates to its source message', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(ui, /#pinnedMessages\s*\{[^}]*position:\s*sticky/s);
  assert.match(ui, /function scrollToPinnedMessage\(messageId\)/);
  assert.match(ui, /item\.addEventListener\("click".*scrollToPinnedMessage/s);
  assert.match(ui, /message\.classList\.add\("pinned-focus"\)/);
});

test('SESSION header keeps New session aligned to the right', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(ui, /\.hist-head\s*\{[^}]*display:\s*flex/s);
  assert.match(ui, /\.hist-head \.session-new\s*\{[^}]*margin-left:\s*auto/s);
});
