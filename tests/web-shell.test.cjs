const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('Pages deployment builds and uploads the canonical Commandblock UI', () => {
  const workflow = fs.readFileSync(path.join(root, '.github', 'workflows', 'deploy-pages.yml'), 'utf8');
  assert.match(workflow, /node scripts\/build-web\.mjs/);
  assert.match(workflow, /path: site/);
  assert.doesNotMatch(workflow, /path: web/);
});

test('public Supabase configuration rejects placeholder values', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'js', 'config.js'), 'utf8');
  assert.match(source, /export function getSupabaseConfig/);
  assert.match(source, /ตั้งค่า Supabase URL และ anon key ก่อนเริ่มใช้งาน/);
});
