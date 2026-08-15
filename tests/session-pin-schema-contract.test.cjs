const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('SESSION pins are durable, owner-scoped, and indexed for pinned-first ordering', () => {
  const sql = fs.readFileSync(path.join(__dirname, '..', 'supabase', 'migrations', '202608160001_session_pins.sql'), 'utf8');
  assert.match(sql, /alter table public\.conversations\s+add column if not exists is_pinned boolean not null default false/i);
  assert.match(sql, /conversations_user_pin_updated_idx/i);
  assert.match(sql, /user_id, is_pinned desc, updated_at desc, id desc/i);
});
