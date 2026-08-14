---
name: refactoring
description: Improve code structure without changing behavior. Use when simplifying code, reducing duplication, extracting components, renaming, or reorganizing modules.
---

# Refactoring

1. Establish existing behavior with tests or a repeatable check before changing structure.
2. Keep each refactor small: one responsibility, one rename, or one extraction at a time.
3. Preserve public interfaces unless the request explicitly changes them.
4. Avoid unrelated formatting churn and preserve user changes in the worktree.
5. Run the same verification after each meaningful refactor.
