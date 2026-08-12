# Update curl fallback design

## Evidence

The in-app `ureq` downloader failed three times with `Unexpected EOF` for the GitHub release ZIP. On the same Windows PC, `curl.exe --fail --location --retry 3 --retry-all-errors` downloaded the same 5,201,391-byte file successfully and produced the release's published SHA-256.

## Design

Keep the existing resumable `ureq` download as the primary path. If all three primary attempts fail, invoke the Windows-provided `curl.exe` as a hidden child process with redirect following and its own retry policy. Read its temporary output back into memory, validate its byte count when release metadata provides one, and then run the existing mandatory SHA-256 validation before staging files.

## Failure handling

If curl is missing, fails, or returns an incorrectly sized result, show both the primary and fallback failures. No executable is extracted or installed until the existing checksum comparison succeeds.
