---
name: codebase-navigation
description: Explore a codebase, locate relevant files, explain architecture, and trace dependencies. Use when asked where code lives, how a feature works, or what a project contains.
---

# Codebase Navigation

1. Start with `rg --files`, then search identifiers with `rg` rather than scanning unrelated files.
2. Read entry points, configuration, and call sites around the requested feature.
3. Map data flow as input → transformation → output; include ownership and dependencies.
4. Report exact file paths and symbols. Distinguish observed facts from inferences.
5. Do not edit files unless the request includes a change.
