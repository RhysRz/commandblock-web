const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('desktop API exposes explicit SESSION and pin endpoints', () => {
  const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');
  const cloud = fs.readFileSync(path.join(root, 'src', 'cloud.rs'), 'utf8');

  assert.match(gui, /\("GET", "\/api\/conversations"\)/);
  assert.match(gui, /\("POST", "\/api\/conversations"\)/);
  assert.match(gui, /\/api\/messages\//);
  assert.match(cloud, /pub fn list_conversations/);
  assert.match(cloud, /pub fn create_conversation/);
  assert.match(cloud, /pub fn set_message_pin/);
});

test('cloud reads pin state and scopes selected conversation requests', () => {
  const cloud = fs.readFileSync(path.join(root, 'src', 'cloud.rs'), 'utf8');

  assert.match(cloud, /pub is_pinned: bool/);
  assert.match(cloud, /select=id,role,content,created_at,is_pinned/);
  assert.match(cloud, /conversation_id=eq\.\{conv_id\}.*user_id=eq/s);
  assert.match(cloud, /is_pinned/);
});
