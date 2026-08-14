const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('Cloud chat keeps bounded context and saves an interrupted agent checkpoint', () => {
  const adapter = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');
  const fn = fs.readFileSync(path.join(__dirname, '..', 'supabase', 'functions', 'chat', 'index.ts'), 'utf8');
  assert.match(adapter, /conversationMessages/);
  assert.match(adapter, /saveRunState/);
  assert.match(adapter, /loadRunState/);
  assert.match(adapter, /saveMessage\(session, 'assistant', lastContent\)/);
  assert.match(fn, /usage_events/);
  assert.match(fn, /messages\.length/);
  assert.doesNotMatch(fn, /usage_events[\s\S]{0,300}apiKey/);
});
