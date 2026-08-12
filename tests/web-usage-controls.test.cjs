const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('web cloud chat returns provider usage and shows a credit control', () => {
  const adapter = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');
  const ui = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
  const functionCode = fs.readFileSync(path.join(__dirname, '..', 'supabase', 'functions', 'chat', 'index.ts'), 'utf8');
  assert.match(adapter, /data\.usage/);
  assert.match(ui, /usageCredit/);
  assert.match(ui, /platform\.deepseek\.com\/top_up/);
  assert.match(functionCode, /usage:\s*data\.usage/);
});
