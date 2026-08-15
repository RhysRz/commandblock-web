const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');

test('mobile exposes a native model dropdown, binds settings after its function, and disables viewport zoom', () => {
  assert.match(html, /maximum-scale=1, user-scalable=no/);
  assert.match(html, /id="mobileModelSelect"/);
  assert.match(html, /function modelLabel\(m\)\{ return \(m\.source \|\| "config"\) \+ " · " \+ m\.name; \}/);
  assert.match(html, /function renderMobileModelSelect\(list\)/);
  assert.match(html, /mobileModelSelect\.addEventListener\("change"/);
  assert.match(html, /@media \(max-width: 900px\)[\s\S]*#mobileModelSelect\s*\{[\s\S]*display:\s*block/);
  assert.match(html, /--mobile-top-clearance: calc\(env\(safe-area-inset-top, 0px\) \+ 34px\)/);
  assert.match(html, /#chatpane\s*\{[^}]*padding-top: var\(--mobile-top-clearance\);/);
  assert.ok(
    html.indexOf('function openSettings()') < html.lastIndexOf('mobileSettingsToggle.addEventListener("click", openSettings)'),
    'mobile settings handler must be registered after openSettings is available',
  );
});
