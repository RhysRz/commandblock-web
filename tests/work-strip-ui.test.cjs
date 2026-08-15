const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');

test('canonical UI renders an expandable work strip and moves mobile status into the tool drawer', () => {
  assert.match(html, /className="workstrip"/);
  assert.match(html, /id="mobileStatusDrawer"/);
  assert.match(html, /function renderWorkStrip\(bub\)/);
  assert.match(html, /function syncMobileStatusPlacement\(\)/);
  assert.match(html, /rightPane\.insertBefore\(mobileStatusDrawer,\s*rightPane\.firstChild\)/);
  assert.match(html, /\.workstrip summary/);
});
