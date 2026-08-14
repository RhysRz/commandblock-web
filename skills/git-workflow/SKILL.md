---
name: git-workflow
description: Inspect Git state, prepare focused commits, and hand off changes safely. Use for status, diffs, branches, commits, pushes, pull requests, merges, or release handoff.
---

# Git Workflow

1. Inspect `git status`, branch, and diff before staging anything.
2. Stage only files belonging to the requested change; never absorb unrelated edits.
3. Use a concise conventional commit message that explains the user-visible change.
4. Run relevant validation before committing and report the resulting commit hash.
5. Push or create external pull requests only when authorized by the request.
