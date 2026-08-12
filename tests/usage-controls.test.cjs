const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('desktop UI exposes local DeepSeek credit, top-up, and token usage controls', () => {
  const ui = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
  assert.match(ui, /id="usageCredit"/);
  assert.match(ui, /id="usageTopUp"/);
  assert.match(ui, /id="usageTokens"/);
  assert.match(ui, /https:\/\/platform\.deepseek\.com\/top_up/);
  assert.match(ui, /ev === "usage"/);
  assert.match(ui, /commandblock-usage-v1/);
});
