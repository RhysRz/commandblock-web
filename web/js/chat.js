export async function createConversation(client, userId, model) {
  const { data, error } = await client.from('conversations').insert({ user_id: userId, model_id: model }).select().single();
  if (error) throw error; return data;
}
export async function appendMessage(client, conversationId, userId, role, content) {
  const { error } = await client.from('messages').insert({ conversation_id: conversationId, user_id: userId, role, content });
  if (error) throw error;
}
export async function sendCloudMessage(client, session, settings, message) {
  const { data, error } = await client.functions.invoke('chat', { headers: { Authorization: `Bearer ${session.access_token}` }, body: { ...settings, message } });
  if (error) throw error; if (data.error) throw new Error(data.error); return data.content;
}
