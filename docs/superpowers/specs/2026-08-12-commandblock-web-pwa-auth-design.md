# Commandblock Web/PWA Authentication Design

## Goal

สร้าง Commandblock เวอร์ชันเว็บ/PWA ที่สมัครและเข้าสู่ระบบจากคนละเครื่องได้ ใช้ได้โดยตรงบนมือถือและ Remote PC โดยผู้ใช้ใส่ Cloud API key ของตนเอง

## Scope

โครงการนี้ครอบคลุมเว็บ/PWA และระบบบัญชีออนไลน์เท่านั้น ส่วน Desktop Connector สำหรับอ่านไฟล์หรือสั่ง Terminal ของ PC ระยะไกลเป็นโครงการถัดไป

## Architecture

- **Frontend:** เว็บ responsive และ PWA ที่แยกจากหน้าต่าง Rust เดิม แต่ใช้ภาษาและธีม Obsidian–ม่วงของ Commandblock เดิม
- **Authentication and data:** Supabase Auth และ Postgres บน Free tier
- **AI proxy:** Supabase Edge Function รับคำขอที่ยืนยัน session แล้วส่งต่อไปยังผู้ให้บริการ OpenAI-compatible เช่น DeepSeek
- **Hosting:** GitHub Pages สำหรับ static frontend; Supabase เป็น backend
- **Source control:** GitHub repository แบบ public โดยไม่มี API key, session หรือไฟล์ build ใน repository

## Authentication

- รองรับ Email/Password
- เปิดให้สมัครได้ทันที
- ส่งอีเมลยืนยันบัญชีและรองรับ reset password; ไม่ใช้ Google Sign-in เพื่อหลีกเลี่ยงการผูก Billing ของ Google Cloud
- ผู้ใช้ที่ยังไม่เข้าสู่ระบบจะเห็นหน้า Welcome / Login / Register ก่อนหน้าแชท
- หลัง login สำเร็จ จะสร้าง profile หนึ่งรายการและพาไปหน้าแชท

## Data Model and Access Rules

- `profiles`: `id`, `display_name`, `avatar_url`, `created_at`
- `conversations`: `id`, `user_id`, `title`, `model_id`, `created_at`, `updated_at`
- `messages`: `id`, `conversation_id`, `user_id`, `role`, `content`, `created_at`
- เปิด Row Level Security กับทุกตาราง: ผู้ใช้ดูและแก้ไขได้เฉพาะแถวที่ `user_id` ของตนเอง
- เก็บเฉพาะชื่อโมเดลที่เลือกในฐานข้อมูล ไม่เก็บ Cloud API key

## Cloud Model and API Key Handling

- ผู้ใช้เลือกโมเดลและวาง API key ของตนเองในหน้า Settings
- แอปจะเก็บ key ในหน่วยความจำของ session เท่านั้น ไม่เขียนลง Supabase, localStorage, GitHub หรือประวัติแชท
- ทุกคำขอแชทส่ง key ผ่าน HTTPS ไปยัง Edge Function พร้อม access token ของ Supabase
- Edge Function ตรวจสอบผู้ใช้ ส่งต่อ key ไปยัง provider และไม่ log ค่า Authorization หรือ request body ที่มี key
- DeepSeek และ Cloud provider อื่นเป็นค่าใช้จ่ายของเจ้าของ key; โครงสร้างเว็บ, Auth และฐานข้อมูลอยู่ใน Free tier ตามโควตาผู้ให้บริการ

## Responsive UI

### Mobile (0–767px)

- แชทเต็มจอ มี header ขนาดกะทัดรัด
- เมนูหลักเป็น bottom navigation: Chat, History, Models, Settings
- composer ยึดด้านล่างและรองรับ safe-area ของ iOS/Android
- modal เป็น full-screen sheet, ปุ่มแตะมีพื้นที่อย่างน้อย 44px

### Desktop and Remote PC (768px+)

- ใช้ layout 3 ส่วนแบบ Commandblock เดิม: navigation, chat/history, utility tabs
- หน้าต่างแคบแบบ Remote PC จะยุบ sidebar และ utility tabs เป็น drawer
- รองรับ keyboard navigation, focus indicator และ reduced-motion

## PWA

- มี manifest, icons และ service worker สำหรับ shell ของ UI
- หน้าแอปที่ติดตั้งแล้วเปิดเหมือนแอปบนมือถือได้
- ไม่ cache API key, access token, คำตอบแชทส่วนบุคคล หรือ request ที่มี Authorization
- เมื่อตัดอินเทอร์เน็ต ให้แสดง offline state ที่ชัดเจนและยังเปิดหน้า login ที่ cache ได้

## GitHub Safety

- เพิ่ม `.gitignore` สำหรับ `config.json`, `.env*`, session, build artifacts และ installer payload
- เพิ่ม `.env.example` ที่มีเฉพาะ Supabase public URL และ anon key placeholder
- เพิ่ม `config.example.json` โดยไม่มี secret
- Edge Function secrets (เช่น provider default key หากมีในอนาคต) ตั้งใน Supabase เท่านั้น

## Explicit Non-Goals

- ไม่ให้เว็บเข้าถึงไฟล์หรือ Terminal บน PC โดยตรง
- ไม่แจกหรือใช้ DeepSeek key กลางของเจ้าของโปรเจกต์
- ไม่สร้าง paid service, custom domain หรือ backend server เพิ่มในระยะแรก

## Verification

- ทดสอบ Auth: สมัคร, ยืนยันอีเมล, login, logout, reset password และ Google callback
- ทดสอบ RLS ด้วยบัญชีสองคนเพื่อยืนยันว่าอ่านประวัติข้ามกันไม่ได้
- ทดสอบ Edge Function ว่าปฏิเสธ request ที่ไม่มี session หรือไม่มี key และไม่เขียน key ลง log
- ทดสอบ viewport 360px, 768px และ 1440px; ทดสอบ install PWA และ offline state
