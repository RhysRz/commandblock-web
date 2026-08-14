---
name: backend-api
description: Design and implement reliable backend APIs. Use for HTTP endpoints, request validation, authentication, business logic, streaming, webhooks, or service integrations.
---

# Backend API

1. Define request, response, error, and authorization contracts before implementation.
2. Validate all external input at the boundary and return stable error shapes.
3. Keep secrets server-side; never send provider keys to untrusted clients.
4. Make mutations idempotent or explicitly protect against duplicate requests when needed.
5. Test success, validation, permission, and provider-failure paths.
