# Remote ICE UDP binding design

## Root cause

`PeerConnectionBuilder` from webrtc 0.20 defaults to an empty UDP bind-address list. CommandBlock constructed its Remote PC peer without calling `with_udp_addrs`, so no UDP socket was available for ICE candidates. The host consequently never produced an answer after approval.

## Change

Remote PC binds one ephemeral wildcard UDP socket (`0.0.0.0:0`) when creating its WebRTC peer. This follows the library's documented builder pattern; STUN remains unchanged and media/control traffic continues to use the direct WebRTC DataChannel.

## Verification

A Rust unit test asserts the exact ephemeral UDP binding configuration. Existing Remote approval and security tests remain in the regression suite.
