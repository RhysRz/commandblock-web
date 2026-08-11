# Commandblock Canonical Web UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deploy the existing Commandblock UI as the hosted web application with Supabase-authenticated DeepSeek chat.

**Architecture:** A build script transforms `src/ui.html` into a static Pages artifact and injects one synchronous cloud adapter before the original UI scripts. The adapter preserves the `/api/*` contract expected by the UI, delegates cloud chat to the existing Supabase Edge Function, and clearly declines browser-incompatible desktop operations.

**Tech Stack:** Static HTML, browser JavaScript, Node.js build/test scripts, GitHub Pages, Supabase Auth and Edge Functions.

## Global Constraints

- `src/ui.html` remains the canonical UI; do not create a replacement chat layout.
- The DeepSeek key may exist only in browser `sessionStorage`.
- Only the Supabase publishable key may be embedded in the static site.
- Filesystem, terminal, and native dialog requests must never be faked in a browser.

---

### Task 1: Build the hosted page from the canonical UI

**Files:**
- Create: `scripts/build-web.mjs`
- Create: `tests/canonical-web-build.test.cjs`
- Modify: `.github/workflows/deploy-pages.yml`

**Interfaces:**
- Consumes: `src/ui.html`, `web/cloud-adapter.js`, and `web/manifest.webmanifest`.
- Produces: `site/index.html`, `site/cloud-adapter.js`, and `site/manifest.webmanifest`.

- [ ] **Step 1: Write the failing test**

```js
test('web build injects the cloud adapter before Commandblock scripts', () => {
  execFileSync(process.execPath, ['scripts/build-web.mjs'], { cwd: root });
  const html = readFileSync(join(root, 'site', 'index.html'), 'utf8');
  assert.match(html, /id="chat"/);
  assert.ok(html.indexOf('cloud-adapter.js') < html.indexOf('<script>'));
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/canonical-web-build.test.cjs`

Expected: FAIL because `scripts/build-web.mjs` does not exist.

- [ ] **Step 3: Write minimal implementation**

```js
const html = readFileSync('src/ui.html', 'utf8');
const injected = html.replace('<script>', '<script src="./cloud-adapter.js"></script><script>');
mkdirSync('site', { recursive: true });
writeFileSync('site/index.html', injected);
copyFileSync('web/cloud-adapter.js', 'site/cloud-adapter.js');
```

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test tests/canonical-web-build.test.cjs`

Expected: PASS.

- [ ] **Step 5: Update deployment workflow and commit**

```yaml
- uses: actions/setup-node@v4
- run: node scripts/build-web.mjs
- uses: actions/upload-pages-artifact@v3
  with: { path: site }
```

Commit: `git commit -m "feat: build web from canonical Commandblock UI"`

### Task 2: Provide the cloud adapter and authentication gate

**Files:**
- Create: `web/cloud-adapter.js`
- Create: `tests/cloud-adapter-contract.test.cjs`

**Interfaces:**
- Consumes: Supabase URL and publishable key defined in `cloud-adapter.js`.
- Produces: a `window.fetch` adapter for `/api/state`, `/api/models`, `/api/model`, `/api/chat`, `/api/history`, `/api/notes`, and unsupported desktop APIs.

- [ ] **Step 1: Write the failing test**

```js
test('adapter uses session storage and routes cloud chat through Supabase', () => {
  const source = readFileSync(join(root, 'web', 'cloud-adapter.js'), 'utf8');
  assert.match(source, /sessionStorage/);
  assert.match(source, /functions\/v1\/chat/);
  assert.match(source, /Desktop Connector/);
  assert.doesNotMatch(source, /localStorage\.setItem\([^)]*(api|key)/i);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/cloud-adapter-contract.test.cjs`

Expected: FAIL because the adapter does not exist.

- [ ] **Step 3: Write minimal implementation**

```js
window.fetch = async (input, init = {}) => {
  if (pathname === '/api/chat') return cloudChat(init);
  if (pathname === '/api/state') return json(state());
  if (desktopOnly.has(pathname)) return json({ ok: false, requires_connector: true });
  return originalFetch(input, init);
};
```

The implementation creates a full-page email/password auth gate, uses the hosted Supabase client, asks for a DeepSeek key only when chat is first sent, and returns synthetic SSE `event: content` frames from the Edge Function JSON response.

- [ ] **Step 4: Run tests to verify they pass**

Run: `node --test tests/cloud-adapter-contract.test.cjs tests/canonical-web-build.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

Commit: `git commit -m "feat: connect canonical web UI to cloud chat"`

### Task 3: Verify the complete deployment contract

**Files:**
- Modify: `tests/web-shell.test.cjs`
- Modify: `README.md`

**Interfaces:**
- Consumes: generated `site/index.html` and deployed Pages workflow.
- Produces: documentation that distinguishes cloud chat from Desktop Connector features.

- [ ] **Step 1: Write the failing test**

```js
test('deployment workflow publishes generated site rather than legacy web shell', () => {
  const workflow = readFileSync(join(root, '.github/workflows/deploy-pages.yml'), 'utf8');
  assert.match(workflow, /node scripts\/build-web\.mjs/);
  assert.match(workflow, /path: site/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test tests/web-shell.test.cjs`

Expected: FAIL because Pages currently uploads `web` directly.

- [ ] **Step 3: Update documentation**

Add a concise web section stating that chat is cloud-ready and local project tools require the desktop app until a Desktop Connector is installed.

- [ ] **Step 4: Run full verification**

Run: `node --test tests/*.test.cjs; cargo test`

Expected: all Node tests and Rust tests PASS.

- [ ] **Step 5: Commit and publish**

Commit: `git commit -m "docs: clarify Commandblock web capabilities"`

Push `main`, wait for the Pages workflow, and manually confirm the hosted page has the canonical UI after authentication.
