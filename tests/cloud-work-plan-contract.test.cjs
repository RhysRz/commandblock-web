const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const source = fs.readFileSync(path.join(__dirname, '..', 'web', 'cloud-adapter.js'), 'utf8');

test('cloud agent exposes the same update_plan tool as the desktop agent', () => {
  assert.match(source, /name:\s*'update_plan'/);
  assert.match(source, /บันทึกแผนงานเป็นข้อความลำดับขั้นเพื่อแสดง Todo ให้ผู้ใช้/);
  assert.match(source, /if \(name === 'update_plan'\) return \{ ok: true, plan: args\.plan \|\| '' \}/);
  assert.match(source, /อัปเดต Todo เมื่อเริ่มงานและเมื่อขั้นตอนเสร็จ/);
});
