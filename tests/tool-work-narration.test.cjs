const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');
const html = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');
const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');

test('tool boundary preserves AI text while work strip remains a separate active area', () => {
  assert.match(gui, /fn tools_begin\(&mut self\)\s*\{[\s\S]*sse\(self\.out, "tools_begin", json!\(\{\}\)\)/);
  assert.match(html, /rememberConversationMessage\("assistant", acc, textSegment\.holder\)/);
  assert.match(html, /const work=ensureWorkSegment\(\);/);
  assert.doesNotMatch(html, /function moveNarrationToWorkStrip/);
});
