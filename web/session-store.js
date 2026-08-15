(() => {
  function compareText(a, b) {
    return String(a || '').localeCompare(String(b || ''));
  }

  function compareSessions(a, b) {
    const pin = Number(Boolean(b.is_pinned)) - Number(Boolean(a.is_pinned));
    if (pin) return pin;
    const updated = compareText(b.updated_at, a.updated_at);
    return updated || compareText(b.id, a.id);
  }

  function compareMessages(a, b) {
    const created = compareText(a.created_at, b.created_at);
    return created || compareText(a.id, b.id);
  }

  function sortMessages(rows) {
    return [...(rows || [])].sort(compareMessages);
  }

  function togglePinned(row) {
    return { ...row, is_pinned: !row.is_pinned };
  }

  globalThis.CommandBlockSessions = { compareSessions, sortMessages, togglePinned };
})();
