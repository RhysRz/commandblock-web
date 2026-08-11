const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

test('auth module exposes email, reset, and logout actions without paid social login', () => {
  const source = fs.readFileSync(path.join(__dirname, '..', 'web', 'js', 'auth.js'), 'utf8');
  for (const name of ['signUp', 'signIn', 'sendPasswordReset', 'signOut']) {
    assert.match(source, new RegExp(`async function ${name}`));
  }
  assert.doesNotMatch(source, /signInWithGoogle/);
  assert.match(source, /onAuthStateChange/);
});
