# Remote ICE UDP Binding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give Remote PC a local UDP socket so WebRTC ICE can produce an answer after approval.

**Architecture:** A focused helper supplies the wildcard ephemeral `SocketAddr`; `run_peer` passes it to the existing `PeerConnectionBuilder` with `with_udp_addrs`.

**Tech Stack:** Rust, webrtc 0.20, Cargo unit tests.

## Global Constraints

- Bind only an ephemeral UDP port; never reserve a fixed network port.
- Do not alter the approval-code or WebRTC DataChannel security boundary.
- Keep the existing STUN server configuration.

---

### Task 1: Bind a UDP socket for Remote ICE

**Files:**
- Modify: `src/remote.rs:405-450`
- Test: `src/remote.rs` unit-test module

**Interfaces:**
- Produces: `fn remote_udp_bind_addrs() -> Vec<std::net::SocketAddr>`

- [x] **Step 1: Write failing test** — asserts `0.0.0.0:0` is the Remote ICE bind address.
- [x] **Step 2: Verify red** — test failed because the helper did not exist.
- [x] **Step 3: Implement** — pass the helper result to `PeerConnectionBuilder::with_udp_addrs`.
- [x] **Step 4: Verify green** — targeted Cargo test passes.
- [ ] **Step 5: Run full verification and commit** — run library tests, Node contracts, and release build.
