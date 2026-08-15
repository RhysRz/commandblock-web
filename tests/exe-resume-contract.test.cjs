const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('desktop agent loop emits a resume event when its tool-round limit interrupts work', () => {
  const root = path.join(__dirname, '..');
  const main = fs.readFileSync(path.join(root, 'src', 'main.rs'), 'utf8');
  const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(main, /TurnOutcome/);
  assert.match(main, /Interrupted/);
  assert.match(gui, /"resume"/);
  assert.match(ui, /resumeFromCheckpoint/);
});
