import { createClient } from 'https://esm.sh/@supabase/supabase-js@2';

const cors = { 'access-control-allow-origin': Deno.env.get('ALLOWED_ORIGIN') ?? '*', 'access-control-allow-headers': 'authorization, content-type', 'content-type': 'application/json' };
const reply = (body: Record<string, unknown>, status = 200) => new Response(JSON.stringify(body), { status, headers: cors });

Deno.serve(async (request) => {
  if (request.method === 'OPTIONS') return new Response(null, { headers: cors });
  if (request.method !== 'POST') return reply({ error: 'ใช้ POST เท่านั้น' }, 405);
  const token = request.headers.get('authorization')?.replace(/^Bearer\s+/i, '');
  if (!token) return reply({ error: 'กรุณาเข้าสู่ระบบก่อนส่งข้อความ' }, 401);
  const auth = createClient(Deno.env.get('SUPABASE_URL')!, Deno.env.get('SUPABASE_ANON_KEY')!);
  const { data: { user } } = await auth.auth.getUser(token);
  if (!user) return reply({ error: 'Session หมดอายุ กรุณาเข้าสู่ระบบใหม่' }, 401);
  const { model, baseUrl, apiKey, message } = await request.json();
  if (!apiKey || !message || typeof message !== 'string' || message.length > 32000) return reply({ error: 'กรอก API key และข้อความให้ถูกต้อง' }, 400);
  if (baseUrl !== 'https://api.deepseek.com') return reply({ error: 'ผู้ให้บริการนี้ยังไม่อนุญาต' }, 400);
  const upstream = await fetch(`${baseUrl}/chat/completions`, { method: 'POST', headers: { authorization: `Bearer ${apiKey}`, 'content-type': 'application/json' }, body: JSON.stringify({ model, messages: [{ role: 'user', content: message }] }) });
  if (!upstream.ok) return reply({ error: 'โมเดลตอบกลับไม่สำเร็จ โปรดตรวจ API key และเครดิต' }, upstream.status);
  const data = await upstream.json();
  return reply({
    content: data.choices?.[0]?.message?.content ?? '',
    usage: data.usage ? { ...data.usage, exact: true } : null,
  });
});
