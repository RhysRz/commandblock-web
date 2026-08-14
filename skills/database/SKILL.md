---
name: database
description: Work safely with database schemas, queries, migrations, and access rules. Use for SQL, data modeling, Supabase, Postgres, indexes, migrations, or row-level security.
---

# Database

1. Inspect the current schema, constraints, and access policies before writing a migration.
2. Keep migrations additive and reversible where practical; separate schema changes from data backfills.
3. Enable least-privilege access rules and test with both allowed and denied users.
4. Use parameters for values; never interpolate untrusted input into SQL.
5. Verify migration output and query plans on representative data before claiming success.
