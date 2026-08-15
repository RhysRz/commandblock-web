const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.join(__dirname, '..');

test('preview capability is allowlisted to CommandBlock local preview and recorded in work activity', () => {
  const tools = fs.readFileSync(path.join(root, 'src', 'tools.rs'), 'utf8');
  const connector = fs.readFileSync(path.join(root, 'src', 'connector.rs'), 'utf8');
  const adapter = fs.readFileSync(path.join(root, 'web', 'cloud-adapter.js'), 'utf8');
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(tools, /"preview_open"/);
  assert.match(tools, /"preview_inspect"/);
  assert.match(tools, /"preview_click"/);
  assert.match(tools, /http:\/\/127\.0\.0\.1:/);
  assert.match(connector, /"preview_action"/);
  assert.match(adapter, /preview_action/);
  assert.match(ui, /Preview:/);
});

test('Preview tools keep local previews in CommandBlock and retain an explicit browser escape hatch', () => {
  const tools = fs.readFileSync(path.join(root, 'src', 'tools.rs'), 'utf8');
  const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');
  const previewTools = tools.slice(
    tools.indexOf('pub fn reopen_preview()'),
    tools.indexOf('/// รันเซิร์ฟเวอร์ static')
  );

  assert.doesNotMatch(previewTools, /open_browser\(&url\)/);
  assert.match(previewTools, /เปิดแท็บ Preview ใน CommandBlock/);
  assert.match(ui, /obj\.name === "open_preview" \|\| String\(obj\.name\|\|""\)\.startsWith\("preview_"\)/);
  assert.match(gui, /"preview_ready"/);
  assert.match(ui, /ev === "preview_ready"/);
  assert.match(ui, /window\.open\(state\.preview_url,"_blank"\)/);
});

test('desktop right workspace has a persistent accessible resize handle', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(ui, /id="rightPaneResizer"/);
  assert.match(ui, /--rightpane-width/);
  assert.match(ui, /commandblock\.rightPaneWidth/);
  assert.match(ui, /Math\.max\(240, Math\.min\(720,/);
  assert.match(ui, /@media \(max-width:\s*900px\)[\s\S]*#rightPaneResizer/);
});
