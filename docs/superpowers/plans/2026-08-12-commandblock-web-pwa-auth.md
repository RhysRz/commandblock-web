# Commandblock Web/PWA Authentication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a free-tier-hosted Commandblock Web/PWA with Supabase email/password authentication, responsive mobile/Remote PC UI, and bring-your-own cloud model keys.

**Architecture:** A dependency-free static frontend in `web/` is hosted by GitHub Pages and loads Supabase’s browser SDK from its official ESM CDN. Supabase Auth and RLS-protected Postgres persist identity and conversations; a Supabase Edge Function validates the session and proxies a one-request, user-supplied OpenAI-compatible key to the selected provider without persisting it.

**Tech Stack:** HTML, CSS, browser ES modules, Node.js built-in test runner, Supabase Auth/Postgres/RLS/Edge Functions, Deno, GitHub Pages, Web App Manifest and Service Worker.

## Global Constraints

- Keep the existing Rust desktop app and `src/ui.html` functioning; the PWA is a separate `web/` client in this release.
- Support email/password, email confirmation, and password reset through Supabase Auth; exclude Google OAuth to avoid Google Cloud Billing setup.
- Allow self-registration and enforce per-user data isolation with RLS on every user-data table.
- Never commit or persist user Cloud API keys in Supabase, browser storage, GitHub, chat history, logs, or examples.
- Use GitHub Pages and Supabase Free-tier-compatible resources only; Cloud model charges remain the responsibility of each user’s own key.
- Mobile target starts at 360px; desktop/Remote PC layout starts at 768px; controls have a minimum 44px target.
- Do not expose PC files or Terminal from the PWA. A Desktop Connector is out of scope.

---

## File Structure

- `web/index.html`: Shell, auth gate, chat surface, settings modal, and responsive navigation.
- `web/styles.css`: Obsidian–ม่วง design tokens plus mobile-first and desktop layout rules.
- `web/js/config.js`: Validates public Supabase configuration and creates the Supabase client.
- `web/js/auth.js`: Sign-up, email/password login, Google OAuth, logout, reset password, and auth-gate state.
- `web/js/chat.js`: Conversation and message persistence plus streaming chat request handling.
- `web/js/settings.js`: In-memory Cloud provider key/model selection; no browser persistence.
- `web/js/app.js`: Routes UI events and initializes the authenticated application.
- `web/manifest.webmanifest`, `web/sw.js`, `web/assets/`: PWA metadata, safe app-shell cache, and existing Commandblock icons.
- `supabase/migrations/202608120001_commandblock_web.sql`: Profiles, conversations, messages, trigger, indexes, RLS policies.
- `supabase/functions/chat/index.ts`: Authenticated and CORS-safe OpenAI-compatible Cloud model proxy.
- `supabase/functions/chat/index.test.ts`: Deno tests for authorization, validation, header forwarding, and no-key leakage.
- `tests/web-shell.test.cjs`, `tests/web-auth-contract.test.cjs`, `tests/web-responsive-contract.test.cjs`: Node contract tests for static frontend behavior.
- `.gitignore`, `.env.example`, `config.example.json`, `README.md`: Safe source-control and setup documentation.

### Task 1: Add a safe Web/PWA scaffold and repository boundaries

**Files:**
- Create: `web/index.html`, `web/styles.css`, `web/js/config.js`, `web/js/app.js`, `web/manifest.webmanifest`, `web/sw.js`, `tests/web-shell.test.cjs`
- Modify: `.gitignore`, `.env.example`, `config.example.json`, `README.md`

**Interfaces:**
- Produces `window.CommandblockWeb.start()` for the app bootstrap in later tasks.
- Produces `getSupabaseConfig(): { url: string, anonKey: string }` for `auth.js`.

- [ ] **Step 1: Write the failing web-shell test**

```js
test('web shell exposes an auth gate, app gate, PWA manifest, and module bootstrap', () => {
  const html = fs.readFileSync(path.join(root, 'web', 'index.html'), 'utf8');
  assert.match(html, /id="authGate"/);
  assert.match(html, /id="appGate"/);
  assert.match(html, /rel="manifest" href="manifest\.webmanifest"/);
  assert.match(html, /type="module" src="js\/app\.js"/);
});
```

- [ ] **Step 2: Run the test and verify it fails because the web shell does not exist**

Run: `node --test tests/web-shell.test.cjs`

