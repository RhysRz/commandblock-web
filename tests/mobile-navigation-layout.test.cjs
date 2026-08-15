const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');

test('mobile uses floating history, menu, and settings controls without a bottom rail', () => {
  assert.match(html, /id="mobileHistoryToggle"/);
  assert.match(html, /id="mobileMenuToggle"/);
  assert.match(html, /id="mobileSettingsToggle"/);
  assert.match(html, /function setMobileMenuDrawer\(open\)/);
  assert.match(html, /const histPane = document\.getElementById\("histpane"\)/);
  assert.match(html, /rightPane\.style\.transform = visible \? "translateX\(0\)" : "translateX\(105%\)"/);
  assert.match(html, /mobileSettingsToggle\.addEventListener\("click", openSettings\)/);
  assert.match(html, /#iconrail\s*\{\s*display:\s*none/);
  assert.match(html, /body\.mobmenu #rightpane\s*\{\s*transform:\s*translateX\(0\)/);
});
