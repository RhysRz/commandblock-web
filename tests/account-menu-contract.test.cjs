const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const ui = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');
const gui = fs.readFileSync(path.join(__dirname, '..', 'src', 'gui.rs'), 'utf8');
const auth = fs.readFileSync(path.join(__dirname, '..', 'src', 'auth.rs'), 'utf8');
const cloudAdapter = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');

test('signed-in account chip opens an upward account management menu', () => {
  assert.match(ui, /<button class="accchip" id="accChip" type="button"/);
  assert.match(ui, /id="accountMenu"/);
  assert.match(ui, /id="accountManageBtn"/);
  assert.match(ui, /id="accountDevicesBtn"/);
  assert.match(ui, /id="accountUsageBtn"/);
  assert.match(ui, /function setAccountMenuOpen\(open\)/);
  assert.match(ui, /account-menu/);
});

test('account menu sends password recovery and confirms sign out in-app', () => {
  assert.match(ui, /id="accountResetPasswordBtn"/);
  assert.match(ui, /id="accountSignOutBtn"/);
  assert.match(ui, /fetch\("\/api\/auth\/recover"/);
  assert.match(ui, /await confirmAction\([\s\S]*ออกจากระบบ/);
  assert.match(gui, /\("POST", "\/api\/auth\/recover"\)/);
  assert.match(auth, /pub fn send_password_recovery\(/);
  assert.match(auth, /\/auth\/v1\/recover/);
  assert.match(cloudAdapter, /async function authRecover\(\)/);
  assert.match(cloudAdapter, /resetPasswordForEmail\(cloudUser\.email/);
  assert.match(cloudAdapter, /path === '\/api\/auth\/recover'/);
});
