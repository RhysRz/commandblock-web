import { createClient } from 'https://esm.sh/@supabase/supabase-js@2';
import { getSupabaseConfig } from './config.js';
import { createAuthController } from './auth.js';

export function start() {
  const authGate = document.querySelector('#authGate');
  const appGate = document.querySelector('#appGate');
  const status = document.querySelector('#authStatus');
  const email = document.querySelector('#authEmail');
  const password = document.querySelector('#authPassword');
  const name = document.querySelector('#authName');
  const view = {
    showAuthenticated: () => { authGate.hidden = true; appGate.hidden = false; },
    showUnauthenticated: () => { authGate.hidden = false; appGate.hidden = true; },
    showNotice: (text) => { status.textContent = text; },
  };
  try {
    const { url, anonKey } = getSupabaseConfig();
    const auth = createAuthController(createClient(url, anonKey), view);
    document.querySelector('#authForm').addEventListener('submit', async (event) => {
      event.preventDefault(); try { await auth.signIn(email.value, password.value); } catch (error) { view.showNotice(error.message); }
    });
    document.querySelector('[data-auth-action="register"]').addEventListener('click', async () => {
      try { await auth.signUp(email.value, password.value, name.value); } catch (error) { view.showNotice(error.message); }
    });
    document.querySelector('[data-auth-action="reset"]').addEventListener('click', async () => {
      try { await auth.sendPasswordReset(email.value); } catch (error) { view.showNotice(error.message); }
    });
  } catch (error) { view.showNotice(error.message); }
}

window.CommandblockWeb = { start };
if ('serviceWorker' in navigator) navigator.serviceWorker.register('./sw.js').catch(() => {});
start();
