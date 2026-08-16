const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');

const root = path.join(__dirname, '..');

function loadTabs() {
  const source = fs.readFileSync(path.join(root, 'web', 'preview-tabs.js'), 'utf8');
  const sandbox = { window: {}, URL, crypto: { randomUUID: () => 'tab-test' } };
  vm.runInNewContext(source, sandbox, { filename: 'preview-tabs.js' });
  return sandbox.window.CommandBlockPreviewTabs;
}

test('adds a selected tab with a hostname title', () => {
  const tabs = loadTabs();
  const next = tabs.add({ tabs: [], activeId: '' }, 'https://example.com/docs');

  assert.equal(next.tabs.length, 1);
  assert.equal(next.tabs[0].title, 'example.com');
  assert.equal(next.tabs[0].kind, 'https');
  assert.equal(next.activeId, 'tab-test');
});

test('creates a selected blank tab for the plus button', () => {
  const tabs = loadTabs();
  const next = tabs.blank({ tabs: [], activeId: '' });

  assert.equal(next.tabs.length, 1);
  assert.equal(next.tabs[0].title, 'New tab');
  assert.equal(next.tabs[0].url, '');
  assert.equal(next.tabs[0].kind, 'blank');
  assert.equal(next.activeId, 'tab-test');
});

test('selects and closes tabs without losing the adjacent active tab', () => {
  const tabs = loadTabs();
  const first = tabs.add({ tabs: [], activeId: '' }, 'https://one.example');
  const second = tabs.add(first, 'http://127.0.0.1:5000/index.html');
  const selected = tabs.select(second, first.tabs[0].id);
  const closed = tabs.close(selected, first.tabs[0].id);

  assert.equal(closed.tabs.length, 1);
  assert.equal(closed.tabs[0].url, 'http://127.0.0.1:5000/index.html');
  assert.equal(closed.activeId, closed.tabs[0].id);
  assert.equal(closed.tabs[0].kind, 'local');
});

test('restores only valid persisted preview tab state', () => {
  const tabs = loadTabs();
  const valid = {
    getItem: () => JSON.stringify({ tabs: [{ id: 'a', url: 'https://example.com', title: 'Example', kind: 'https' }], activeId: 'a' }),
  };
  const malformed = { getItem: () => '{not json' };

  assert.equal(JSON.stringify(tabs.restore(valid)), JSON.stringify({ tabs: [{ id: 'a', url: 'https://example.com', title: 'Example', kind: 'https' }], activeId: 'a' }));
  assert.equal(JSON.stringify(tabs.restore(malformed)), JSON.stringify({ tabs: [], activeId: '' }));
});
