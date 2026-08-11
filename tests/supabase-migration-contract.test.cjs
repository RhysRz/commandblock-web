const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('migration defines user-owned data with row level security', () => {
  const sql = fs.readFileSync(path.join(__dirname, '..', 'supabase', 'migrations', '202608120001_commandblock_web.sql'), 'utf8');
  for (const table of ['profiles', 'conversations', 'messages']) {
    assert.match(sql, new RegExp(`create table public\\.${table}`));
    assert.match(sql, new RegExp(`alter table public\\.${table} enable row level security`));
  }
  assert.match(sql, /with check \(\(select auth\.uid\(\)\) = user_id\)/);
});
