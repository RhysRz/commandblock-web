---
name: testing-quality
description: Design and run focused software tests. Use when adding tests, investigating test failures, creating regression coverage, or verifying a change before delivery.
---

# Testing Quality

1. Identify the smallest observable behavior that must hold.
2. Prefer deterministic unit tests; add integration or browser tests for real boundaries.
3. Make a regression test fail before changing production code when fixing a defect.
4. Run the narrow test first, then the relevant suite and build checks.
5. Report commands run, pass/fail results, and coverage limitations.
