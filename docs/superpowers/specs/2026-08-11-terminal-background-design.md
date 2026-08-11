# Buff Terminal Background Design

## Goal

ใช้สีพื้นหลังเดียวกับ Terminal กับทุก panel ของ Buff โดยคงปุ่ม กล่องข้อความ และ accent แบบ Minecraft Overworld ไว้เพื่อการแยกส่วนและการอ่านที่ชัดเจน.

## Scope

- กำหนดพื้นหลังของ chat pane, chat scroll area, history pane, right pane, modal และ panel เนื้อหาเป็น `var(--code-bg)`.
- ยกเลิกภาพพื้นหลังท้องฟ้า หญ้า และดินของ chat area.
- คงสีของ bubble, ปุ่ม, tab, กรอบ pixel และสถานะกลางวัน/กลางคืน.
- ไม่เปลี่ยน HTML, JavaScript, Rust backend, endpoint หรือ logic ของธีมตามเวลา.

## Implementation

เพิ่ม CSS overrides ในส่วน Minecraft Overworld theme ของ `src/ui.html`. Day และ night ใช้ค่า `--code-bg` ที่มีอยู่ของแต่ละธีม จึงยังมีความต่างตามเวลา แต่ทุกพื้นที่พื้นหลังจะสอดคล้องกับ Terminal ในธีมนั้น.

## Verification

- รัน `node --test tests/ui-theme.test.cjs` เพื่อยืนยันว่า logic day/night ยังทำงาน.
- รัน `cargo build --release` และคัดลอก executable ที่ build แล้วไปยัง `buff.exe`.
- เปิดแอป ตรวจว่า chat, history, ทุกแท็บ และ settings modal มีพื้นหลังเข้มเดียวกับ Terminal และข้อความยังอ่านได้.
