const assert = require('node:assert/strict');
const test = require('node:test');

const recovery = require('../web/chat-recovery.js');

function memoryStorage() {
  const values = new Map();
  return {
    getItem(key) { return values.get(key) ?? null; },
    setItem(key, value) { values.set(key, String(value)); },
    removeItem(key) { values.delete(key); },
  };
}

test('resume state restores the same user conversation and keeps completed tool evidence', () => {
  const storage = memoryStorage();
  recovery.saveRunState(storage, 'user-1', {
    conversationId: 'conv-123',
    messages: [
      { role: 'assistant', content: '', tool_calls: [{ id: 'call-1', type: 'function', function: { name: 'read_file', arguments: '{"path":"src/main.rs"}' } }] },
      { role: 'tool', tool_call_id: 'call-1', content: '{"content":"fn main() {}"}' },
    ],
  });

  assert.deepEqual(recovery.loadRunState(storage, 'user-1'), {
    conversationId: 'conv-123',
    messages: [
      { role: 'assistant', content: '', tool_calls: [{ id: 'call-1', type: 'function', function: { name: 'read_file', arguments: '{"path":"src/main.rs"}' } }] },
      { role: 'tool', tool_call_id: 'call-1', content: '{"content":"fn main() {}"}' },
    ],
  });
});

test('resume state is scoped to the active project and keeps interruption details', () => {
  const storage = memoryStorage();
  recovery.saveRunState(storage, 'user-1', {
    conversationId: 'conv-123', messages: [], projectKey: 'C:/demo',
    plan: '- [ ] build', savedAt: 100, reason: 'step_limit',
  });

  assert.equal(recovery.loadRunState(storage, 'user-1', 'C:/other'), null);
  assert.deepEqual(recovery.loadRunState(storage, 'user-1', 'C:/demo'), {
    conversationId: 'conv-123', messages: [], projectKey: 'C:/demo',
    plan: '- [ ] build', savedAt: 100, reason: 'step_limit',
  });
});

test('resume state is isolated per account and can be cleared after a completed run', () => {
  const storage = memoryStorage();
  recovery.saveRunState(storage, 'user-a', { conversationId: 'conv-a', messages: [{ role: 'user', content: 'แก้บั๊ก' }] });

  assert.equal(recovery.loadRunState(storage, 'user-b'), null);
  recovery.clearRunState(storage, 'user-a');
  assert.equal(recovery.loadRunState(storage, 'user-a'), null);
});

test('continuation phrases are recognized without treating a new task as a resume', () => {
  assert.equal(recovery.isContinuationRequest('ทำต่อ'), true);
  assert.equal(recovery.isContinuationRequest('continue from checkpoint'), true);
  assert.equal(recovery.isContinuationRequest('สร้างหน้าเว็บใหม่'), false);
});
