const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const test = require('node:test');

test('release workflow generates a Thai summary for the updater', () => {
  const root = path.join(__dirname, '..');
  const helper = fs.readFileSync(path.join(root, 'tools', 'render-release-notes.ps1'), 'utf8');
  const workflow = fs.readFileSync(path.join(root, '.github', 'workflows', 'release-windows.yml'), 'utf8');
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(helper, /function Convert-CommitSubjectToThai/);
  assert.match(helper, /หน้าต่างยืนยันการทำรายการ/);
  assert.match(helper, /ปรับปรุงความเสถียรและประสิทธิภาพของ CommandBlock/);
  assert.match(workflow, /render-release-notes\.ps1/);
  assert.match(workflow, /--notes-file release-notes\.md/);
  assert.match(ui, /id="updateNotesSummary"/);
  assert.match(ui, /function splitReleaseNotes/);
});

test('release-summary subjects become a specific Thai update bullet', () => {
  const root = path.join(__dirname, '..');
  const command = "& '.\\tools\\render-release-notes.ps1' -Subject @('feat(updater): add Thai release summaries')";
  const result = spawnSync('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', command], { cwd: root, encoding: 'utf8' });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /- เพิ่มสรุปการอัปเดตภาษาไทย/);
});
