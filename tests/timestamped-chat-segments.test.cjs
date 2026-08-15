const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');

function timeline() {
  const source = fs.readFileSync(path.join(__dirname, '..', 'web', 'chat-timeline.js'), 'utf8');
  const sandbox = {};
  sandbox.globalThis = sandbox;
  vm.runInNewContext(source, sandbox, { filename: 'chat-timeline.js' });
  return sandbox.CommandBlockTimeline;
}

test('conversation rows insert by database timestamp even when an older row arrives late', () => {
  const api = timeline();
  const existing = [
    { id: 'assistant-later', role: 'assistant', content: 'สรุปหลัง', created_at: '2026-08-15T11:03:00.000Z' },
  ];
  const older = { id: 'assistant-earlier', role: 'assistant', content: 'สรุปก่อน', created_at: '2026-08-15T11:02:00.000Z' };

  assert.equal(api.insertionIndex(existing, older), 0);
});

test('conversation ordering keeps timestamp ties deterministic by message id', () => {
  const api = timeline();
  const rows = [
    { id: 'b', role: 'assistant', content: 'สอง', created_at: '2026-08-15T11:02:00.000Z' },
    { id: 'a', role: 'assistant', content: 'หนึ่ง', created_at: '2026-08-15T11:02:00.000Z' },
  ];

  rows.sort(api.compareRows);
  assert.deepEqual(rows.map((row) => row.id), ['a', 'b']);
});

test('a tool boundary finalizes visible text before the active work area starts', () => {
  const api = timeline();
  const turn = api.createLiveTurn();

  turn.appendText('## ตรวจสอบไฟล์\nเรียบร้อย');
  const boundary = turn.beginTools();
  assert.equal(boundary.content, '## ตรวจสอบไฟล์\nเรียบร้อย');
  assert.equal(boundary.workActive, true);
  assert.equal(turn.activeText(), '');
  turn.appendText('## สรุปผล\nแก้แล้ว');
  assert.equal(turn.finishText(), '## สรุปผล\nแก้แล้ว');
});

test('desktop sync keeps the database row identity and timestamp for chronological rendering', () => {
  const gui = fs.readFileSync(path.join(__dirname, '..', 'src', 'gui.rs'), 'utf8');
  assert.match(gui, /json!\(\{"id": message\.id, "role": message\.role, "content": message\.content, "created_at": message\.created_at\}\)/);
});

test('shared UI loads the timeline helper and uses it for synchronized rows', () => {
  const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
  assert.match(html, /<script src="\/assets\/chat-timeline\.js"><\/script>/);
  assert.match(html, /function stampConversationMessage\(el, row\)[\s\S]*chatTimeline\.insertionIndex/);
});

test('cloud adapter starts a work boundary and persists each assistant phase independently', () => {
  const adapter = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');
  assert.match(adapter, /push\('tools_begin', \{\}\);/);
  assert.match(adapter, /if \(data\.content\.trim\(\)\) await saveMessage\(session, 'assistant', data\.content\);/);
  assert.doesNotMatch(adapter, /if \(lastContent\) await saveMessage\(session, 'assistant', lastContent\);/);
});
