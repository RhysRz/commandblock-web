# Update Freshness Design

## Goal

Show the CommandBlock update card only when GitHub has a newer runtime build.

## Decision

Each desktop build exposes two immutable values:

- `COMMAND_BLOCK_BUILD_ID`: the latest Git revision that changed runtime inputs.
- `COMMAND_BLOCK_BUILD_TIMESTAMP`: the UTC build time.

The updater accepts a release only when both conditions hold: its `build-<id>` tag differs from the installed runtime ID and its GitHub `published_at` timestamp is later than the installed build timestamp. The release workflow reads the ID from the compiled executable, so installer-only commits reuse the runtime ID and do not create a second runtime release.

## Error Handling

Malformed or missing publication timestamps are treated as a check error rather than offering an unverified update. A matching runtime tag remains immediately up to date without depending on a network timestamp comparison.

## Tests

Unit tests cover matching tags and timestamp ordering. Static workflow tests assert that release tags originate from the compiled runtime ID and duplicate tags are skipped.
