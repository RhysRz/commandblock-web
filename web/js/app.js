import { createClient } from 'https://esm.sh/@supabase/supabase-js@2';
import { getSupabaseConfig } from './config.js'; import { createAuthController } from './auth.js';
import { createSettingsStore } from './settings.js'; import { appendMessage, createConversation, sendCloudMessage } from './chat.js';
export function start() {
 const authGate=document.querySelector('#authGate'),appGate=document.querySelector('#appGate'),status=document.querySelector('#authStatus'), email=document.querySelector('#authEmail'),password=document.querySelector('#authPassword'),name=document.querySelector('#authName'), dialog=document.querySelector('#settingsDialog'), key=document.querySelector('#cloudKey'), settings=createSettingsStore(); let client,user,session,conversation;
 const view={showAuthenticated:(u,activeSession)=>{user=u;session=activeSession;authGate.hidden=true;appGate.hidden=false;},showUnauthenticated:()=>{authGate.hidden=false;appGate.hidden=true;},showNotice:t=>status.textContent=t};
 try { const c=getSupabaseConfig(); client=createClient(c.url,c.anonKey); const auth=createAuthController(client,view);
  document.querySelector('#authForm').addEventListener('submit',async e=>{e.preventDefault();try{await auth.signIn(email.value,password.value)}catch(x){view.showNotice(x.message)}});
  document.querySelector('[data-auth-action="register"]').onclick=async()=>{try{await auth.signUp(email.value,password.value,name.value)}catch(x){view.showNotice(x.message)}};
  document.querySelector('[data-auth-action="reset"]').onclick=async()=>{try{await auth.sendPasswordReset(email.value)}catch(x){view.showNotice(x.message)}};
  document.querySelector('.model-chip').onclick=()=>dialog.showModal(); document.querySelector('#saveSettings').onclick=()=>settings.set({apiKey:key.value.trim()});
  document.querySelector('.composer').addEventListener('submit',async e=>{e.preventDefault();const box=document.querySelector('#message'),text=box.value.trim(),s=settings.get();if(!text)return;if(!s.apiKey){dialog.showModal();return}try{if(!conversation)conversation=await createConversation(client,user.id,s.model);await appendMessage(client,conversation.id,user.id,'user',text);const answer=await sendCloudMessage(client,session,s,text);await appendMessage(client,conversation.id,user.id,'assistant',answer);document.querySelector('#chatMessages').innerHTML=`<p>${answer.replace(/</g,'&lt;')}</p>`;box.value=''}catch(x){document.querySelector('#chatMessages').innerHTML=`<p>เกิดข้อผิดพลาด: ${x.message}</p>`}});
 }catch(x){view.showNotice(x.message)} }
window.CommandblockWeb={start}; if('serviceWorker'in navigator)navigator.serviceWorker.register('./sw.js').catch(()=>{}); start();