Expected: FAIL with `ENOENT` for `web/index.html`.

- [ ] **Step 3: Implement the minimal safe shell and public configuration contract**

```js
// web/js/config.js
export function getSupabaseConfig() {
  const url = document.querySelector('meta[name="supabase-url"]')?.content?.trim();
  const anonKey = document.querySelector('meta[name="supabase-anon-key"]')?.content?.trim();
  if (!url || !anonKey || url.includes('YOUR_') || anonKey.includes('YOUR_')) {
    throw new Error('ตั้งค่า Supabase URL และ anon key ก่อนเริ่มใช้งาน');
  }
  return { url, anonKey };
}
```

Create `index.html` with empty `authGate` and hidden `appGate`; link the manifest and load `js/app.js` as a module. Create `.gitignore` rules for `config.json`, `.env`, `.env.*` except `.env.example`, `buff_session.json`, `target/`, `dist/`, and `installer/payload/`. Put placeholders only in `.env.example` and `config.example.json`; do not copy the current secret-bearing config.

- [ ] **Step 4: Run the shell test and repository secret scan**

Run: `node --test tests/web-shell.test.cjs; rg -n 'gsk_|sk-or-|sk-[A-Za-z0-9]{16,}|AIza' --glob '!config.json' --glob '!target/**' --glob '!installer/payload/**' .`

Expected: shell test PASS; secret scan returns no tracked-source matches.

### Task 2: Create the Supabase schema and prove user-data isolation

**Files:**
- Create: `supabase/migrations/202608120001_commandblock_web.sql`, `supabase/tests/rls-contract.sql`
- Modify: `README.md`

**Interfaces:**
- Produces tables `profiles`, `conversations`, and `messages` with `user_id uuid` ownership.
- Produces RLS policies named `users_manage_own_profiles`, `users_manage_own_conversations`, and `users_manage_own_messages`.

- [ ] **Step 1: Write the failing SQL contract test**

```sql
begin;
select plan(4);
select has_table('public', 'profiles');
select has_table('public', 'conversations');
select has_table('public', 'messages');
select throws_ok(
  $$insert into public.conversations (user_id, title, model_id)
    values ('00000000-0000-0000-0000-000000000002', 'blocked', 'deepseek-v4-flash')$$,
  '42501',
  'row-level security policy',
  'a user cannot insert a conversation for another user'
);
rollback;
```

- [ ] **Step 2: Run the local Supabase database test and verify it fails before the migration exists**

Run: `supabase start && supabase test db`

Expected: FAIL because the three tables and policies do not exist.

- [ ] **Step 3: Implement schema, trigger, indexes, and RLS**

```sql
create table public.conversations (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  title text not null default 'แชทใหม่',
  model_id text not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
alter table public.conversations enable row level security;
create policy users_manage_own_conversations on public.conversations
  for all using ((select auth.uid()) = user_id)
  with check ((select auth.uid()) = user_id);
```

Add equivalent ownership columns and policies for `profiles` and `messages`, a `handle_new_user()` trigger that creates a profile on `auth.users` insert, and indexes on `conversations(user_id, updated_at desc)` and `messages(conversation_id, created_at)`.

- [ ] **Step 4: Re-run schema/RLS verification**

Run: `supabase db reset && supabase test db`

Expected: PASS; the cross-user insert is denied by RLS.

### Task 3: Implement authentication and account recovery UI

**Files:**
- Create: `web/js/auth.js`, `tests/web-auth-contract.test.cjs`
- Modify: `web/index.html`, `web/styles.css`, `web/js/app.js`, `README.md`

**Interfaces:**
- Consumes `getSupabaseConfig()`.
- Produces `createAuthController(client, view)` with `signUp`, `signIn`, `signInWithGoogle`, `sendPasswordReset`, `signOut`, and `onAuthStateChange` methods.
- `view.showAuthenticated(user)` and `view.showUnauthenticated()` are implemented by `app.js`.

- [ ] **Step 1: Write a failing auth contract test**

```js
test('auth module exposes all supported account actions', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'js', 'auth.js'), 'utf8');
  for (const name of ['signUp', 'signIn', 'signInWithGoogle', 'sendPasswordReset', 'signOut']) {
    assert.match(source, new RegExp(`async function ${name}`));
  }
  assert.match(source, /onAuthStateChange/);
});
```

