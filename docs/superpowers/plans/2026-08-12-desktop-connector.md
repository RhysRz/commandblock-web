# Commandblock Desktop Connector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect Commandblock Web to an authenticated Commandblock desktop process through Supabase without opening inbound ports.

**Architecture:** The web queues user-scoped, device-scoped commands in Supabase. `Commandblock.exe --connector` authenticates for the current process, polls commands over HTTPS, confines all paths to a chosen root, and posts results. The web adapts existing pane API calls to this protocol.

**Tech Stack:** Rust + ureq + serde, Supabase Postgres/RLS REST API, browser JavaScript.

## Global Constraints

- Connector sessions and provider keys stay in memory only.
- Commands must be owner-scoped, device-scoped, and root-confined.
- Terminal execution needs local approval and has a fixed timeout.
- Do not use service-role keys, inbound ports, or a public filesystem endpoint.

---

### Task 1: Define relay schema and web command contract

**Files:**
- Create: `supabase/migrations/202608120002_connector.sql`
- Create: `tests/connector-schema-contract.test.cjs`
- Modify: `web/cloud-adapter.js`

**Interfaces:**
- `connector_devices(id, user_id, name, root_name, last_seen_at)`.
- `connector_commands(id, user_id, device_id, action, payload, status, result)`.
- `requestConnector(action, payload): Promise<object>` returns a completed result or an explicit connector error.

- [ ] Write a failing SQL contract test for RLS, indexes, and device-targeted commands.
- [ ] Implement tables, ownership policies, status check constraints, and partial queue index.
- [ ] Write a failing web contract for `requestConnector` and selected-device session state.
- [ ] Implement command creation, bounded status polling, device selection, and offline errors.
- [ ] Run Node contracts and commit.

### Task 2: Implement Commandblock connector mode

**Files:**
- Create: `src/connector.rs`
- Modify: `src/main.rs`
- Create: `tests/connector_path_tests.rs`

**Interfaces:**
- `connector::run(agent: ureq::Agent) -> Result<(), String>`.
- `connector::safe_child(root: &Path, requested: &str) -> Result<PathBuf, String>`.
- `connector::execute(action: &str, payload: &Value, root: &Path) -> Value`.

- [ ] Write failing path-confinement tests for traversal and absolute paths.
- [ ] Implement root selection, password-based Supabase session acquisition for the current process, heartbeat, polling, and lifecycle updates.
- [ ] Implement files/read/queue/changes/pick-folder and approval-gated exec actions.
- [ ] Add `--connector` routing and help text.
- [ ] Run Rust tests and commit.

### Task 3: Connect the existing web panes and document deployment

**Files:**
- Modify: `web/cloud-adapter.js`
- Modify: `README.md`
- Modify: `tests/cloud-adapter-contract.test.cjs`

- [ ] Write failing contracts for connector-backed file/terminal requests.
- [ ] Route existing pane endpoints through `requestConnector` when an active device is selected.
- [ ] Add a connected-device selector and truthful states for offline/no connector.
- [ ] Document one-command startup and Supabase migration application.
- [ ] Run full Node/Rust verification, deploy web, and commit.
