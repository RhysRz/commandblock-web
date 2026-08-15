const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('session and pin migration keeps pins durable and owner-scoped', () => {
  const migration = path.join(__dirname, '..', 'supabase', 'migrations', '202608150001_sessions_and_pins.sql');
  const sql = fs.readFileSync(migration, 'utf8');

  assert.match(sql, /alter table public\.messages\s+add column if not exists is_pinned boolean not null default false/i);
  assert.match(sql, /messages_conversation_pin_created_idx/i);
  assert.match(sql, /auth\.uid\(\).*user_id/i);
});
