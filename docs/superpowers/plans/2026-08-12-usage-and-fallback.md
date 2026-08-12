# Usage and Fallback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make model spend understandable without exposing secrets, and recover from transient provider failures through user-configured fallbacks.

**Architecture:** The LLM stream extracts provider usage from its final chunks and returns it through `StreamedResult`. A local `usage` module owns exact/estimated counters and local preferences. Desktop exposes it through fixed local APIs; CommandBlock Web keeps its copy in browser local storage.

**Tech Stack:** Rust, serde_json, local HTTP UI, JavaScript, browser local storage.

## Global Constraints

- Credit is user-entered local data, not provider account data.
- The DeepSeek top-up URL is exactly `https://platform.deepseek.com/top_up`.
- Every token metric declares `exact` or `estimated`.
- Fallback never handles 401, invalid configuration, or malformed request errors.

---

### Task 1: Parse and preserve stream usage

**Files:**
- Modify: `src/llm.rs`, `src/main.rs`, `src/gui.rs`
- Test: `src/llm.rs`

**Interfaces:**
- Produces `TokenUsage { prompt_tokens, completion_tokens, total_tokens, exact }` and `StreamedResult { usage: Option<TokenUsage>, .. }`.

- [ ] **Step 1: Write failing parser tests**

```rust
#[test]
fn usage_from_final_stream_event_is_exact() {
    let usage = parse_usage(&json!({"usage":{"prompt_tokens":12,"completion_tokens":8,"total_tokens":20}}));
    assert_eq!(usage.unwrap().total_tokens, 20);
}
```

- [ ] **Step 2: Run it**

Run: `cargo test llm::tests::usage_from_final_stream_event_is_exact`

Expected: FAIL because `parse_usage` does not exist.

- [ ] **Step 3: Implement usage parsing**

Read `usage` before skipping chunks with no `choices`; retain the last valid usage object. When absent, calculate an explicitly estimated count from UTF-8 characters divided by four for user-visible counters only.

- [ ] **Step 4: Run focused tests**

Run: `cargo test llm::tests`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add src/llm.rs src/main.rs src/gui.rs && git commit -m "feat(usage): surface provider token usage"`

### Task 2: Add local credit and token counters to desktop

**Files:**
- Create: `src/usage.rs`
- Modify: `src/gui.rs`, `src/ui.html`, `src/main.rs`
- Test: `src/usage.rs`, `tests/usage-controls.test.cjs`

**Interfaces:**
- Produces `UsageStore::{record, snapshot, update_preferences}` and endpoints `GET/POST /api/usage`.

- [ ] **Step 1: Write failing counter/UI tests**

```rust
#[test]
fn usage_rolls_into_session_and_today_totals() {
    let mut store = UsageStore::new_for_test();
    store.record(TokenUsage::exact(10, 5));
    assert_eq!(store.snapshot().session.total_tokens, 15);
}
```

```js
assert.match(ui, /id="usageCredit"/);
assert.match(ui, /id="usageTopUp"/);
assert.match(ui, /https:\/\/platform\.deepseek\.com\/top_up/);
assert.match(ui, /\/api\/usage/);
```

- [ ] **Step 2: Run tests**

Run: `cargo test usage::tests; node --test tests/usage-controls.test.cjs`

Expected: FAIL because no store/UI exists.

- [ ] **Step 3: Implement desktop store and controls**

Persist `{credit_usd, thb_per_usd, daily_date, daily_usage}` in a local `.freebuff/usage.json` beside existing settings. Render compact `$x.xx · ≈ ฿y` button, `+ เติมเงิน` external link, and an edit dialog. Expose latest, session, and day counters without returning any API key or message body.

- [ ] **Step 4: Re-run tests**

Run: `cargo test usage::tests; node --test tests/usage-controls.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add src/usage.rs src/gui.rs src/ui.html src/main.rs tests/usage-controls.test.cjs && git commit -m "feat(usage): add local credit and token dashboard"`

### Task 3: Add browser usage controls and model fallback

**Files:**
- Modify: `web/cloud-adapter.js`, `src/config.rs`, `src/llm.rs`, `src/gui.rs`, `src/ui.html`
- Test: `tests/cloud-usage-contract.test.cjs`, `src/llm.rs`, `src/config.rs`

**Interfaces:**
- Produces `FallbackModel { model, base_url, api_key }`, `should_try_fallback(status: u16) -> bool`, and browser key `commandblock-usage-v1`.

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn only_retryable_or_billing_statuses_use_fallback() {
    assert!(should_try_fallback(429));
    assert!(should_try_fallback(402));
    assert!(!should_try_fallback(401));
}
```

```js
assert.match(adapter, /commandblock-usage-v1/);
assert.match(adapter, /platform\.deepseek\.com\/top_up/);
assert.doesNotMatch(adapter, /insert\([^)]*(apiKey|usage)/);
```

- [ ] **Step 2: Run tests**

Run: `cargo test llm::tests::only_retryable_or_billing_statuses_use_fallback; node --test tests/cloud-usage-contract.test.cjs`

Expected: FAIL because fallback predicate and web counters do not exist.

- [ ] **Step 3: Implement minimal fallback and web rendering**

Use configured model entries after the active model in order. Retry once per distinct fallback, report `ใช้ fallback: <model>` to the UI, and retain the original error if all fail. Browser storage contains only numeric preferences/counters, not key or message data; it renders the same USD/THB/top-up and exact/estimated token labels.

- [ ] **Step 4: Run all related tests**

Run: `cargo test; node --test tests/cloud-usage-contract.test.cjs tests/cloud-proxy-safety.test.cjs`

Expected: PASS.

- [ ] **Step 5: Commit**

Run: `git add web/cloud-adapter.js src/config.rs src/llm.rs src/gui.rs src/ui.html tests/cloud-usage-contract.test.cjs && git commit -m "feat(models): add usage display and safe fallback"`
