const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');

function cssRule(selector) {
  const match = html.match(new RegExp(`${selector}\\s*\\{([^}]*)\\}`));
  assert.ok(match, `${selector} rule must exist`);
  return match[1];
}

test('chat pane can shrink so long conversations scroll inside the viewport', () => {
  assert.match(cssRule('#chatpane'), /min-height:\s*0/);
  const chat = cssRule('#chat');
  assert.match(chat, /min-height:\s*0/);
  assert.match(chat, /overflow-y:\s*auto/);
});
