const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('web build publishes the canonical CommandBlock UI and inserts the adapter first', () => {
  execFileSync(process.execPath, ['scripts/build-web.mjs'], { cwd: root });

  const output = fs.readFileSync(path.join(root, 'site', 'index.html'), 'utf8');
  assert.match(output, /id="iconrail"/);
  assert.match(output, /id="chat"/);
  assert.match(output, /CommandBlock — ผู้ช่วยพัฒนาโค้ด AI/);
  assert.match(output, /src="\.\/cloud-adapter\.js"/);
  assert.ok(output.indexOf('cloud-adapter.js') < output.indexOf('<script>'));
  assert.match(output, /src="\.\/assets\/buff-command-block\.png"/);
  assert.match(output, /src="\.\/assets\/preview-tabs\.js"/);
  assert.ok(fs.existsSync(path.join(root, 'site', 'assets', 'buff-command-block.png')));
  assert.ok(fs.existsSync(path.join(root, 'site', 'assets', 'preview-tabs.js')));
});
