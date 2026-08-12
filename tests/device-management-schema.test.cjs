const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('device management migration keeps audit rows owner-scoped', () => {
  const sql = fs.readFileSync(path.join(__dirname, '..', 'supabase', 'migrations', '202608120004_device_management.sql'), 'utf8');
  assert.match(sql, /create table public\.device_audit_events/);
  assert.match(sql, /alter table public\.device_audit_events enable row level security/);
  assert.match(sql, /\(select auth\.uid\(\)\) = user_id/);
  assert.match(sql, /device_audit_events_owner_created_idx/);
});
