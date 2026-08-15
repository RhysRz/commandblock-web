const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');

test('Thinking uses a closed, throttled, tail-only renderer', () => {
  assert.match(html, /const THINK_RENDER_DELAY_MS\s*=\s*200/);
  assert.match(html, /const THINK_VISIBLE_CHAR_LIMIT\s*=\s*3000/);
  assert.match(html, /function scheduleThinkingRender\(bub, fullText\)/);
  assert.match(html, /setTimeout\(flushThinkingRender, THINK_RENDER_DELAY_MS\)/);
  assert.match(html, /th\.open=false/);
  assert.match(html, /fullText\.slice\(-THINK_VISIBLE_CHAR_LIMIT\)/);
});