- [ ] **Step 2: Run the auth contract test and verify it fails because `auth.js` is absent**

Run: `node --test tests/web-auth-contract.test.cjs`

Expected: FAIL with `ENOENT` for `web/js/auth.js`.

- [ ] **Step 3: Implement the auth controller and accessible screens**

```js
export async function signIn(email, password) {
  const { error } = await client.auth.signInWithPassword({ email, password });
  if (error) throw error;
}

export async function signInWithGoogle() {
  const { error } = await client.auth.signInWithOAuth({
    provider: 'google',
    options: { redirectTo: `${location.origin}${location.pathname}` },
  });
  if (error) throw error;
}
```

Add Register, Login, Forgot password, and post-reset views with labels, inline actionable errors, 44px controls, and focus management. Document the required Supabase Auth redirect URL and Google provider credentials in `README.md` without placing their secrets in source.

- [ ] **Step 4: Run frontend contracts**

Run: `node --test tests/web-shell.test.cjs tests/web-auth-contract.test.cjs`

Expected: PASS.

### Task 4: Build the authenticated Cloud chat proxy without retaining keys

**Files:**
- Create: `supabase/functions/chat/index.ts`, `supabase/functions/chat/index.test.ts`
- Modify: `README.md`

**Interfaces:**
- Accepts `POST /functions/v1/chat` JSON: `{ conversationId: string, model: string, baseUrl: string, apiKey: string, message: string }`.
- Requires a valid Supabase bearer token and returns provider response text or a sanitized `{ error: string }` JSON response.
- Produces no database row or log output containing `apiKey`.

- [ ] **Step 1: Write failing Deno tests for auth and key validation**

```ts
Deno.test('rejects a request without a Supabase bearer token', async () => {
  const response = await handler(new Request('http://localhost/chat', {
    method: 'POST', body: JSON.stringify(validBody), headers: { 'content-type': 'application/json' },
  }));
  assertEquals(response.status, 401);
});

Deno.test('does not forward a provider request when apiKey is missing', async () => {
  const response = await handler(requestFor({ ...validBody, apiKey: '' }));
  assertEquals(response.status, 400);
});
```

- [ ] **Step 2: Run the Edge Function tests and verify they fail because the handler is absent**

Run: `deno test --allow-env --allow-net supabase/functions/chat/index.test.ts`

Expected: FAIL with module-not-found for `index.ts`.

- [ ] **Step 3: Implement the CORS-safe, authenticated proxy**

```ts
const authorization = req.headers.get('Authorization') ?? '';
const { data: { user } } = await supabase.auth.getUser(authorization.replace(/^Bearer\s+/i, ''));
if (!user) return json({ error: 'กรุณาเข้าสู่ระบบก่อนส่งข้อความ' }, 401);

const provider = await fetch(`${baseUrl.replace(/\/$/, '')}/chat/completions`, {
  method: 'POST',
  headers: { Authorization: `Bearer ${apiKey}`, 'content-type': 'application/json' },
  body: JSON.stringify({ model, messages: [{ role: 'user', content: message }] }),
});
```

Validate HTTPS provider URLs against an allowlist initially containing `https://api.deepseek.com`; reject empty, oversized, or malformed fields; remove authorization details from all error paths; and configure CORS for the GitHub Pages origin supplied through `ALLOWED_ORIGIN` Edge Function secret.

- [ ] **Step 4: Run Edge Function tests and deploy locally**

Run: `deno test --allow-env --allow-net supabase/functions/chat/index.test.ts; supabase functions serve chat --no-verify-jwt`

Expected: Deno tests PASS; local function starts without printing provider keys.

### Task 5: Implement persisted conversations and the responsive Commandblock chat experience

**Files:**
- Create: `web/js/chat.js`, `web/js/settings.js`, `tests/web-responsive-contract.test.cjs`
- Modify: `web/index.html`, `web/styles.css`, `web/js/app.js`

**Interfaces:**
- Consumes authenticated Supabase client and session-only `getModelSettings()` result.
- Produces `createConversation()`, `listConversations()`, `appendMessage()`, and `sendCloudMessage()`.
- `getModelSettings()` returns `{ model: string, baseUrl: string, apiKey: string }` only while the page session is open.

- [ ] **Step 1: Write failing responsive and key-retention tests**

