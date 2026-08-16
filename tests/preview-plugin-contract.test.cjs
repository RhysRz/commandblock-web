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
  assert.match(ui, /window\.open\(url,"_blank","noopener"\)/);
});

test('the preview command accepts a published HTTPS URL without weakening local preview tools', () => {
  const tools = fs.readFileSync(path.join(root, 'src', 'tools.rs'), 'utf8');
  const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(tools, /pub fn set_https_preview_url/);
  assert.match(tools, /pub fn preview_command/);
  assert.match(tools, /Url::parse/);
  assert.match(gui, /"\/preview"\s*=>\s*tools::preview_command/);
  assert.match(ui, /const requestedPreview\s*=\s*\/\^\\\/preview\\b\/\.test\(text\)/);
  assert.match(tools, /http:\/\/127\.0\.0\.1:/);
});

test('Preview exposes browser-style tab controls backed by the shared tab-state helper', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');
  const gui = fs.readFileSync(path.join(root, 'src', 'gui.rs'), 'utf8');

  assert.match(ui, /id="previewTabs"/);
  assert.match(ui, /id="previewTabAdd"/);
  assert.match(ui, /id="previewUrlInput"/);
  assert.match(ui, /id="previewFrameFallback"/);
  assert.match(ui, /function renderPreviewTabs/);
  assert.match(ui, /CommandBlockPreviewTabs/);
  assert.match(ui, /\/assets\/preview-tabs\.js/);
  assert.match(gui, /const PREVIEW_TABS_JS: &str = include_str!\("\.\.\/web\/preview-tabs\.js"\)/);
  assert.match(gui, /\("GET", "\/assets\/preview-tabs\.js"\)/);
});

test('desktop right workspace has a persistent accessible resize handle', () => {
  const ui = fs.readFileSync(path.join(root, 'src', 'ui.html'), 'utf8');

  assert.match(ui, /id="rightPaneResizer"/);
  assert.match(ui, /--rightpane-width/);
  assert.match(ui, /commandblock\.rightPaneWidth/);
  assert.match(ui, /Math\.max\(240, Math\.min\(720,/);
  assert.match(ui, /@media \(max-width:\s*900px\)[\s\S]*#rightPaneResizer/);
});
