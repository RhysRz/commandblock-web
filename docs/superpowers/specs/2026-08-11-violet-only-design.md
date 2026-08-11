# Buff Obsidian Violet-Only Design

## Goal

กำจัดทุกสีและ CSS ของธีม Minecraft/Terminal legacy แล้วคงเฉพาะ Obsidian Liquid Glass palette ที่ใช้ม่วง, lavender, indigo และขาวอมม่วง.

## Scope

- ลบบล็อก CSS Minecraft Overworld และ Terminal-background override ทั้งหมด.
- แทนสถานะ success, warning และ error รวมถึง terminal dots ด้วยเฉดม่วง/indigo/lavender.
- ไม่เปลี่ยน Liquid Glass, aurora, floating tab tray, DOM, JavaScript หรือ backend.
- ไม่มีสีน้ำตาล, ทอง, เขียว, แดง, เหลือง หรือส้มในสีที่ UI ใช้งาน.

## Palette

| Role | Value |
| --- | --- |
| Obsidian surface | `#09090f` / `#0c0b13` |
| Violet primary | `#9a68ff` |
| Lavender highlight | `#c28cff` |
| Indigo depth | `#6940cd` |
| Status soft | `#b899ff` |
| Status bright | `#dfc6ff` |
| Text | `#f4f1ff` / `#aaa6bd` |

## Verification

- Search the active CSS layer for legacy palette names and brown/gold/green/red/yellow/orange values; none remain.
- Run JavaScript syntax parsing, `cargo test`, and `cargo build --release`.
- Replace `buff.exe` with the verified release output after it is closed; SHA-256 hashes must match.
