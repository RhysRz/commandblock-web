# Buff Minecraft Overworld UI Design

## Goal

ปรับหน้าตาแอป Buff ทุกแท็บให้เป็นธีม Minecraft Overworld โดยคงเลย์เอาต์ 3 คอลัมน์และพฤติกรรมเดิมทั้งหมดไว้ พร้อมสลับธีมกลางวันและกลางคืนตามเวลาท้องถิ่นของเครื่อง

## Scope

- ครอบคลุม activity bar, แชต, ประวัติสนทนา และแท็บ Queue, Files, Changes, Preview, Terminal และ Notes
- คงตำแหน่ง, `id`, event handler, HTTP endpoint และโครงสร้างข้อมูลที่มีอยู่
- ใช้ CSS และไอคอน/texture ที่สร้างในโค้ดเท่านั้น; ไม่เพิ่มรูป, ฟอนต์ หรือ asset จากภายนอก
- ไม่แก้ Rust backend, `config.json`, การเชื่อมต่อโมเดล หรือ API key

## Visual Language

หน้าตาใช้กรอบสี่เหลี่ยม, เส้นขอบหนา, เงา offset และลาย pixel เพื่อสื่อถึงบล็อกใน Minecraft. แชตเป็นพื้นที่ Overworld ที่มีชั้นท้องฟ้า หญ้า และดิน; history pane ใช้ชื่อและภาพลักษณ์ Quest Log; แผงทางขวาใช้ภาษาภาพแบบ inventory และปุ่มที่ใช้งานได้จะคล้ายช่อง item slot.

ธีมกลางวันใช้ท้องฟ้าฟ้า, หญ้าเขียว, ดินน้ำตาลและปุ่มสีทอง. ธีมกลางคืนใช้ท้องฟ้าน้ำเงินเข้ม, หญ้าเขียวเข้ม, ดินเข้มและ accent สีแสงจันทร์. ข้อความและพื้นที่อ่านโค้ดรักษาความต่างสีให้ชัดเจน.

## Theme Architecture

ไฟล์ `src/ui.html` เป็นจุดแก้ไขหลัก. เพิ่ม CSS custom properties (design tokens) สำหรับสี พื้นผิว ขอบ เงา และสถานะใน `:root`, `[data-theme="day"]` และ `[data-theme="night"]`. Selector ขององค์ประกอบเดิมใช้ตัวแปรเหล่านี้แทน palette เดิม เพื่อเปลี่ยนธีมทั่วหน้าโดยไม่แก้ markup หรือ JavaScript ของแต่ละแท็บ.

เพิ่มฟังก์ชันฝั่ง UI ที่คำนวณ theme จาก `new Date()` ดังนี้:

- `day`: 06:00 ถึง 17:59 ตามเวลาท้องถิ่นของ Windows
- `night`: 18:00 ถึง 05:59 ตามเวลาท้องถิ่นของ Windows

ฟังก์ชันกำหนด `document.body.dataset.theme`, อัปเดตไอคอน ☀/☾ และข้อความเวลาใน header. เรียกเมื่อโหลดหน้า และเรียกซ้ำทุก 60 วินาที. หากอ่านเวลาไม่ได้หรือเกิดข้อผิดพลาด ให้ใช้ `day` เป็นค่าเริ่มต้น. การสลับ theme ต้องไม่สร้างข้อความแชตใหม่, reset history หรือสลับแท็บที่เปิดอยู่.

## Component Mapping

| ส่วนเดิม | การแสดงผลใหม่ |
| --- | --- |
| Activity bar | เสาเครื่องมือแบบบล็อก; active state เป็นช่อง inventory สีทอง |
| Header และ logo | ป้ายไม้/ใบไม้พร้อมบล็อกหญ้า CSS; แสดงสถานะกลางวันหรือกลางคืน |
| Chat bubbles และ composer | กล่อง parchment/wood; composer เป็น command field ขอบบล็อก |
| History pane | Quest Log พร้อมรายการที่เลือกเป็นสีทอง |
| Queue / Files / Changes / Preview / Terminal / Notes | แท็บ inventory และ panel กรอบบล็อก โดยคงเนื้อหาและปุ่มเดิม |
| Scrollbar, code block, badges | แถบเลื่อนและป้ายแบบ pixel; โค้ดคง contrast ที่อ่านง่าย |

## Responsive Behavior

คง media query และกติกาหน้าจอแคบที่มีอยู่: เมื่อพื้นที่ไม่พอ แอปยังยุบหรือซ่อน panel ตามลอจิกเดิม. CSS ธีมใหม่ต้องไม่บังคับขนาดที่ทำให้ chat, terminal หรือ preview ล้นหน้าจอ.

## Verification

1. ใช้ `cargo build --release` ยืนยันว่าไฟล์ที่แก้ยัง build เป็น `buff.exe` ได้
2. เปิด UI และตรวจ day/night โดยเรียกฟังก์ชันสลับธีมใน developer tools หรือจำลองเวลา
3. ยืนยันว่าคลิก activity bar และแท็บ Queue, Files, Changes, Preview, Terminal และ Notes ได้ตามเดิม
4. ส่งข้อความแชตและตรวจว่า streaming, history และ model selector ยังทำงาน
5. ตรวจหน้าต่างแคบว่า layout ยุบแบบเดิมและไม่มี overflow ที่บดบัง composer

## Exclusions

ไม่มี theme switcher แบบ manual, ไม่มีการเปลี่ยนโครงสร้าง 3 คอลัมน์, ไม่มี asset ภายนอก, ไม่มีการแก้ backend และไม่มีการเปลี่ยนบริการ AI.
