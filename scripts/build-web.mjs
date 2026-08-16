import { copyFileSync, cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sourcePath = resolve(root, 'src/ui.html');
const adapterPath = resolve(root, 'web/cloud-adapter.js');
const recoveryPath = resolve(root, 'web/chat-recovery.js');
const timelinePath = resolve(root, 'web/chat-timeline.js');
const sessionStorePath = resolve(root, 'web/session-store.js');
const previewTabsPath = resolve(root, 'web/preview-tabs.js');
const sitePath = resolve(root, 'site');

if (!existsSync(adapterPath)) throw new Error('Missing web/cloud-adapter.js');
if (!existsSync(recoveryPath)) throw new Error('Missing web/chat-recovery.js');
if (!existsSync(timelinePath)) throw new Error('Missing web/chat-timeline.js');
if (!existsSync(sessionStorePath)) throw new Error('Missing web/session-store.js');
if (!existsSync(previewTabsPath)) throw new Error('Missing web/preview-tabs.js');

rmSync(sitePath, { recursive: true, force: true });
mkdirSync(sitePath, { recursive: true });

const original = readFileSync(sourcePath, 'utf8');
const firstScript = original.indexOf('<script>');
if (firstScript < 0) throw new Error('src/ui.html must contain an inline script');

const injected = [
  '<script src="https://cdn.jsdelivr.net/npm/@supabase/supabase-js@2"></script>',
  '<script src="./chat-recovery.js"></script>',
  '<script src="./cloud-adapter.js"></script>',
  '<script>',
].join('\n');
const hosted = original
  .replace('<script>', injected)
  .replaceAll('src="/assets/', 'src="./assets/');

writeFileSync(resolve(sitePath, 'index.html'), hosted);
copyFileSync(adapterPath, resolve(sitePath, 'cloud-adapter.js'));
copyFileSync(recoveryPath, resolve(sitePath, 'chat-recovery.js'));
copyFileSync(resolve(root, 'web/manifest.webmanifest'), resolve(sitePath, 'manifest.webmanifest'));
if (existsSync(resolve(root, 'assets'))) cpSync(resolve(root, 'assets'), resolve(sitePath, 'assets'), { recursive: true });
mkdirSync(resolve(sitePath, 'assets'), { recursive: true });
copyFileSync(timelinePath, resolve(sitePath, 'assets', 'chat-timeline.js'));
copyFileSync(sessionStorePath, resolve(sitePath, 'assets', 'session-store.js'));
copyFileSync(previewTabsPath, resolve(sitePath, 'assets', 'preview-tabs.js'));
