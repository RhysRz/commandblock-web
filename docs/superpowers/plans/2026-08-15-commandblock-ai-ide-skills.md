# CommandBlock AI IDE Skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a complete, discoverable AI IDE skill suite while retaining all existing EXE-discovered skills.

**Architecture:** Store CommandBlock-owned skills under `C:\Codex\skills\<skill-name>\SKILL.md`. The existing scanner already searches that directory before `~\.agents\skills` and exposes both sources in desktop Project Settings and web Project Settings through Desktop Connector. CommandBlock-owned names must not collide with existing EXE skills.

**Tech Stack:** Markdown SKILL.md files, CommandBlock Rust skill scanner, existing Project Settings UI.

## Global Constraints

- Use lowercase hyphenated skill folder names.
- Keep every SKILL.md concise and task-specific.
- Preserve externally installed skills; never copy, overwrite, or delete them.
- Require inspection and relevant verification before reporting a completed task.
- Do not include credentials or secrets in a skill.

---

### Task 1: Verify the existing discovery boundary

**Files:**
- Inspect: `src/tools.rs:1345-1361`
- Inspect: `src/gui.rs:1098-1105`

**Interfaces:**
- Consumes: `skill_dirs()` and `list_skills_structured()`.
- Produces: A verified installation target of `C:\Codex\skills`.

- [ ] **Step 1: Inspect discovery roots**

Run: `rg -n -C 4 'fn skill_dirs|list_skills_structured' src/tools.rs src/gui.rs`

Expected: Project `skills` appears before user-level `.agents/skills`.

- [ ] **Step 2: Record the non-duplication rule**

Confirm the new CommandBlock-owned names do not collide with any existing EXE-discovered skill before returning cards to Project Settings.

### Task 2: Create core engineering workflows

**Files:**
- Create: `skills/project-planning/SKILL.md`
- Create: `skills/codebase-navigation/SKILL.md`
- Create: `skills/code-implementation/SKILL.md`
- Create: `skills/debugging/SKILL.md`
- Create: `skills/testing-quality/SKILL.md`
- Create: `skills/refactoring/SKILL.md`
- Create: `skills/git-workflow/SKILL.md`

**Interfaces:**
- Consumes: CommandBlock file, terminal, search, and change-tracking tools.
- Produces: Reusable workflows for planning, analysis, implementation, debugging, testing, refactoring, and Git.

- [ ] **Step 1: Create failing discovery check**

Run: `Test-Path skills\project-planning\SKILL.md`

Expected: `False` before the suite exists.

- [ ] **Step 2: Create each core SKILL.md**

Include `name` and `description` frontmatter, then imperative instructions covering inspect → act → verify.

- [ ] **Step 3: Validate metadata**

Run: `Get-ChildItem skills -Directory | ForEach-Object { Test-Path "$($_.FullName)\SKILL.md" }`

Expected: Every created skill returns `True`.

### Task 3: Create product and platform workflows

**Files:**
- Create: `skills/web-ui/SKILL.md`
- Create: `skills/backend-api/SKILL.md`
- Create: `skills/database/SKILL.md`
- Create: `skills/devops-release/SKILL.md`
- Create: `skills/ide-security-review/SKILL.md`
- Create: `skills/ide-performance/SKILL.md`
- Create: `skills/documentation/SKILL.md`
- Create: `skills/remote-workspace/SKILL.md`

**Interfaces:**
- Consumes: Desktop Connector commands, browser preview, project files, and release tooling.
- Produces: Web, API, database, release, security, performance, documentation, and remote-workspace workflows.

- [ ] **Step 1: Create each platform SKILL.md**

Use the same frontmatter shape and state explicit safety boundaries for data migration, release, external publishing, and destructive commands.

- [ ] **Step 2: Validate all generated skill names**

Run: `Get-ChildItem skills -Directory | ForEach-Object { Select-String -Path "$($_.FullName)\SKILL.md" -Pattern '^name: [a-z0-9-]+$' }`

Expected: One valid `name:` line per new skill.

### Task 4: Verify merged EXE discovery and publish

**Files:**
- Inspect: `src/tools.rs`
- Modify: `README.md`

**Interfaces:**
- Consumes: `C:\Codex\skills` plus `~\.agents\skills`.
- Produces: A documented suite that appears alongside existing EXE skills without duplicates.

- [ ] **Step 1: Run the skill listing**

Run: `Commandblock.exe /skills` or the existing skill-listing API.

Expected: New AI IDE skills and pre-existing EXE skills appear together.

- [ ] **Step 2: Add a concise README note**

Document the CommandBlock-owned AI IDE suite and explain that existing user/exe skills remain available.

- [ ] **Step 3: Run regression checks**

Run: `cargo test` and `git diff --check`.

Expected: All tests pass and no whitespace errors are reported.

- [ ] **Step 4: Commit and push**

Run: `git add skills README.md && git commit -m "feat: add AI IDE skills suite" && git push origin main`
