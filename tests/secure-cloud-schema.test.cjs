const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('secure Cloud suite schema has owner usage and Remote approval fields', () => {
  const migration = fs.readFileSync(path.join(__dirname, '..', 'supabase', 'migrations', '202608120005_secure_cloud_suite.sql'), 'utf8');
  assert.match(migration, /create table public\.usage_events/i);
  assert.match(migration, /approval_code_hash/i);
  assert.match(migration, /approval_code_input/i);
  assert.match(migration, /enable row level security/i);
  assert.match(migration, /usage_events_user_day_idx/i);
});
