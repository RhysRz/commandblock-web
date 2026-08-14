# CommandBlock AI IDE Skills

## Goal

Provide reusable IDE workflows for both the desktop app and the web app through Desktop Connector. Skills must use the least privilege needed, inspect before editing, and verify every material change.

## Skill set

1. `project-planning` — inspect a codebase, clarify scope, write an implementation plan.
2. `codebase-navigation` — locate files, explain architecture, trace call paths and dependencies.
3. `code-implementation` — implement scoped changes, preserve existing work, and validate builds.
4. `debugging` — reproduce defects, collect evidence, isolate root causes, and fix with regression tests.
5. `testing-quality` — add/run unit, integration, browser, and regression tests.
6. `refactoring` — improve structure without changing behaviour, with tests before and after.
7. `git-workflow` — inspect status/diffs, create focused commits, and prepare safe handoffs.
8. `web-ui` — build responsive accessible interfaces and verify them in a browser.
9. `backend-api` — design and implement APIs, validation, error handling, and auth boundaries.
10. `database` — create safe migrations, query data, and protect data access rules.
11. `devops-release` — build, package, release, diagnose CI, and only offer verified updates.
12. `ide-security-review` — review secrets, auth, permissions, input handling, and dependency risk without replacing the existing global `security-review` skill.
13. `ide-performance` — measure bottlenecks, improve client/server performance, and verify results without replacing the existing global `performance` skill.
14. `documentation` — create and maintain README, setup guides, API docs, and changelogs.
15. `remote-workspace` — operate files, terminal commands, and previews through Desktop Connector with confirmation for destructive actions.

## Shared behaviour

- Treat every skill as a focused workflow, rather than a blanket permission grant.
- Prefer `rg` for discovery; inspect existing behaviour before edits.
- Never expose API keys, passwords, tokens, or private content in chat output.
- Require explicit confirmation for destructive operations, external publishing, payments, and credential changes.
- Run relevant validation before reporting success; show concise errors with a useful next action.

## Installation and discovery

Create each skill below the CommandBlock-discoverable skills directory, with a concise `SKILL.md` and UI metadata. The web Project Settings screen will discover them through Desktop Connector; users can enable only the skills needed for a project.

## Verification

Validate every skill manifest, confirm discovery through the existing skill scanner, and test representative prompts for navigation, implementation, debugging, web UI, database, release, and security.
