const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('remote sessions are owner-scoped, expiring, and limited to view or control', () => {
  const sql = fs.readFileSync(path.join(__dirname, '..', 'supabase', 'migrations', '202608120003_remote_sessions.sql'), 'utf8');
  assert.match(sql, /create table public\.remote_devices/i);
  assert.match(sql, /create table public\.remote_sessions/i);
  assert.match(sql, /mode text not null check \(mode in \('view', 'control'\)\)/i);
  assert.match(sql, /status text not null.*'requested'.*'connected'.*'closed'/is);
  assert.match(sql, /expires_at timestamptz not null/i);
  assert.match(sql, /alter table public\.remote_devices enable row level security/i);
  assert.match(sql, /alter table public\.remote_sessions enable row level security/i);
  assert.match(sql, /auth\.uid\(\)\) = user_id/i);
  assert.match(sql, /where status in \('requested', 'accepted', 'connected'\)/i);
});
