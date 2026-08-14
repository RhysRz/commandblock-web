---
name: ide-security-review
description: Review software for secrets exposure, unsafe permissions, authentication flaws, injection, and insecure defaults. Use for security audits, auth changes, remote access, payments, or sensitive data handling.
---

# Security Review

1. Identify assets, trust boundaries, identities, and external inputs.
2. Search for exposed keys, passwords, tokens, unsafe logging, and client-side secrets.
3. Check authorization separately from authentication for every protected action.
4. Validate input, escape output, constrain file paths, and use least privilege.
5. Report severity, evidence, practical remediation, and verification steps; never reveal a secret value.
