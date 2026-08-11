# Orange command-block-inspired application icon

## Goal

Replace Buff's purple placeholder application icon with a polished, original orange voxel command-block-inspired icon.  The same asset must appear in the application window and taskbar and as the icon of `buff.exe` in Windows Explorer.

## Visual direction

- A centered, three-quarter isometric orange voxel cube.
- Cream and pale-gold inset circuit panels make the cube recognisable at small sizes.
- Transparent background, no text, no brand marks, and no copied game artwork.
- Crisp silhouette and high contrast so the icon remains legible at 16, 32, 48, and 256 pixels.

## Technical approach

1. Generate an original square PNG asset, remove its chroma-key background, and keep it under `assets/`.
2. Create a multi-resolution `.ico` file from that PNG for the Windows executable resource.
3. Add a Windows build resource so the generated `buff.exe` carries the icon visible in Explorer.
4. Load the PNG at compile time for the Winit window icon, which also feeds the taskbar icon.
5. Add a focused validity test for the embedded window icon and verify it through a release build.

## Acceptance criteria

- `buff.exe` contains the orange voxel icon in Windows Explorer after rebuild.
- The app window and taskbar use the matching image.
- The icon is transparent, original, and legible at common Windows sizes.
- Existing application behavior and the current Obsidian–purple UI are unaffected.
