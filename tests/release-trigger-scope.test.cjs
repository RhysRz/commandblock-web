const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('Windows releases are triggered only by runtime source changes', () => {
  const deploy = fs.readFileSync(path.join(__dirname, '..', '.github', 'workflows', 'deploy-pages.yml'), 'utf8');
  const releasePath = path.join(__dirname, '..', '.github', 'workflows', 'release-windows.yml');
  assert.doesNotMatch(deploy, /release-windows:/);
  assert.ok(fs.existsSync(releasePath), 'Windows release must have a dedicated workflow');
  const release = fs.readFileSync(releasePath, 'utf8');
  assert.match(release, /paths:/);
  assert.match(release, /'src\/\*\*'/);
  assert.match(release, /'Cargo\.toml'/);
  assert.match(release, /commandblock\.exe --build-id/);
  assert.match(release, /gh release view \$tag/);
  assert.match(release, /gh release create \$tag/);
  assert.match(release, /gh release create/);
});
