(() => {
  'use strict';

  const STORAGE_KEY = 'commandblock.preview-tabs.v1';
  const empty = () => ({ tabs: [], activeId: '' });
  const copy = (state) => ({ tabs: state.tabs.map((tab) => ({ ...tab })), activeId: state.activeId });

  function labelFor(url) {
    try { return new URL(url).hostname || 'New tab'; } catch { return 'New tab'; }
  }

  function kindFor(url) {
    return /^http:\/\/127\.0\.0\.1(?::\d+)?\//.test(url) ? 'local' : 'https';
  }

  function validTab(tab) {
    const blank = tab && tab.url === '' && tab.title === 'New tab' && tab.kind === 'blank';
    const navigable = tab && typeof tab.url === 'string' && /^https?:\/\//.test(tab.url)
      && (tab.kind === 'local' || tab.kind === 'https');
    return tab && typeof tab.id === 'string' && typeof tab.title === 'string' && (blank || navigable);
  }

  function restore(storage) {
    try {
      const value = JSON.parse(storage?.getItem(STORAGE_KEY) || '');
      if (!value || !Array.isArray(value.tabs) || !value.tabs.every(validTab)) return empty();
      const state = { tabs: value.tabs.map((tab) => ({ ...tab })), activeId: String(value.activeId || '') };
      if (!state.tabs.some((tab) => tab.id === state.activeId)) state.activeId = state.tabs[0]?.id || '';
      return state;
    } catch (_) {
      return empty();
    }
  }

  function persist(storage, state) {
    try { storage?.setItem(STORAGE_KEY, JSON.stringify(state)); } catch (_) { /* session storage is optional */ }
  }

  function nextId(state) {
    const candidate = typeof globalThis.crypto?.randomUUID === 'function' ? globalThis.crypto.randomUUID() : `preview-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    return state.tabs.some((tab) => tab.id === candidate) ? `preview-${Date.now()}-${Math.random().toString(36).slice(2)}` : candidate;
  }

  function add(state, url) {
    const next = copy(state || empty());
    const value = String(url || '').trim();
    if (!/^https?:\/\//.test(value)) return next;
    const existing = next.tabs.find((tab) => tab.url === value);
    if (existing) { next.activeId = existing.id; return next; }
    const tab = { id: nextId(next), url: value, title: labelFor(value), kind: kindFor(value) };
    next.tabs.push(tab);
    next.activeId = tab.id;
    return next;
  }

  function blank(state) {
    const next = copy(state || empty());
    const tab = { id: nextId(next), url: '', title: 'New tab', kind: 'blank' };
    next.tabs.push(tab);
    next.activeId = tab.id;
    return next;
  }

  function select(state, id) {
    const next = copy(state || empty());
    if (next.tabs.some((tab) => tab.id === id)) next.activeId = id;
    return next;
  }

  function close(state, id) {
    const next = copy(state || empty());
    const index = next.tabs.findIndex((tab) => tab.id === id);
    if (index < 0) return next;
    const wasActive = next.activeId === id;
    next.tabs.splice(index, 1);
    if (wasActive) next.activeId = next.tabs[index - 1]?.id || next.tabs[index]?.id || '';
    return next;
  }

  window.CommandBlockPreviewTabs = { STORAGE_KEY, restore, persist, add, blank, select, close, labelFor };
})();
