# Reliable Update and Remote Design

## Goal

Make Windows update downloads recover from transient GitHub Release CDN failures, and make Remote PC explain P2P connectivity limits without requiring paid infrastructure.

## Update downloads

The updater retains partial ZIP bytes in the CommandBlock updates directory. A failed transfer retries three times with a short increasing delay and resumes using HTTP Range when the CDN supports it. Each retry reports a Thai status message and preserves checksum validation before extraction. It does not add a second hosting provider or reduce integrity checks.

## Remote PC

Remote PC continues direct WebRTC and the existing one-time device approval code. It shows an actionable relay hint when ICE fails, explaining that a TURN server is optional and is not bundled because reliable TURN requires infrastructure.

## Tests

Tests cover retry helpers, Range-request resume behavior, and actionable Remote failure copy. Existing update, Remote, checksum, and Rust test suites remain required.
