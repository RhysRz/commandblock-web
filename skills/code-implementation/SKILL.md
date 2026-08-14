---
name: code-implementation
description: Implement scoped software changes safely. Use when asked to add, modify, or repair code, configuration, or application behavior.
---

# Code Implementation

1. Inspect the relevant code and current Git diff before editing.
2. Define the smallest change that meets the requested behavior.
3. Add or update a focused regression test before implementation when practical.
4. Use existing patterns, preserve unrelated user edits, and avoid rewriting generated files.
5. Run the narrowest relevant test, then build or run the application when risk warrants it.
