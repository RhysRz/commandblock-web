const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');

function runAutoGrow(scrollHeight) {
  const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
  const match = html.match(/function autoGrow\(\)\{[^}]+\}/);
  assert.ok(match, 'autoGrow function must exist');
  const input = { scrollHeight, style: {} };
  const context = { input };
  vm.runInNewContext(`${match[0]}; autoGrow();`, context);
  return input.style;
}

test('shows a vertical scrollbar when a message exceeds the composer limit', () => {
  const style = runAutoGrow(360);
  assert.equal(style.height, '120px');
  assert.equal(style.overflowY, 'auto');
});

test('keeps the composer scrollbar hidden for a short message', () => {
  const style = runAutoGrow(48);
  assert.equal(style.height, '48px');
  assert.equal(style.overflowY, 'hidden');
});
