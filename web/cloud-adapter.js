(() => {
  'use strict';

  const SUPABASE_URL = 'https://qympivgklmstrnhfaywn.supabase.co';
  const SUPABASE_PUBLISHABLE_KEY = 'sb_publishable_UJMuyL3QY8lMEWJKZi3zAQ_NFKZY8TH';
  const MODEL = 'deepseek-v4-flash';
  const KEY_NAME = 'commandblock.deepseek.api-key';
  const NOTES_NAME = 'commandblock.cloud-notes';
  const ACTIVE_DEVICE_NAME = 'commandblock.active-device-id';
  const originalFetch = window.fetch.bind(window);
  const client = window.supabase?.createClient(SUPABASE_URL, SUPABASE_PUBLISHABLE_KEY);
  let conversationId = null;
  document.documentElement.classList.add('cb-auth-pending');
  const authLockStyle = document.createElement('style');
  authLockStyle.textContent = 'html.cb-auth-pending body > :not(#cb-cloud-gate){visibility:hidden!important}';
  document.head.appendChild(authLockStyle);

  const json = (body, status = 200) => new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
  const sse = (events, status = 200) => new Response(events.join(''), {
    status,
    headers: { 'content-type': 'text/event-stream; charset=utf-8' },
  });
  const cloudState = () => ({
    version: 'web', backend: 'cloud', model: MODEL, base_url: 'https://api.deepseek.com',
    folder: 'Cloud session', folder_path: '', preview_url: '', skill_count: 0, session_messages: 0,
  });
  const connectorMessage = 'ฟีเจอร์นี้ต้องเชื่อมต่อ Desktop Connector เพื่อเข้าถึงไฟล์และเครื่องของคุณ';

  function apiPath(input) {
    const value = typeof input === 'string' ? input : input?.url;
    try { return new URL(value, location.origin).pathname; } catch { return ''; }
  }
  function requestBody(init) {
    try { return JSON.parse(init?.body || '{}'); } catch { return {}; }
  }
  function event(name, payload) {
    return `event: ${name}\ndata: ${JSON.stringify(payload)}\n\n`;
  }
  function sessionKey() { return sessionStorage.getItem(KEY_NAME) || ''; }
  function askForKey() {
    const current = sessionKey();
    const entered = window.prompt('ใส่ DeepSeek API key สำหรับ session นี้ (เก็บเฉพาะแท็บเบราว์เซอร์)\nเช่น sk-... ', current);
    if (entered?.trim()) sessionStorage.setItem(KEY_NAME, entered.trim());
    return sessionKey();
  }
  async function currentSession() {
    if (!client) throw new Error('ไม่สามารถโหลด Supabase ได้ กรุณารีเฟรชหน้าเว็บ');
    const { data } = await client.auth.getSession();
    if (!data.session) throw new Error('กรุณาเข้าสู่ระบบก่อนส่งข้อความ');
    return data.session;
  }
  async function ensureConversation(session, message) {
    if (conversationId) return conversationId;
    const title = (message || 'แชทใหม่').trim().slice(0, 80) || 'แชทใหม่';
    const { data, error } = await client.from('conversations')
      .insert({ user_id: session.user.id, title, model_id: MODEL }).select('id').single();
    if (error) throw new Error('สร้างประวัติสนทนาไม่สำเร็จ');
    conversationId = data.id;
    return conversationId;
  }
  async function saveMessage(session, role, content) {
    const id = await ensureConversation(session, content);
    const { error } = await client.from('messages')
      .insert({ conversation_id: id, user_id: session.user.id, role, content });
    if (error) throw new Error('บันทึกประวัติสนทนาไม่สำเร็จ');
    await client.from('conversations').update({ updated_at: new Date().toISOString() }).eq('id', id);
  }
  async function cloudChat(init) {
    try {
      const session = await currentSession();
      const { message } = requestBody(init);
      if (!message?.trim()) return sse([event('note', { t: 'กรุณาพิมพ์ข้อความก่อนส่ง' })], 400);
      const apiKey = sessionKey() || askForKey();
      if (!apiKey) return sse([event('note', { t: 'ต้องใส่ DeepSeek API key ก่อนใช้งาน Cloud chat' })], 400);
      await saveMessage(session, 'user', message);
      const response = await originalFetch(`${SUPABASE_URL}/functions/v1/chat`, {
        method: 'POST',
        headers: { authorization: `Bearer ${session.access_token}`, 'content-type': 'application/json' },
        body: JSON.stringify({ model: MODEL, baseUrl: 'https://api.deepseek.com', apiKey, message }),
      });
      const data = await response.json();
      if (!response.ok || data.error) throw new Error(data.error || 'Cloud chat ไม่สำเร็จ');
      const content = String(data.content || '');
      await saveMessage(session, 'assistant', content);
      return sse([event('content', { t: content })]);
    } catch (error) {
      return sse([event('note', { t: error.message || 'ไม่สามารถเชื่อมต่อ Cloud chat ได้' })], 400);
    }
  }
  async function cloudHistory() {
    try {
      const session = await currentSession();
      const { data: rows, error } = await client.from('messages')
        .select('content, created_at').eq('user_id', session.user.id).eq('role', 'user')
        .order('created_at', { ascending: false }).limit(24);
      if (error) throw error;
      return json({ prompts: (rows || []).map((row) => row.content) });
    } catch { return json({ prompts: [] }); }
  }
  async function cloudNotes(init) {
    if ((init?.method || 'GET').toUpperCase() === 'POST') {
      const { notes = '' } = requestBody(init);
      sessionStorage.setItem(NOTES_NAME, String(notes));
      return json({ saved: true, scope: 'session' });
    }
    return json({ notes: sessionStorage.getItem(NOTES_NAME) || '' });
  }
  async function activeConnector(session) {
    const staleBefore = new Date(Date.now() - 45_000).toISOString();
    const preferred = sessionStorage.getItem(ACTIVE_DEVICE_NAME);
    if (preferred) {
      const { data } = await client.from('connector_devices').select('id,name,root_name,last_seen_at')
        .eq('id', preferred).gte('last_seen_at', staleBefore).maybeSingle();
      if (data) return data;
      sessionStorage.removeItem(ACTIVE_DEVICE_NAME);
    }
    const { data, error } = await client.from('connector_devices').select('id,name,root_name,last_seen_at')
      .eq('user_id', session.user.id).gte('last_seen_at', staleBefore)
      .order('last_seen_at', { ascending: false }).limit(1).maybeSingle();
    if (error || !data) throw new Error('ไม่พบ Desktop Connector ที่ออนไลน์อยู่ — เปิด Commandblock.exe --connector บนเครื่องก่อน');
    sessionStorage.setItem(ACTIVE_DEVICE_NAME, data.id);
    return data;
  }
  async function requestConnector(action, payload) {
    const session = await currentSession();
    const device = await activeConnector(session);
    const { data: command, error: insertError } = await client.from('connector_commands')
      .insert({ user_id: session.user.id, device_id: device.id, action, payload }).select('id').single();
    if (insertError || !command) throw new Error('ส่งคำสั่งไปยัง Desktop Connector ไม่สำเร็จ');
    for (let attempt = 0; attempt < 24; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 750));
      const { data, error } = await client.from('connector_commands').select('status,result,error')
        .eq('id', command.id).maybeSingle();
      if (error || !data) throw new Error('ไม่สามารถอ่านผลจาก Desktop Connector ได้');
      if (data.status === 'completed') return data.result || {};
      if (data.status === 'rejected') throw new Error(data.error || 'คำสั่งถูกปฏิเสธบนเครื่อง Desktop Connector');
      if (data.status === 'failed') throw new Error(data.error || 'Desktop Connector ทำงานไม่สำเร็จ');
    }
    throw new Error('Desktop Connector ยังไม่ตอบกลับภายใน 18 วินาที');
  }
  async function connectorResult(action, payload, fallback) {
    try { return json(await requestConnector(action, payload)); }
    catch (error) { return json(fallback(error.message || connectorMessage), 503); }
  }
  function unsupported(path) {
    const activity = [`⚠️ ${connectorMessage}`];
    if (path === '/api/files') return json({ files: [], requires_connector: true, message: connectorMessage });
    if (path === '/api/changes') return json({ changes: [], requires_connector: true, message: connectorMessage });
    if (path === '/api/queue') return json({ activity, requires_connector: true, message: connectorMessage });
    if (path === '/api/exec') return json({ output: connectorMessage, requires_connector: true });
    if (path === '/api/read') return json({ content: connectorMessage, requires_connector: true });
    if (path === '/api/pick-folder') return json({ ok: false, requires_connector: true, message: connectorMessage });
    if (path === '/api/settings') return json({ startup_script: '', skills: [], available_skills: [], path: connectorMessage, requires_connector: true });
    if (path === '/api/startup-log') return json({ log: [], requires_connector: true });
    return json({ ok: false, requires_connector: true, message: connectorMessage });
  }

  window.fetch = async (input, init = {}) => {
    const path = apiPath(input);
    if (path === '/api/state') return json(cloudState());
    if (path === '/api/models') return json({ models: [{ name: MODEL, base_url: 'https://api.deepseek.com', source: 'cloud', active: true }] });
    if (path === '/api/model') return json({ ok: true, backend: 'cloud', model: MODEL, base_url: 'https://api.deepseek.com' });
    if (path === '/api/chat') return cloudChat(init);
    if (path === '/api/history') return cloudHistory();
    if (path === '/api/notes') return cloudNotes(init);
    if (path === '/api/files') return connectorResult('files', {}, (message) => ({ files: [], requires_connector: true, message }));
    if (path === '/api/changes') return connectorResult('changes', {}, (message) => ({ changes: [], requires_connector: true, message }));
    if (path === '/api/queue') return connectorResult('queue', {}, (message) => ({ activity: [`⚠️ ${message}`], requires_connector: true }));
    if (path === '/api/read') {
      const requested = new URL(typeof input === 'string' ? input : input.url, location.origin).searchParams.get('path') || '';
      return connectorResult('read', { path: requested }, (message) => ({ content: message, requires_connector: true }));
    }
    if (path === '/api/pick-folder') return connectorResult('pick_folder', {}, (message) => ({ ok: false, requires_connector: true, message }));
    if (path === '/api/exec') return connectorResult('exec', requestBody(init), (message) => ({ output: message, requires_connector: true }));
    if (['/api/settings', '/api/startup-log'].includes(path)) return unsupported(path);
    return originalFetch(input, init);
  };

  function mountAuthGate() {
    const style = document.createElement('style');
    style.textContent = `#cb-cloud-gate{position:fixed;inset:0;z-index:9999;display:grid;place-items:center;padding:24px;background:radial-gradient(circle at 50% 0,#251044 0,#0d0918 58%,#07060d 100%);color:#f5efff;font-family:"Segoe UI","Noto Sans Thai",sans-serif}#cb-cloud-gate[hidden]{display:none}.cb-cloud-card{width:min(440px,100%);padding:32px;border:1px solid rgba(184,137,255,.35);border-radius:24px;background:rgba(23,14,43,.84);box-shadow:0 28px 80px rgba(0,0,0,.48);backdrop-filter:blur(18px)}.cb-cloud-card h1{margin:0 0 10px;font-size:28px}.cb-cloud-card p{color:#cdbfe8;line-height:1.6}.cb-cloud-card input{width:100%;margin-top:10px;padding:13px 14px;border:1px solid #5b3e84;border-radius:12px;background:#100a20;color:#fff;font:inherit}.cb-cloud-actions{display:grid;gap:10px;margin-top:18px}.cb-cloud-actions button{padding:12px;border:1px solid #7344ba;border-radius:12px;background:#271343;color:#fff;font:inherit;font-weight:700;cursor:pointer}.cb-cloud-actions button.primary{border:0;background:linear-gradient(135deg,#7034df,#a65cff)}.cb-cloud-status{min-height:24px;margin-top:14px;color:#bfeadf;font-size:13px}.cb-cloud-link{background:none!important;border:0!important;color:#c69aff!important;text-decoration:underline;font-weight:400!important}`;
    style.textContent += `#settingsModal .set-sec:nth-of-type(2){display:none}#statsRight{display:none}#modelPill{max-width:none!important;overflow:visible!important}#cb-cloud-logout{margin-left:auto;flex-shrink:0;padding:3px 10px}@media (max-width: 760px){body{grid-template-columns:44px minmax(0,1fr);height:100dvh}#histpane,#rightpane{display:none}#chatpane{min-width:0}.chat-head{gap:7px;padding:10px}.chat-title .sub{display:none}.chat-title h1{font-size:15px}.pill#folderBtn{display:none}.chat-foot{padding:8px}.statusbar{font-size:10px}.feedback{display:none}.inputbox{min-height:46px}.logo img{width:34px;height:34px}.wrap{padding:14px 10px}.bubble{max-width:96%}}`;
    document.head.appendChild(style);
    const gate = document.createElement('section');
    gate.id = 'cb-cloud-gate';
    gate.innerHTML = `<div class="cb-cloud-card"><h1>Commandblock Web</h1><p>เข้าสู่ระบบเพื่อใช้ Commandblock เดิมบนเว็บ พร้อม Cloud chat ของคุณ</p><form id="cb-cloud-form"><input id="cb-cloud-name" placeholder="ชื่อที่แสดง (เฉพาะตอนสมัคร)" autocomplete="name"><input id="cb-cloud-email" type="email" placeholder="อีเมล" autocomplete="email" required><input id="cb-cloud-password" type="password" placeholder="รหัสผ่านอย่างน้อย 8 ตัว" autocomplete="current-password" minlength="8" required><div class="cb-cloud-actions"><button class="primary" type="submit">เข้าสู่ระบบ</button><button id="cb-cloud-register" type="button">สร้างบัญชี</button><button id="cb-cloud-reset" class="cb-cloud-link" type="button">ลืมรหัสผ่าน</button></div></form><div id="cb-cloud-status" class="cb-cloud-status" role="status"></div><p>API key ของ DeepSeek อยู่เฉพาะ session นี้ และไม่ถูกบันทึกในบัญชี</p></div>`;
    document.body.appendChild(gate);
    const status = gate.querySelector('#cb-cloud-status');
    const email = gate.querySelector('#cb-cloud-email');
    const password = gate.querySelector('#cb-cloud-password');
    const name = gate.querySelector('#cb-cloud-name');
    const report = (message, error = false) => { status.textContent = message; status.style.color = error ? '#ffc2d8' : '#bfeadf'; };
    const errorText = (error) => error?.message === 'Invalid login credentials' ? 'อีเมลหรือรหัสผ่านไม่ถูกต้อง' : (error?.message || 'เกิดข้อผิดพลาด กรุณาลองใหม่');
    const openApp = () => { gate.hidden = true; document.documentElement.classList.remove('cb-auth-pending'); };
    const logout = document.createElement('button');
    logout.id = 'cb-cloud-logout';
    logout.className = 'pill';
    logout.type = 'button';
    logout.textContent = 'Log out';
    document.querySelector('.statusbar')?.appendChild(logout);
    logout.addEventListener('click', async () => {
      const { error } = await client.auth.signOut();
      if (error) { window.alert(errorText(error)); return; }
      sessionStorage.removeItem(KEY_NAME);
      conversationId = null;
      gate.hidden = false;
      document.documentElement.classList.add('cb-auth-pending');
      password.value = '';
      report('ออกจากระบบแล้ว');
    });
    if (!client) { report('โหลดระบบเข้าสู่ระบบไม่สำเร็จ กรุณารีเฟรชหน้าเว็บ', true); return; }
    client.auth.getSession().then(({ data }) => { if (data.session) openApp(); });
    gate.querySelector('#cb-cloud-form').addEventListener('submit', async (event) => {
      event.preventDefault(); report('กำลังเข้าสู่ระบบ…');
      const { data, error } = await client.auth.signInWithPassword({ email: email.value.trim(), password: password.value });
      if (error) { report(errorText(error), true); return; }
      if (data.session) location.reload();
    });
    gate.querySelector('#cb-cloud-register').addEventListener('click', async () => {
      report('กำลังสร้างบัญชี…');
      const { error } = await client.auth.signUp({ email: email.value.trim(), password: password.value, options: { data: { full_name: name.value.trim() }, emailRedirectTo: location.href } });
      report(error ? errorText(error) : 'ส่งอีเมลยืนยันแล้ว เปิดลิงก์ในอีเมลก่อนเข้าสู่ระบบ', Boolean(error));
    });
    gate.querySelector('#cb-cloud-reset').addEventListener('click', async () => {
      if (!email.value.trim()) { report('กรอกอีเมลก่อนกดลืมรหัสผ่าน', true); return; }
      const { error } = await client.auth.resetPasswordForEmail(email.value.trim(), { redirectTo: location.href });
      report(error ? errorText(error) : 'ส่งลิงก์ตั้งรหัสผ่านใหม่แล้ว', Boolean(error));
    });
  }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', mountAuthGate, { once: true });
  else mountAuthGate();
})();
