# Update checksum retry design

## Root cause

The update archive download has resumable retry logic, but the subsequent download of its SHA-256 checksum made one network request only. A transient GitHub CDN disconnect while reading that small checksum file returns `Unexpected EOF`, so a complete archive cannot be verified or staged.

## Design

The checksum fetch will use the same bounded three-attempt policy as the archive. It will retry only transient fetch failures, with the existing one- then two-second delays. The checksum value remains mandatory: after it arrives, the existing SHA-256 comparison must still match before any archive is extracted or installed.

## Safety and test

A small generic retry helper is exercised with an operation that fails twice then succeeds, proving the third attempt is used. The actual checksum fetch calls that helper; no update can bypass checksum validation.
