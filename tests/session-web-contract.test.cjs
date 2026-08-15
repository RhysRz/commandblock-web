const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('browser adapter creates and selects explicit sessions instead of always selecting the newest', () => {
  const adapter = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');

  assert.match(adapter, /async function listConversations\(session\)/);
  assert.match(adapter, /async function createConversation\(session\)/);
  assert.match(adapter, /async function selectConversation\(session, id\)/);
  assert.match(adapter, /async function toggleMessagePin\(session, id, isPinned\)/);
  assert.match(adapter, /select\('id,title,model_id,created_at,updated_at'\)/);
  assert.match(adapter, /select\('id,role,content,created_at,is_pinned'\)/);
});

test('shared UI labels the panel SESSION and provides new-session and context pin controls', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(ui, />SESSION\s*</);
  assert.match(ui, /id="newSession"/);
  assert.match(ui, /id="messageContextMenu"/);
  assert.match(ui, /contextmenu/);
  assert.match(ui, /Pin message/);
  assert.match(ui, /\/assets\/session-store\.js/);
});

test('shared UI opens an Obsidian context popup for messages and SESSION deletion', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');
  const adapter = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');

  assert.match(ui, /id="deleteSession"/);
  assert.match(ui, /function openContextMenu/);
  assert.match(ui, /function deleteSession/);
  assert.match(ui, /background:\s*#120b20/);
  assert.match(ui, /kind:\s*["']session["']/);
  assert.match(adapter, /async function deleteConversation/);
});
