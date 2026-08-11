# Orange Command-Block-Inspired Icon Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give Buff an original orange voxel command-block-inspired icon in its window, taskbar, and `buff.exe` Explorer entry.

**Architecture:** Store one transparent PNG source and a multi-resolution ICO derivative under `assets/`. A small runtime loader converts the embedded PNG into Winit RGBA data, while `build.rs` asks Windows' resource compiler to embed the ICO in the executable.

**Tech Stack:** Rust 2021, Winit 0.30, `image` PNG decoding, `winres` Windows resource compilation, Pillow for one-time ICO creation.

## Global Constraints

- Keep Buff's current Obsidian–purple UI and all existing behaviour unchanged.
- The icon must be original artwork: an orange voxel command-block-inspired cube, not a copied game asset.
- Keep a transparent background and include 16, 32, 48, 64, 128, and 256 pixel ICO entries.
- Change both the Winit window/taskbar icon and the Windows Explorer EXE resource.
- `C:\Codex` is not a Git repository; do not create commits for this work.

---

## File structure

- Create: `assets/buff-command-block.png` — transparent 1024px source artwork.
- Create: `assets/buff-command-block.ico` — multi-resolution Windows icon built from the PNG.
- Create: `build.rs` — Windows-only EXE icon resource configuration.
- Modify: `Cargo.toml` — add PNG decoder and Windows resource build dependency.
- Modify: `src/gui.rs:14,485,501-514` — embed and decode the icon for Winit.
- Modify: `Cargo.lock` — lock the two new Cargo dependencies.

### Task 1: Produce and validate the icon assets

**Files:**
- Create: `assets/buff-command-block.png`
- Create: `assets/buff-command-block.ico`

**Interfaces:**
- Produces: `assets/buff-command-block.png`, consumable through `include_bytes!("../assets/buff-command-block.png")`.
- Produces: `assets/buff-command-block.ico`, consumable by `winres::WindowsResource::set_icon`.

- [x] **Step 1: Generate the source artwork**

Create a 1024 × 1024 PNG with this exact visual brief: a centered, three-quarter isometric orange voxel cube; cream and pale-gold inset circuit panels; no text, logos, shadows, or copied art; an initially flat magenta chroma-key background.

- [x] **Step 2: Remove the chroma-key background and inspect it**

Run the provided helper against the generated image, then inspect the resulting PNG. The background must be transparent and the cube must have at least 15% clear padding on every side.

```powershell
python C:\Users\exocr\.codex\skills\.system\imagegen\scripts\remove_chroma_key.py <generated.png> assets\buff-command-block.png --color FF00FF
```

- [x] **Step 3: Build a multi-size ICO file**

Use Pillow to save the source image as an ICO containing 16, 32, 48, 64, 128, and 256 pixel frames.

```powershell
@'
from PIL import Image
source = Image.open("assets/buff-command-block.png").convert("RGBA")
source.save("assets/buff-command-block.ico", sizes=[(16,16), (32,32), (48,48), (64,64), (128,128), (256,256)])
'@ | python -
```

- [x] **Step 4: Verify both assets before code integration**

```powershell
@'
from PIL import Image
png = Image.open("assets/buff-command-block.png")
ico = Image.open("assets/buff-command-block.ico")
assert png.mode == "RGBA"
assert png.getbbox() is not None
assert ico.size == (256, 256)
print("Icon assets valid")
'@ | python -
```

Expected: `Icon assets valid`.

### Task 2: Embed the icon in the application and executable

**Files:**
- Create: `build.rs`
- Modify: `Cargo.toml`
- Modify: `src/gui.rs:14,485,501-514`
- Modify: `Cargo.lock`

**Interfaces:**
- Consumes: the PNG and ICO produced by Task 1.
- Produces: `fn build_icon() -> Option<winit::window::Icon>` and an EXE compiled with `assets/buff-command-block.ico` as its Windows resource.

- [x] **Step 1: Write the failing runtime-icon test**

Append this private test module to `src/gui.rs`:

```rust
#[cfg(test)]
mod icon_tests {
    use super::build_icon;

    #[test]
    fn embeds_a_valid_command_block_window_icon() {
        assert!(build_icon().is_some());
    }
}
```

- [x] **Step 2: Run the test to verify the current implementation fails**

```powershell
cargo test embeds_a_valid_command_block_window_icon
```

Expected: failure because `build_icon` still returns `Result<Icon, BadIcon>` rather than `Option<Icon>`.

- [x] **Step 3: Add the required Cargo dependencies**

Add the following entries to `Cargo.toml`:

```toml
[dependencies]
image = { version = "0.25", default-features = false, features = ["png"] }

[build-dependencies]
winres = "0.1"
```

Keep every existing dependency unchanged. Let Cargo update `Cargo.lock`.

- [x] **Step 4: Add the Windows resource compiler entry point**

Create `build.rs` with this complete content:

```rust
fn main() {
    println!("cargo:rerun-if-changed=assets/buff-command-block.ico");

    if std::env::var_os("CARGO_CFG_TARGET_OS").as_deref()
        == Some(std::ffi::OsStr::new("windows"))
    {
        let mut resource = winres::WindowsResource::new();
        resource.set_icon("assets/buff-command-block.ico");
        resource
            .compile()
            .expect("failed to embed the Buff executable icon");
    }
}
```

- [x] **Step 5: Replace the procedural purple icon loader**

Add `use image::GenericImageView;` beside the imports. Replace the old `build_icon` function and its `build_icon().ok()` caller with the following code:

```rust
        icon: build_icon(),
```

```rust
/// โหลดไอคอน Command Block สีส้มจาก PNG ที่ฝังมากับโปรแกรม
fn build_icon() -> Option<winit::window::Icon> {
    let image = image::load_from_memory_with_format(
        include_bytes!("../assets/buff-command-block.png"),
        image::ImageFormat::Png,
    )
    .ok()?;
    let (width, height) = image.dimensions();
    winit::window::Icon::from_rgba(image.to_rgba8().into_raw(), width, height).ok()
}
```

- [x] **Step 6: Run the focused test and full Rust test suite**

```powershell
cargo test embeds_a_valid_command_block_window_icon
cargo test
```

Expected: both commands pass.

### Task 3: Build and deliver the executable

**Files:**
- Modify: `target\release\buff.exe` (build output)
- Modify: `buff.exe` (delivered executable)

**Interfaces:**
- Consumes: all source and asset changes from Tasks 1–2.
- Produces: `buff.exe` with matching SHA-256 to the fresh release build.

- [x] **Step 1: Build the release executable**

```powershell
cargo build --release
```

Expected: success and `target\release\buff.exe` is created.

- [x] **Step 2: Replace the delivered executable only after it is closed**

```powershell
Copy-Item -LiteralPath target\release\buff.exe -Destination buff.exe -Force
```

If Windows reports that `buff.exe` is in use, ask the user to close Buff before rerunning this step.

- [x] **Step 3: Verify delivered binary identity**

```powershell
(Get-FileHash target\release\buff.exe -Algorithm SHA256).Hash
(Get-FileHash buff.exe -Algorithm SHA256).Hash
```

Expected: the two hashes are identical.

- [ ] **Step 4: Manually inspect the delivered icon**

Open `buff.exe` in Windows Explorer and launch it. Confirm the Explorer file icon and the window/taskbar icon both show the orange voxel cube.