```js
test('mobile CSS supplies bottom navigation and desktop CSS supplies a three-pane layout', () => {
  const css = fs.readFileSync(path.join(root, 'web', 'styles.css'), 'utf8');
  assert.match(css, /@media\s*\(max-width:\s*767px\)/);
  assert.match(css, /\.bottom-nav/);
  assert.match(css, /@media\s*\(min-width:\s*768px\)/);
  assert.match(css, /grid-template-columns/);
});

test('settings never persist an API key to browser storage', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'js', 'settings.js'), 'utf8');
  assert.doesNotMatch(source, /localStorage|sessionStorage|indexedDB/);
});
```

- [ ] **Step 2: Run the responsive contract test and verify it fails because modules are absent**

Run: `node --test tests/web-responsive-contract.test.cjs`

Expected: FAIL with `ENOENT` for `web/js/settings.js` or assertion failure for missing breakpoints.

- [ ] **Step 3: Implement chat persistence, settings, and layout adaptation**

```js
export function createSettingsStore() {
  let value = { model: 'deepseek-v4-flash', baseUrl: 'https://api.deepseek.com', apiKey: '' };
  return { get: () => ({ ...value }), set: (next) => { value = { ...value, ...next }; }, clear: () => { value.apiKey = ''; } };
}
```

Insert each user and assistant message through Supabase under the signed-in user’s conversation. On send, require a session-only API key, call the Edge Function with the current access token, and render a clear provider error without echoing the key. On 360px show chat with bottom navigation and a full-screen settings sheet; at 768px show history/sidebar plus the familiar utility layout; on narrower Remote PC windows collapse side panes into drawers.

- [ ] **Step 4: Run all frontend contract tests**

Run: `node --test tests/web-shell.test.cjs tests/web-auth-contract.test.cjs tests/web-responsive-contract.test.cjs`

Expected: PASS.

### Task 6: Finish PWA behavior, GitHub Pages automation, and setup documentation

**Files:**
- Create: `.github/workflows/deploy-pages.yml`, `tests/pwa-safety.test.cjs`
- Modify: `web/manifest.webmanifest`, `web/sw.js`, `README.md`, `.gitignore`

**Interfaces:**
- Produces a GitHub Pages deployment that publishes only `web/`.
- Produces an installable PWA shell that caches static assets only.

- [ ] **Step 1: Write failing PWA safety test**

```js
test('service worker caches only static app-shell assets', () => {
  const source = fs.readFileSync(path.join(root, 'web', 'sw.js'), 'utf8');
  assert.match(source, /CACHE_NAME/);
  assert.doesNotMatch(source, /\/functions\/v1\/chat|Authorization|apiKey/);
});

test('Pages workflow publishes the web directory', () => {
  const source = fs.readFileSync(path.join(root, '.github', 'workflows', 'deploy-pages.yml'), 'utf8');
  assert.match(source, /path:\s*web/);
});
```

- [ ] **Step 2: Run the PWA safety test and verify it fails because the workflow is absent**

Run: `node --test tests/pwa-safety.test.cjs`

Expected: FAIL with `ENOENT` for `.github/workflows/deploy-pages.yml`.

- [ ] **Step 3: Implement static-only caching and Pages deployment**

```js
const APP_SHELL = ['./', './index.html', './styles.css', './js/app.js', './manifest.webmanifest'];
self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET' || new URL(event.request.url).origin !== location.origin) return;
  event.respondWith(caches.match(event.request).then((hit) => hit ?? fetch(event.request)));
});
```

Create a Pages workflow with `actions/configure-pages`, `actions/upload-pages-artifact` using `path: web`, and `actions/deploy-pages`. Document exact Supabase Dashboard steps: create project, run migration, enable Email and Google providers, set redirect URLs, set `ALLOWED_ORIGIN`, deploy the function, then put only the public Supabase URL and anon key into the web build configuration.

- [ ] **Step 4: Run the full verification set**

Run: `node --test tests/web-shell.test.cjs tests/web-auth-contract.test.cjs tests/web-responsive-contract.test.cjs tests/pwa-safety.test.cjs; cargo test`

Expected: all Node tests PASS and the existing Rust test suite PASS.

- [ ] **Step 5: Prepare a safe GitHub handoff**

Run: `git status --short; git check-ignore config.json .env 2>$null`

Expected: only intended source, test, and documentation files are ready; `config.json` and `.env` are ignored before any repository is published.
