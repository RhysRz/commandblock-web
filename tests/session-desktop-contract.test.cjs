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

test('desktop API deletes only the selected owner-scoped SESSION', () => {
  const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');
  const cloud = fs.readFileSync(path.join(root, 'src', 'cloud.rs'), 'utf8');

  assert.match(gui, /path\.starts_with\("\/api\/conversations\/"\).*ends_with\("\/delete"\)/s);
  assert.match(cloud, /pub fn delete_conversation/);
  assert.match(cloud, /a\.delete\(&url\)/);
  assert.match(cloud, /conversations\?id=eq\.\{conversation_id\}&user_id=eq/);
});

test('desktop API persists a pin state for the selected owner-scoped SESSION', () => {
  const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');
  const cloud = fs.readFileSync(path.join(root, 'src', 'cloud.rs'), 'utf8');

  assert.match(gui, /path\.starts_with\("\/api\/conversations\/"\).*ends_with\("\/pin"\)/s);
  assert.match(cloud, /pub fn set_conversation_pin/);
  assert.match(cloud, /json!\(\{"is_pinned": is_pinned\}\)/);
  assert.match(cloud, /select=id,title,model_id,is_pinned,created_at,updated_at/);
});

test('cloud reads pin state and scopes selected conversation requests', () => {
  const cloud = fs.readFileSync(path.join(root, 'src', 'cloud.rs'), 'utf8');

  assert.match(cloud, /pub is_pinned: bool/);
  assert.match(cloud, /select=id,role,content,created_at,is_pinned/);
  assert.match(cloud, /conversation_id=eq\.\{conv_id\}.*user_id=eq/s);
  assert.match(cloud, /is_pinned/);
});
