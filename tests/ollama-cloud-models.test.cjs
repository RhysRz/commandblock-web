const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const config = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'config.json'), 'utf8'));
const endpoint = 'http://localhost:11434/v1';

for (const name of ['glm-5.2:cloud', 'minimax-m3:cloud']) {
  test(`${name} is configured as an Ollama Cloud model`, () => {
    const entries = config.models.filter((entry) => entry.model === name);
    assert.equal(entries.length, 1);
    assert.equal(entries[0].base_url, endpoint);
    assert.equal(entries[0].api_key, '');
  });
}
