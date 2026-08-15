const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');
const html = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');
const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');

test('tool narration is moved into the expandable work strip', () => {
  assert.match(gui, /fn tools_begin\(&mut self\)\s*\{[\s\S]*sse\(self\.out, "tools_begin", json!\(\{\}\)\)/);
  assert.match(html, /function moveNarrationToWorkStrip\(bub, narration\)/);
  assert.match(html, /ev==="tools_begin"/);
  assert.match(html, /className="work-narration"/);
});
