const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('connector relay schema scopes devices and commands to their owning user', () => {
  const sql = fs.readFileSync(path.join(root, 'supabase', 'migrations', '202608120002_connector.sql'), 'utf8');

  assert.match(sql, /create table public\.connector_devices/i);
  assert.match(sql, /create table public\.connector_commands/i);
  assert.match(sql, /device_id uuid not null references public\.connector_devices/i);
  assert.match(sql, /alter table public\.connector_devices enable row level security/i);
  assert.match(sql, /alter table public\.connector_commands enable row level security/i);
  assert.match(sql, /auth\.uid\(\)\) = user_id/i);
  assert.match(sql, /status in \('queued', 'running', 'completed', 'rejected', 'failed'\)/i);
  assert.match(sql, /where status = 'queued'/i);
});
