const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('web shell exposes auth and application gates with PWA bootstrap', () => {
  const html = fs.readFileSync(path.join(root, 'web', 'index.html'), 'utf8');
  assert.match(html, /id="authGate"/);
  assert.match(html, /id="appGate"/);
  assert.match(html, /rel="manifest" href="manifest\.webmanifest"/);
  assert.match(html, /type="module" src="js\/app\.js"/);
});

test('public Supabase configuration rejects placeholder values', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'js', 'config.js'), 'utf8');
  assert.match(source, /export function getSupabaseConfig/);
  assert.match(source, /ตั้งค่า Supabase URL และ anon key ก่อนเริ่มใช้งาน/);
});
