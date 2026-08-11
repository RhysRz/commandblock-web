const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const html = fs.readFileSync(path.join(__dirname, '..', 'src', 'ui.html'), 'utf8');

test('header logo uses the shared orange command-block image', () => {
  assert.match(
    html,
    /<div class="logo">\s*<img src="\/assets\/buff-command-block\.png" alt="Commandblock Command Block">\s*<\/div>/,
  );
  assert.match(html, /\.logo img\s*\{[^}]*object-fit:\s*contain/);
});
