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
    return id;
  }
  async function conversationMessages(session, id) {
    const { data, error } = await client.from('messages')
      .select('role,content,created_at').eq('user_id', session.user.id).eq('conversation_id', id)
      .order('created_at', { ascending: false }).limit(16);
    if (error) throw new Error('โหลดบริบทสนทนาไม่สำเร็จ');
    return (data || []).reverse().map((row) => ({ role: row.role, content: row.content }));
  }
  async function cloudChat(init) {
    try {
      const session = await currentSession();
      const { message } = requestBody(init);
      if (!message?.trim()) return sse([event('note', { t: 'กรุณาพิมพ์ข้อความก่อนส่ง' })], 400);
      const apiKey = sessionKey() || askForKey();
      if (!apiKey) return sse([event('note', { t: 'ต้องใส่ DeepSeek API key ก่อนใช้งาน Cloud chat' })], 400);
      const id = await saveMessage(session, 'user', message);
      const messages = await conversationMessages(session, id);
      const response = await originalFetch(`${SUPABASE_URL}/functions/v1/chat`, {
        method: 'POST',
        headers: { authorization: `Bearer ${session.access_token}`, 'content-type': 'application/json' },
        body: JSON.stringify({ model: MODEL, baseUrl: 'https://api.deepseek.com', apiKey, messages, conversationId: id }),
      });
      const data = await response.json();
      if (!response.ok || data.error) throw new Error(data.error || 'Cloud chat ไม่สำเร็จ');
      const content = String(data.content || '');
      await saveMessage(session, 'assistant', content);
      const usage = data.usage && typeof data.usage === 'object' ? data.usage : {
        prompt_tokens: Math.ceil(message.length / 4), completion_tokens: Math.ceil(content.length / 4), total_tokens: Math.ceil((message.length + content.length) / 4), exact: false,
      };
      return sse([event('content', { t: content }), event('usage', usage)]);
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

  function waitForIceComplete(peer, timeout = 12000) {
    if (peer.iceGatheringState === 'complete') return Promise.resolve();
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => { peer.removeEventListener('icegatheringstatechange', done); reject(new Error('เตรียมการเชื่อมต่อ P2P นานเกินไป')); }, timeout);
      const done = () => { if (peer.iceGatheringState === 'complete') { clearTimeout(timer); peer.removeEventListener('icegatheringstatechange', done); resolve(); } };
      peer.addEventListener('icegatheringstatechange', done);
    });
  }
  function mountRemotePC() {
    if (document.querySelector('#cb-remote-open')) return;
    const style = document.createElement('style');
    style.textContent = `#cb-remote-open{margin-left:6px;flex-shrink:0;padding:3px 10px}.cb-remote-modal{position:fixed;inset:0;z-index:10002;display:grid;place-items:center;padding:20px;background:rgba(4,2,10,.76);backdrop-filter:blur(8px)}.cb-remote-modal[hidden]{display:none}.cb-remote-card{width:min(1100px,100%);max-height:calc(100dvh - 40px);overflow:auto;padding:20px;border:1px solid rgba(181,126,255,.4);border-radius:20px;background:#110a20;color:#f8f2ff;box-shadow:0 30px 90px #000}.cb-remote-card h2{margin:0 0 6px}.cb-remote-card p{color:#cbbce4;line-height:1.5}.cb-remote-row{display:flex;gap:10px;flex-wrap:wrap;align-items:center}.cb-remote-row button{min-height:40px;padding:9px 12px;border:1px solid #7042aa;border-radius:10px;background:#25143f;color:white;font:inherit;cursor:pointer}.cb-remote-row button.primary{border:0;background:linear-gradient(135deg,#7034df,#a65cff)}.cb-remote-row input{min-height:40px;min-width:180px;padding:9px 12px;border:1px solid #7042aa;border-radius:10px;background:#0c0716;color:white;font:inherit;letter-spacing:.15em}.cb-remote-actions{display:flex;gap:10px;flex-wrap:wrap;margin:12px 0}.cb-remote-actions button{min-height:44px;padding:10px 14px;border:1px solid #7042aa;border-radius:10px;background:#25143f;color:white;font:inherit;font-weight:700;cursor:pointer}.cb-remote-actions .primary{border:0;background:linear-gradient(135deg,#7034df,#a65cff)}#cb-remote-canvas{display:block;width:100%;max-height:68dvh;object-fit:contain;background:#050308;border:1px solid #41236d;border-radius:12px;cursor:crosshair;outline:none}.cb-remote-note{font-size:12px;color:#bda9de}.cb-remote-status{min-height:22px;color:#bfeadf}.cb-remote-device{padding:9px 12px;border-radius:10px;border:1px solid #513071;background:#171022;cursor:pointer}.cb-remote-device.selected{outline:2px solid #9e5cff;background:#2c1748}@media(max-width:760px){#cb-remote-open{width:100%;min-height:44px;justify-content:center;margin:4px 0}.statusbar{flex-wrap:wrap}.cb-remote-modal{padding:0;place-items:stretch}.cb-remote-card{width:100%;max-height:100dvh;padding:14px;border-radius:0}.cb-remote-row{display:flex;overflow-x:auto;flex-wrap:nowrap}.cb-remote-row button{flex:0 0 auto;min-height:44px}.cb-remote-actions{position:sticky;bottom:0;z-index:2;display:grid;grid-template-columns:1fr 1fr;background:#110a20;padding:10px 0;margin:8px 0}.cb-remote-actions button{width:100%;min-height:44px}.cb-remote-actions #cb-remote-disconnect{grid-column:1/-1}#cb-remote-canvas{max-height:52dvh;touch-action:none}.cb-remote-note{padding-bottom:12px}}`;
    document.head.appendChild(style);
    const open = document.createElement('button'); open.id = 'cb-remote-open'; open.className = 'pill'; open.type = 'button'; open.textContent = '🖥 Remote PC';
    document.querySelector('.statusbar')?.appendChild(open);
    const modal = document.createElement('section'); modal.className = 'cb-remote-modal'; modal.hidden = true;
    modal.innerHTML = `<div class="cb-remote-card" role="dialog" aria-modal="true" aria-label="Remote PC"><div class="cb-remote-row"><div><h2>Remote PC</h2><p>ภาพหน้าจอและการควบคุมส่งตรงระหว่างเบราว์เซอร์กับเครื่องของคุณผ่าน WebRTC</p></div><button id="cb-remote-close">ปิด</button></div><div id="cb-remote-devices" class="cb-remote-row"></div><div class="cb-remote-actions"><button id="cb-remote-view" class="primary">👁 ดูหน้าจอ</button><button id="cb-remote-control">🖱 ควบคุมเครื่อง</button><button id="cb-remote-disconnect">ตัดการเชื่อมต่อ</button></div><div id="cb-remote-pin-row" class="cb-remote-row" hidden><input id="cb-remote-pin" inputmode="numeric" pattern="[0-9]*" maxlength="6" autocomplete="one-time-code" placeholder="รหัส 6 หลักจากเครื่อง"><button id="cb-remote-submit-pin" class="primary">ยืนยันรหัส</button></div><div id="cb-remote-status" class="cb-remote-status"></div><canvas id="cb-remote-canvas" width="1280" height="720" tabindex="0"></canvas><p class="cb-remote-note">เครื่องปลายทางต้องเปิด <code>Commandblock.exe --remote</code> และกดยืนยันทุกครั้ง การเชื่อมต่อ P2P อาจใช้ไม่ได้ในบางเครือข่ายที่บล็อก UDP</p></div>`;
    document.body.appendChild(modal);
    const devices = modal.querySelector('#cb-remote-devices'); const status = modal.querySelector('#cb-remote-status'); const canvas = modal.querySelector('#cb-remote-canvas'); const context = canvas.getContext('2d'); const pinRow = modal.querySelector('#cb-remote-pin-row'); const pin = modal.querySelector('#cb-remote-pin'); const submitPin = modal.querySelector('#cb-remote-submit-pin');
    let selected = null; let peer = null; let channel = null; let activeSession = null; let mode = 'view'; let frame = null; let frameImage = null;
    const report = (message, error = false) => { status.textContent = message; status.style.color = error ? '#ffc2d8' : '#bfeadf'; };
    async function loadDevices() {
      const session = await currentSession(); const stale = new Date(Date.now() - 45_000).toISOString();
      const { data, error } = await client.from('remote_devices').select('id,name,last_seen_at').eq('user_id', session.user.id).gte('last_seen_at', stale).order('last_seen_at', { ascending: false });
      if (error) throw error; devices.replaceChildren();
      for (const item of data || []) { const button = document.createElement('button'); button.className = 'cb-remote-device'; button.textContent = `🖥 ${item.name}`; button.onclick = () => { selected = item; devices.querySelectorAll('button').forEach((x) => x.classList.toggle('selected', x === button)); report(`เลือก ${item.name}`); }; devices.appendChild(button); }
      if (!data?.length) report('ยังไม่พบเครื่องออนไลน์ — เปิด Commandblock.exe --remote บนเครื่องปลายทางก่อน', true);
      else { selected = data[0]; devices.firstElementChild.classList.add('selected'); report(`เลือก ${selected.name}`); }
    }
    function handleMessage(message) {
      let data; try { data = JSON.parse(typeof message === 'string' ? message : new TextDecoder().decode(message)); } catch { return; }
      if (data.type === 'frame') frame = { ...data, pieces: new Array(data.chunks) };
      if (data.type === 'frame_chunk' && frame && data.id === frame.id) { frame.pieces[data.index] = data.data; if (frame.pieces.every(Boolean)) { const binary = atob(frame.pieces.join('')); const bytes = Uint8Array.from(binary, (x) => x.charCodeAt(0)); const blob = new Blob([bytes], { type: 'image/jpeg' }); const url = URL.createObjectURL(blob); const image = new Image(); image.onload = () => { URL.revokeObjectURL(frameImage || ''); frameImage = url; canvas.width = frame.width; canvas.height = frame.height; context.drawImage(image, 0, 0); }; image.src = url; frame = null; } }
    }
    async function start(nextMode) {
      if (!selected) { report('เลือกเครื่องก่อน', true); return; } mode = nextMode; pinRow.hidden = true; pin.value = ''; report('กำลังสร้างการเชื่อมต่อ P2P…');
      try {
        const session = await currentSession(); peer = new RTCPeerConnection({ iceServers: [{ urls: 'stun:stun.l.google.com:19302' }] });
        channel = peer.createDataChannel('commandblock-remote', { ordered: true }); channel.binaryType = 'arraybuffer'; channel.onmessage = (event) => handleMessage(event.data); channel.onopen = () => report(nextMode === 'control' ? 'เชื่อมต่อแล้ว — ควบคุมได้' : 'เชื่อมต่อแล้ว — ดูหน้าจอได้'); channel.onclose = () => report('การเชื่อมต่อถูกปิด');
        peer.onconnectionstatechange = () => { if (peer?.connectionState === 'failed') report('P2P เชื่อมต่อไม่สำเร็จ ลองใช้เครือข่ายอื่น', true); };
        const offer = await peer.createOffer(); await peer.setLocalDescription(offer); await waitForIceComplete(peer);
        const { data, error } = await client.from('remote_sessions').insert({ user_id: session.user.id, device_id: selected.id, mode: nextMode, offer: peer.localDescription }).select('id').single();
        if (error || !data) throw new Error(error?.message || 'ส่งคำขอ Remote ไม่สำเร็จ'); activeSession = data.id; report('ส่งคำขอแล้ว — รอให้ยืนยันบนเครื่องปลายทาง…');
        for (let attempt = 0; attempt < 120; attempt += 1) { await new Promise((resolve) => setTimeout(resolve, 1000)); const { data: remote, error: pollError } = await client.from('remote_sessions').select('status,answer,approval_code_hash,approval_expires_at').eq('id', activeSession).single(); if (pollError) throw pollError; if (remote.status === 'denied') throw new Error('รหัสยืนยันไม่ถูกต้องหรือเครื่องปลายทางปฏิเสธคำขอ'); if (remote.status === 'closed' || remote.status === 'expired') throw new Error('คำขอถูกปิดหรือหมดอายุ'); if (remote.approval_code_hash && pinRow.hidden) { pinRow.hidden = false; pin.focus(); report('กรอกรหัส 6 หลักที่แสดงบนเครื่องปลายทางเพื่อยืนยัน'); } if (remote.answer) { pinRow.hidden = true; await peer.setRemoteDescription(remote.answer); return; } }
        throw new Error('รอการยืนยันนานเกินไป');
      } catch (error) { report(error.message || 'Remote PC ไม่สำเร็จ', true); peer?.close(); peer = null; channel = null; }
    }
    canvas.addEventListener('pointermove', (event) => { if (mode !== 'control' || channel?.readyState !== 'open') return; const rect = canvas.getBoundingClientRect(); channel.send(JSON.stringify({ type:'pointer', action:'move', x:(event.clientX-rect.left)/rect.width, y:(event.clientY-rect.top)/rect.height })); });
    for (const [eventName, action] of [['pointerdown','down'], ['pointerup','up']]) canvas.addEventListener(eventName, (event) => { if (mode !== 'control' || channel?.readyState !== 'open') return; const rect = canvas.getBoundingClientRect(); canvas.focus(); channel.send(JSON.stringify({ type:'pointer', action, x:(event.clientX-rect.left)/rect.width, y:(event.clientY-rect.top)/rect.height })); });
    canvas.addEventListener('wheel', (event) => { if (mode === 'control' && channel?.readyState === 'open') { event.preventDefault(); channel.send(JSON.stringify({ type:'wheel', delta:Math.sign(event.deltaY) })); } }, { passive:false });
    canvas.addEventListener('keydown', (event) => { if (mode === 'control' && channel?.readyState === 'open') { event.preventDefault(); channel.send(JSON.stringify({ type:'key', key:event.key })); } });
    async function disconnectRemote(hide = false) { peer?.close(); if (activeSession) await client.from('remote_sessions').update({ status:'closed' }).eq('id', activeSession); peer = null; channel = null; activeSession = null; report('ตัดการเชื่อมต่อแล้ว'); if (hide) modal.hidden = true; }
    submitPin.onclick = async () => { const code = pin.value.replace(/\D/g, '').slice(0, 6); if (code.length !== 6 || !activeSession) { report('กรอกรหัส 6 หลักให้ครบก่อน', true); return; } const { error } = await client.from('remote_sessions').update({ approval_code_input: code }).eq('id', activeSession); if (error) { report('ส่งรหัสยืนยันไม่สำเร็จ', true); return; } report('ส่งรหัสแล้ว — กำลังให้เครื่องปลายทางตรวจสอบ…'); };
    modal.querySelector('#cb-remote-view').onclick = () => start('view'); modal.querySelector('#cb-remote-control').onclick = () => start('control');
    modal.querySelector('#cb-remote-disconnect').onclick = () => disconnectRemote();
    modal.querySelector('#cb-remote-close').onclick = () => disconnectRemote(true);
    open.onclick = async () => { modal.hidden = false; await loadDevices().catch((error) => report(error.message || 'โหลดเครื่องไม่สำเร็จ', true)); };
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

  function mountDevices() {
    if (document.querySelector('#cb-devices-open')) return;
    const open = document.createElement('button');
    open.id = 'cb-devices-open'; open.className = 'pill'; open.type = 'button'; open.textContent = '🖥 My devices';
    document.querySelector('.statusbar')?.appendChild(open);
    const modal = document.createElement('section');
    modal.className = 'cb-remote-modal'; modal.hidden = true;
    modal.innerHTML = '<div class="cb-remote-card" role="dialog" aria-modal="true" aria-label="My devices"><div class="cb-remote-row"><div><h2>My devices</h2><p>จัดการเฉพาะเครื่องในบัญชีนี้</p></div><button id="cb-devices-close">ปิด</button></div><div id="cb-devices-list"></div><p class="cb-remote-note" id="cb-devices-status"></p></div>';
    document.body.appendChild(modal);
    const list = modal.querySelector('#cb-devices-list'); const status = modal.querySelector('#cb-devices-status');
    async function audit(session, device, kind, action) {
      await client.from('device_audit_events').insert({ user_id: session.user.id, device_kind: kind, device_id: device.id, action });
    }
    async function refresh() {
      const session = await currentSession(); status.textContent = 'กำลังโหลด…'; list.replaceChildren();
      const [connectors, remotes, events] = await Promise.all([
        client.from('connector_devices').select('id,name,last_seen_at').eq('user_id', session.user.id).order('last_seen_at', { ascending: false }),
        client.from('remote_devices').select('id,name,last_seen_at').eq('user_id', session.user.id).order('last_seen_at', { ascending: false }),
        client.from('device_audit_events').select('device_kind,device_id,action,mode,created_at').eq('user_id', session.user.id).order('created_at', { ascending: false }).limit(20),
      ]);
      const rows = [...(connectors.data || []).map((item) => ({...item, kind:'connector'})), ...(remotes.data || []).map((item) => ({...item, kind:'remote'}))];
      for (const item of rows) {
        const row = document.createElement('div'); row.className = 'cb-remote-device';
        const label = document.createElement('span'); label.textContent = (item.kind === 'remote' ? '🖥 Remote' : '🔗 Connector') + ' · ' + item.name + ' · ' + new Date(item.last_seen_at).toLocaleString();
        const rename = document.createElement('button'); rename.id = 'cb-device-rename'; rename.textContent = 'เปลี่ยนชื่อ';
        const revoke = document.createElement('button'); revoke.id = 'cb-device-revoke'; revoke.textContent = 'ตัดสิทธิ์';
        rename.onclick = async () => { const name = window.prompt('ชื่อเครื่อง', item.name)?.trim(); if (!name || name.length > 80) return; const table = item.kind === 'remote' ? 'remote_devices' : 'connector_devices'; const { error } = await client.from(table).update({ name }).eq('id', item.id).eq('user_id', session.user.id); if (error) { status.textContent = error.message; return; } await audit(session, item, item.kind, 'renamed'); refresh(); };
        revoke.onclick = async () => { if (!window.confirm('ตัดสิทธิ์ ' + item.name + ' ? ต้องเปิด Connector/Remote ใหม่เพื่อลงทะเบียนอีกครั้ง')) return; const table = item.kind === 'remote' ? 'remote_devices' : 'connector_devices'; if (item.kind === 'remote') await client.from('remote_sessions').update({ status:'closed' }).eq('device_id', item.id).eq('user_id', session.user.id); const { error } = await client.from(table).delete().eq('id', item.id).eq('user_id', session.user.id); if (error) { status.textContent = error.message; return; } await audit(session, item, item.kind, 'revoked').catch(() => {}); refresh(); };
        row.append(label, rename, revoke); list.appendChild(row);
      }
      const history = document.createElement('p'); history.className = 'cb-remote-note'; history.textContent = 'ประวัติล่าสุด: ' + (events.data || []).map((event) => event.action + ' · ' + new Date(event.created_at).toLocaleString()).join(' | ');
      list.appendChild(history); status.textContent = rows.length ? '' : 'ยังไม่พบอุปกรณ์ในบัญชีนี้';
    }
    open.onclick = async () => { modal.hidden = false; try { await refresh(); } catch (error) { status.textContent = error.message || 'โหลดอุปกรณ์ไม่ได้'; } };
    modal.querySelector('#cb-devices-close').onclick = () => { modal.hidden = true; };
  }

  function mountAccount() {
    if (document.querySelector('#cb-account-open')) return;
    const open = document.createElement('button'); open.id = 'cb-account-open'; open.className = 'pill'; open.type = 'button'; open.textContent = 'บัญชี';
    document.querySelector('.statusbar')?.appendChild(open);
    const modal = document.createElement('section'); modal.className = 'cb-remote-modal'; modal.hidden = true;
    modal.innerHTML = '<div class="cb-remote-card" role="dialog" aria-modal="true" aria-label="บัญชี"><div class="cb-remote-row"><div><h2>บัญชีและการใช้งาน</h2><p id="cb-account-email"></p></div><button id="cb-account-close">ปิด</button></div><label>ชื่อที่แสดง <input id="cb-account-name" maxlength="80"></label><div class="cb-remote-actions"><button id="cb-account-save" class="primary">บันทึกชื่อ</button><button id="cb-account-global">ออกจากทุกเครื่อง</button></div><p id="cb-account-usage" class="cb-remote-status"></p></div>';
    document.body.appendChild(modal);
    const report = modal.querySelector('#cb-account-usage');
    async function refresh() {
      const session = await currentSession();
      modal.querySelector('#cb-account-email').textContent = session.user.email || '';
      const [{ data: profile }, { data: usage, error }] = await Promise.all([
        client.from('profiles').select('display_name').eq('id', session.user.id).maybeSingle(),
        client.from('usage_events').select('total_tokens,cost_usd,created_at').eq('user_id', session.user.id).order('created_at', { ascending: false }).limit(500),
      ]);
      if (error) throw error;
      modal.querySelector('#cb-account-name').value = profile?.display_name || session.user.user_metadata?.full_name || '';
      const today = new Date().toDateString(); const month = new Date().getMonth(); const year = new Date().getFullYear();
      const sum = (rows) => rows.reduce((total, row) => total + Number(row.total_tokens || 0), 0);
      const rows = usage || []; const daily = rows.filter((row) => new Date(row.created_at).toDateString() === today); const monthly = rows.filter((row) => { const date = new Date(row.created_at); return date.getMonth() === month && date.getFullYear() === year; });
      report.textContent = `วันนี้ ${sum(daily).toLocaleString()} tokens · เดือนนี้ ${sum(monthly).toLocaleString()} tokens`;
    }
    modal.querySelector('#cb-account-save').onclick = async () => { const session = await currentSession(); const display_name = modal.querySelector('#cb-account-name').value.trim(); if (!display_name) return; const { error } = await client.from('profiles').upsert({ id: session.user.id, display_name }); report.textContent = error ? error.message : 'บันทึกชื่อแล้ว'; };
    modal.querySelector('#cb-account-global').onclick = async () => { if (!window.confirm('ออกจากระบบทุกเครื่อง?')) return; const { error } = await client.auth.signOut({ scope: 'global' }); if (error) { report.textContent = error.message; return; } location.reload(); };
    modal.querySelector('#cb-account-close').onclick = () => { modal.hidden = true; };
    open.onclick = async () => { modal.hidden = false; await refresh().catch((error) => { report.textContent = error.message || 'โหลดบัญชีไม่ได้'; }); };
  }

  function mountAuthGate() {
    const style = document.createElement('style');
    style.textContent = `#cb-cloud-gate{position:fixed;inset:0;z-index:9999;display:grid;place-items:center;padding:24px;background:radial-gradient(circle at 50% 0,#251044 0,#0d0918 58%,#07060d 100%);color:#f5efff;font-family:"Segoe UI","Noto Sans Thai",sans-serif}#cb-cloud-gate[hidden]{display:none}.cb-cloud-card{width:min(440px,100%);padding:32px;border:1px solid rgba(184,137,255,.35);border-radius:24px;background:rgba(23,14,43,.84);box-shadow:0 28px 80px rgba(0,0,0,.48);backdrop-filter:blur(18px)}.cb-cloud-card h1{margin:0 0 10px;font-size:28px}.cb-cloud-card p{color:#cdbfe8;line-height:1.6}.cb-cloud-card input{width:100%;margin-top:10px;padding:13px 14px;border:1px solid #5b3e84;border-radius:12px;background:#100a20;color:#fff;font:inherit}.cb-cloud-actions{display:grid;gap:10px;margin-top:18px}.cb-cloud-actions button{padding:12px;border:1px solid #7344ba;border-radius:12px;background:#271343;color:#fff;font:inherit;font-weight:700;cursor:pointer}.cb-cloud-actions button.primary{border:0;background:linear-gradient(135deg,#7034df,#a65cff)}.cb-cloud-status{min-height:24px;margin-top:14px;color:#bfeadf;font-size:13px}.cb-cloud-link{background:none!important;border:0!important;color:#c69aff!important;text-decoration:underline;font-weight:400!important}`;
    style.textContent += `#settingsModal .set-sec:nth-of-type(2){display:none}#statsRight{display:none}#modelPill{max-width:none!important;overflow:visible!important}#cb-cloud-logout{margin-left:auto;flex-shrink:0;padding:3px 10px}@media (max-width: 760px){body{grid-template-columns:44px minmax(0,1fr);height:100dvh}#histpane,#rightpane{display:none}#chatpane{min-width:0}.chat-head{gap:7px;padding:10px}.chat-title .sub{display:none}.chat-title h1{font-size:15px}.pill#folderBtn{display:none}.chat-foot{padding:8px}.statusbar{font-size:10px}.feedback{display:none}.inputbox{min-height:46px}.logo img{width:34px;height:34px}.wrap{padding:14px 10px}.bubble{max-width:96%}}`;
    document.head.appendChild(style);
    const gate = document.createElement('section');
    gate.id = 'cb-cloud-gate';
    gate.innerHTML = `<div class="cb-cloud-card"><h1>CommandBlock Web</h1><p>เข้าสู่ระบบเพื่อใช้ CommandBlock เดิมบนเว็บ พร้อม Cloud chat ของคุณ</p><form id="cb-cloud-form"><input id="cb-cloud-name" placeholder="ชื่อที่แสดง (เฉพาะตอนสมัคร)" autocomplete="name"><input id="cb-cloud-email" type="email" placeholder="อีเมล" autocomplete="email" required><input id="cb-cloud-password" type="password" placeholder="รหัสผ่านอย่างน้อย 8 ตัว" autocomplete="current-password" minlength="8" required><div class="cb-cloud-actions"><button class="primary" type="submit">เข้าสู่ระบบ</button><button id="cb-cloud-register" type="button">สร้างบัญชี</button><button id="cb-cloud-reset" class="cb-cloud-link" type="button">ลืมรหัสผ่าน</button></div></form><div id="cb-cloud-status" class="cb-cloud-status" role="status"></div><p>API key ของ DeepSeek อยู่เฉพาะ session นี้ และไม่ถูกบันทึกในบัญชี</p></div>`;
    document.body.appendChild(gate);
    const status = gate.querySelector('#cb-cloud-status');
    const email = gate.querySelector('#cb-cloud-email');
    const password = gate.querySelector('#cb-cloud-password');
    const name = gate.querySelector('#cb-cloud-name');
    const report = (message, error = false) => { status.textContent = message; status.style.color = error ? '#ffc2d8' : '#bfeadf'; };
    const errorText = (error) => error?.message === 'Invalid login credentials' ? 'อีเมลหรือรหัสผ่านไม่ถูกต้อง' : (error?.message || 'เกิดข้อผิดพลาด กรุณาลองใหม่');
    const openApp = () => { gate.hidden = true; document.documentElement.classList.remove('cb-auth-pending'); mountRemotePC(); mountDevices(); mountAccount(); };
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
