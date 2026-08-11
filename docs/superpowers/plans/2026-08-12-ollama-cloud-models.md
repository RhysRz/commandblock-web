# Ollama Cloud Models Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add GLM 5.2 and MiniMax M3 Ollama Cloud models to the Commandblock model selector.

**Architecture:** Update only the existing JSON `models` array, preserving every top-level setting and prior model entry. A Node test parses the local file and validates model names, endpoint, blank model keys, and uniqueness without printing secret values.

**Tech Stack:** Existing `config.json`, Node built-in test runner, PowerShell JSON handling.

## Global Constraints

- Add `glm-5.2:cloud` and `minimax-m3:cloud` exactly once.
- Use `http://localhost:11434/v1` and an empty model-specific API key for both.
- Preserve all existing settings, models, and secret values without printing them.
- No source-code or executable changes are required.

---

### Task 1: Add and verify the Ollama Cloud entries

**Files:**
- Create: `tests/ollama-cloud-models.test.cjs`
- Modify: `config.json`

**Interfaces:**
- Consumes: `config.json.models` entries in `{ model, base_url, api_key }` form.
- Produces: exactly one configured entry for each requested Ollama Cloud model.

- [x] **Step 1: Write the failing configuration test**

Create `tests/ollama-cloud-models.test.cjs`:

```js
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
```

- [x] **Step 2: Run it to verify both missing entries fail**

```powershell
node --test tests\ollama-cloud-models.test.cjs
```

Expected: two failing tests with zero matching entries.

- [x] **Step 3: Add missing entries without rewriting existing values**

```powershell
$config = Get-Content config.json -Raw | ConvertFrom-Json
$names = 'glm-5.2:cloud', 'minimax-m3:cloud'
foreach ($name in $names) {
  if (-not ($config.models | Where-Object { $_.model -eq $name })) {
    $config.models += [pscustomobject]@{ model = $name; base_url = 'http://localhost:11434/v1'; api_key = '' }
  }
}
$config | ConvertTo-Json -Depth 10 | Set-Content config.json -Encoding utf8
```

- [x] **Step 4: Run the targeted test and list only safe metadata**

```powershell
node --test tests\ollama-cloud-models.test.cjs
$config = Get-Content config.json -Raw | ConvertFrom-Json
$config.models | Select-Object model, base_url | Format-Table -AutoSize
```

Expected: both tests pass; the listing shows model names and endpoints only.
