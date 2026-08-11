# Header Command-Block Icon Design

## Goal

Replace the robot emoji in Buff's upper-left brand mark with the same original orange voxel command-block-inspired image used by `buff.exe`.

## Design

- Keep the existing compact rounded-square glass logo container, position, text, and header controls unchanged.
- Display the PNG with `object-fit: contain`, preserving its transparent background and a small clear inset.
- Serve the existing `assets/buff-command-block.png` from Buff's local HTTP server at a fixed `/assets/buff-command-block.png` path.
- Do not duplicate or base64-embed the image in `src/ui.html`; the executable and header therefore share one source asset.

## Acceptance criteria

- The robot emoji no longer appears in the top-left logo.
- The orange cube is visible, uncropped, and legible in the existing 38px header logo.
- Existing UI layout and server routes continue to behave unchanged.
