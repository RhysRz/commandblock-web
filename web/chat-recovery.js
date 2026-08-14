(function attachChatRecovery(root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  if (root) root.CommandBlockChatRecovery = api;
}(typeof window === 'undefined' ? globalThis : window, function createChatRecovery() {
  const PREFIX = 'commandblock.chat-recovery.v1';

  function scopedKey(kind, userId) {
    return `${PREFIX}.${kind}.${encodeURIComponent(String(userId || ''))}`;
  }

  function saveJson(storage, key, value) {
    try {
      storage.setItem(key, JSON.stringify(value));
      return true;
    } catch (_) {
      return false;
    }
  }

  function loadJson(storage, key) {
    try {
      const raw = storage.getItem(key);
      return raw ? JSON.parse(raw) : null;
    } catch (_) {
      return null;
    }
  }

  function saveConversationId(storage, userId, conversationId) {
    if (!conversationId) return false;
    return saveJson(storage, scopedKey('conversation', userId), { conversationId: String(conversationId) });
  }

  function loadConversationId(storage, userId) {
    const saved = loadJson(storage, scopedKey('conversation', userId));
    return typeof saved?.conversationId === 'string' && saved.conversationId ? saved.conversationId : null;
  }

  function saveRunState(storage, userId, state) {
    if (!state?.conversationId || !Array.isArray(state.messages)) return false;
    return saveJson(storage, scopedKey('run', userId), {
      conversationId: String(state.conversationId),
      messages: state.messages,
    });
  }

  function loadRunState(storage, userId) {
    const saved = loadJson(storage, scopedKey('run', userId));
    if (!saved || typeof saved.conversationId !== 'string' || !Array.isArray(saved.messages)) return null;
    return { conversationId: saved.conversationId, messages: saved.messages };
  }

  function clearRunState(storage, userId) {
    try { storage.removeItem(scopedKey('run', userId)); } catch (_) { /* storage unavailable */ }
  }

  function isContinuationRequest(message) {
    const text = String(message || '').trim().toLowerCase();
    return /^(ต่อ|ทำต่อ|ดำเนินการต่อ|ทำต่อจากจุดที่บันทึกไว้|continue|resume)(?:\s|$)/.test(text);
  }

  return { saveConversationId, loadConversationId, saveRunState, loadRunState, clearRunState, isContinuationRequest };
}));
