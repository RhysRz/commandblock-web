const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('updater stages and replaces its own sidecar after the app closes', () => {
  const update = fs.readFileSync(path.join(__dirname, '..', 'src', 'update.rs'), 'utf8');
  const updater = fs.readFileSync(path.join(__dirname, '..', 'src', 'bin', 'commandblock-updater.rs'), 'utf8');
  assert.match(update, /commandblock-updater\.exe/);
  assert.match(updater, /commandblock-updater\.exe/);
  assert.match(updater, /\.cmd/);
});

test('updater retries a stalled CDN transfer and resumes it with HTTP Range', () => {
  const update = fs.readFileSync(path.join(__dirname, '..', 'src', 'update.rs'), 'utf8');
  assert.match(update, /for attempt in 0\.\.3/);
  assert.match(update, /\.set\("Range"/);
  assert.match(update, /retry_delay/);
});
