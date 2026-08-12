import { createClient } from 'https://esm.sh/@supabase/supabase-js@2';

const cors = { 'access-control-allow-origin': Deno.env.get('ALLOWED_ORIGIN') ?? '*', 'access-control-allow-headers': 'authorization, content-type', 'content-type': 'application/json' };
const reply = (body: Record<string, unknown>, status = 200) => new Response(JSON.stringify(body), { status, headers: cors });

Deno.serve(async (request) => {
  if (request.method === 'OPTIONS') return new Response(null, { headers: cors });
  if (request.method !== 'POST') return reply({ error: 'ใช้ POST เท่านั้น' }, 405);
  const token = request.headers.get('authorization')?.replace(/^Bearer\s+/i, '');
  if (!token) return reply({ error: 'กรุณาเข้าสู่ระบบก่อนส่งข้อความ' }, 401);
  const supabaseUrl = Deno.env.get('SUPABASE_URL')!;
  const anonKey = Deno.env.get('SUPABASE_ANON_KEY')!;
  const auth = createClient(supabaseUrl, anonKey);
  const { data: { user } } = await auth.auth.getUser(token);
  if (!user) return reply({ error: 'Session หมดอายุ กรุณาเข้าสู่ระบบใหม่' }, 401);
  const { model, baseUrl, apiKey, messages, conversationId } = await request.json();
  if (!apiKey || !Array.isArray(messages) || messages.length < 1 || messages.length > 16) return reply({ error: 'กรอก API key และข้อความให้ถูกต้อง' }, 400);
  const validMessages = messages.every((message) => message && ['user', 'assistant'].includes(message.role) && typeof message.content === 'string' && message.content.length > 0 && message.content.length <= 32000);
  if (!validMessages) return reply({ error: 'รูปแบบบทสนทนาไม่ถูกต้อง' }, 400);
  if (baseUrl !== 'https://api.deepseek.com') return reply({ error: 'ผู้ให้บริการนี้ยังไม่อนุญาต' }, 400);
  const upstream = await fetch(`${baseUrl}/chat/completions`, { method: 'POST', headers: { authorization: `Bearer ${apiKey}`, 'content-type': 'application/json' }, body: JSON.stringify({ model, messages }) });
  if (!upstream.ok) return reply({ error: 'โมเดลตอบกลับไม่สำเร็จ โปรดตรวจ API key และเครดิต' }, upstream.status);
  const data = await upstream.json();
  const usage = data.usage ? { ...data.usage, exact: true } : null;
  const owner = createClient(supabaseUrl, anonKey, { global: { headers: { Authorization: `Bearer ${token}` } } });
  if (usage) await owner.from('usage_events').insert({
    user_id: user.id, conversation_id: conversationId || null, model_id: String(model || 'deepseek-v4-flash'),
    prompt_tokens: Number(usage.prompt_tokens || 0), completion_tokens: Number(usage.completion_tokens || 0), total_tokens: Number(usage.total_tokens || 0), exact: true,
  });
  return reply({
    content: data.choices?.[0]?.message?.content ?? '',
    usage,
  });
});
