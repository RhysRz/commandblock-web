const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');

function sessions() {
  const source = fs.readFileSync(path.join(__dirname, '..', 'web', 'session-store.js'), 'utf8');
  const sandbox = {};
  sandbox.globalThis = sandbox;
  vm.runInNewContext(source, sandbox, { filename: 'session-store.js' });
  return sandbox.CommandBlockSessions;
}

test('sessions order newest update first with id as a stable tie breaker', () => {
  const api = sessions();
  const rows = [
    { id: 'b', updated_at: '2026-08-15T10:01:00.000Z' },
    { id: 'a', updated_at: '2026-08-15T10:01:00.000Z' },
    { id: 'c', updated_at: '2026-08-15T10:02:00.000Z' },
  ];

  rows.sort(api.compareSessions);
  assert.deepEqual(rows.map((row) => row.id), ['c', 'b', 'a']);
});

test('pinned sessions sort before more recently updated unpinned sessions', () => {
  const api = sessions();
  const rows = [
    { id: 'latest', updated_at: '2026-08-16T10:03:00.000Z', is_pinned: false },
    { id: 'pinned', updated_at: '2026-08-16T10:01:00.000Z', is_pinned: true },
  ];

  rows.sort(api.compareSessions);
  assert.deepEqual(rows.map((row) => row.id), ['pinned', 'latest']);
});

test('pinning returns a new row and keeps the original row unchanged', () => {
  const api = sessions();
  const before = { id: 'm1', is_pinned: false };

  assert.deepEqual(JSON.parse(JSON.stringify(api.togglePinned(before))), { id: 'm1', is_pinned: true });
  assert.equal(before.is_pinned, false);
});

test('message ordering stays chronological with stable ids on tied timestamps', () => {
  const api = sessions();
  const rows = api.sortMessages([
    { id: 'b', created_at: '2026-08-15T10:00:00.000Z' },
    { id: 'a', created_at: '2026-08-15T10:00:00.000Z' },
    { id: 'c', created_at: '2026-08-15T10:01:00.000Z' },
  ]);

  assert.deepEqual(Array.from(rows, (row) => row.id), ['a', 'b', 'c']);
});
