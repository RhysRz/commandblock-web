# Buff Obsidian Liquid Glass Design

## Goal

แทนที่ธีม Minecraft และ Terminal ที่มีอยู่ด้วยธีม developer ระดับพรีเมียมแบบ Obsidian Liquid Glass: พื้นมืด, accent ม่วง futuristic, aurora animation ช้า และ panel โปร่งแบบ glass.

## Scope

- คงโครง 3 คอลัมน์, ทุก `id`, event handler, endpoint และพฤติกรรมของ UI เดิม.
- ใช้ Liquid Glass กับ activity rail, chat header, history, tab bar, tool panes, popup menu, modal และ toast.
- ให้ Terminal, code block, file view, notes editor และช่องพิมพ์เป็นพื้นทึบเพื่อ contrast.
- ไม่มีภาพ ฟอนต์ หรือ dependency ภายนอก.
- ใช้ animation aurora ม่วง–น้ำเงินความยาว 18 วินาที; ปิด animation เมื่อ `prefers-reduced-motion: reduce`.

## Visual System

พื้นฐานคือ obsidian `#09090f` พร้อม radial gradients ม่วง/น้ำเงินที่เลื่อนช้าอยู่ด้านหลัง. Glass surface ใช้พื้น `rgba(20, 18, 32, .55-.75)`, `backdrop-filter: blur(20px)`, ขอบขาวโปร่งและ inner highlight บาง. Accent หลักเป็นม่วง `#a775ff`; action สำคัญใช้ gradient ม่วงเข้มไปม่วงสว่าง. ข้อความใช้ขาวอมม่วงและ muted lavender.

## Component Rules

| Component | Appearance |
| --- | --- |
| Main background | Obsidian + aurora pseudo-element + optional fine grain |
| Navigation, history, panels, modal | Glass card, subtle border, restrained shadow |
| Active tab and rail item | violet fill and soft glow |
| Assistant message | translucent dark lavender glass |
| User message | opaque violet gradient for clear separation |
| Terminal, code, file view, notes, composer | opaque near-black surfaces |
| Buttons and model menu | glass at rest; violet gradient for primary action |

## Removal of Prior Theme

Remove the Minecraft Overworld CSS block, the Terminal-background override, `theme-status` markup, `themeForHour`, `applyThemeForHour`, `refreshTheme`, and the timed theme interval. Restore the existing clock to update independently every 30 seconds. Remove `tests/ui-theme.test.cjs`, because it tests the removed timed-theme feature.

## Accessibility and Resilience

Maintain visible keyboard focus, minimum readable text contrast, and current responsive collapse behavior. The aurora must be decorative and non-interactive, positioned behind all UI, and disabled for reduced-motion users. Any browser that does not support `backdrop-filter` still shows a sufficiently opaque dark panel.

## Verification

1. Run `cargo test` and `cargo build --release`.
2. Check embedded JavaScript syntax after removing timed-theme helpers.
3. Launch the rebuilt executable and inspect all tabs, model menu, settings modal, chat composer, terminal, code blocks and narrow layout.
4. Copy the verified release binary to `C:\\Codex\\buff.exe` after confirming Buff is closed and compare SHA-256 hashes.
