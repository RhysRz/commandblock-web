const assert = require('node:assert/strict'); const fs = require('node:fs'); const path = require('node:path'); const test = require('node:test');
test('cloud proxy requires a user session and never persists provider keys', () => {
  const source = fs.readFileSync(path.join(__dirname, '..', 'supabase', 'functions', 'chat', 'index.ts'), 'utf8');
  assert.match(source, /auth\.getUser\(token\)/); assert.match(source, /baseUrl !== 'https:\/\/api\.deepseek\.com'/); assert.doesNotMatch(source, /insert\([^)]*apiKey|console\.log\([^)]*apiKey/);
});
test('Pages deploy publishes the generated site without local configuration', () => {
  const source = fs.readFileSync(path.join(__dirname, '..', '.github', 'workflows', 'deploy-pages.yml'), 'utf8');
  assert.match(source, /node scripts\/build-web\.mjs/);
  assert.match(source, /path: site/);
  assert.doesNotMatch(source, /config\.json/);
});
