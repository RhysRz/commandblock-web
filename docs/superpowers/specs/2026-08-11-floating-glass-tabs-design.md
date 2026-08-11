# Buff Floating Glass Tabs Design

## Goal

เปลี่ยนแท็บด้านบนของ right pane จากแท็บติดขอบเป็น floating glass tab tray ที่เข้ากับธีม Obsidian Liquid Glass.

## Scope

- คงชื่อแท็บ, `data-tab`, event listener และการเลื่อนแนวนอนเดิม.
- ใช้ tray สีดำโปร่งขอบมน วางห่างจากขอบ panel เล็กน้อย.
- แต่ละแท็บเป็น pill โปร่ง; hover สว่างขึ้นเล็กน้อย.
- แท็บ active ใช้ violet gradient, ขอบม่วงอ่อน และ soft glow.
- ไม่มีการเปลี่ยน HTML หรือ JavaScript; เป็น CSS-only.

## Responsive and Accessibility

tray ต้องเลื่อนแนวนอนได้เมื่อพื้นที่ไม่พอ และ focus-visible ของปุ่มยังมองเห็น. สี active ต้องแตกต่างจาก inactive อย่างชัดเจน.

## Verification

- รัน `cargo test` และ `cargo build --release`.
- เปิด app ตรวจคลิก Queue, Files, Changes, Preview, Terminal และ Notes; ต้องเปลี่ยน pane ตามเดิม.
- ตรวจหน้าต่างแคบว่าแถบเลื่อนได้และแท็บไม่ถูกตัด.
