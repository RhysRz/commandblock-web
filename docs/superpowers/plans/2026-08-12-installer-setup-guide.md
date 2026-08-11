# คู่มือการตั้งค่าภายในตัวติดตั้ง Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** เพิ่มคู่มือภาษาไทยที่ปลอดภัยลงในตัวติดตั้ง Commandblock ให้เปิดอ่านได้ทันทีหลังติดตั้งและเปิดซ้ำได้จาก Start Menu

**Architecture:** เก็บคู่มือเป็นไฟล์ข้อความ UTF-8 ใน `docs/` แล้วให้ Inno Setup คัดลอกไปยังโฟลเดอร์แอป ไฟล์ installer เดิมจะเพิ่มทั้งทางลัด Notepad และคำสั่งเปิดคู่มือหลังติดตั้ง โดยไม่เปลี่ยนตำแหน่งข้อมูลผู้ใช้หรือพฤติกรรมของตัวแอป

**Tech Stack:** Rust desktop app, Inno Setup 6, PowerShell build script, Node.js built-in test runner

## Global Constraints

- คู่มือต้องเป็นภาษาไทยและไม่มี API key, session หรือ `config.json` ของผู้ใช้
- คู่มือต้นฉบับคือ `docs/SETUP-GUIDE.txt` และไฟล์ที่ติดตั้งคือ `{app}\SETUP-GUIDE.txt`
- การเปิดคู่มือใช้ Windows Notepad และต้องไม่ทำงานเมื่อติดตั้งแบบ silent
- ทางลัดและ working directory ของ Commandblock ที่มีอยู่ต้องไม่เปลี่ยน
- โปรเจกต์นี้ไม่ใช่ Git repository จึงไม่มีขั้นตอน commit

---

### Task 1: เขียนคู่มือการตั้งค่าภาษาไทย

**Files:**
- Create: `docs/SETUP-GUIDE.txt`

**Interfaces:**
- Consumes: ตำแหน่งข้อมูลผู้ใช้ `%APPDATA%\Commandblock\config.json` และรูปแบบ OpenAI-compatible API ของแอป
- Produces: ไฟล์ข้อความ UTF-8 ที่ Inno Setup คัดลอกได้โดยไม่ต้องใช้โปรแกรมเฉพาะ

- [x] **Step 1: เขียนคู่มือฉบับสมบูรณ์**

ใส่หัวข้อ “คู่มือการตั้งค่าและใช้งาน Commandblock” ตามด้วยคำแนะนำเริ่มต้น การเลือกโฟลเดอร์ การตั้งค่า `config.json` สำหรับการติดตั้งและ portable วิธีตั้งค่า DeepSeek, Groq, Ollama ในเครื่อง และ Ollama Cloud พร้อมตัวอย่างที่ใช้ค่าแทน `<API_KEY_ของคุณ>` เท่านั้น ปิดท้ายด้วยข้อควรระวังว่าไม่แชร์ key และวิธีเปิดคู่มือซ้ำจาก Start Menu

- [x] **Step 2: ตรวจทานคู่มือด้วยตนเอง**

Run: `Get-Content -Raw docs\SETUP-GUIDE.txt`

Expected: เนื้อหาเป็นภาษาไทย อ่านเป็นลำดับ และไม่มี API key ที่ใช้งานได้จริง

### Task 2: เชื่อมคู่มือเข้ากับ Inno Setup

**Files:**
- Modify: `installer/Commandblock.iss`

**Interfaces:**
- Consumes: `docs/SETUP-GUIDE.txt`
- Produces: `{app}\SETUP-GUIDE.txt`, ทางลัด Start Menu และหน้าต่าง Notepad หลังติดตั้ง

- [x] **Step 1: เพิ่มการคัดลอกไฟล์คู่มือ**

เพิ่มบรรทัดต่อไปนี้ใน `[Files]`:

```iss
Source: "..\docs\SETUP-GUIDE.txt"; DestDir: "{app}"; Flags: ignoreversion
```

- [x] **Step 2: เพิ่มทางลัดเปิดคู่มือ**

เพิ่มบรรทัดต่อไปนี้ใน `[Icons]`:

```iss
Name: "{autoprograms}\คู่มือการตั้งค่า Commandblock"; Filename: "{sys}\notepad.exe"; Parameters: """{app}\SETUP-GUIDE.txt"""
```

- [x] **Step 3: เปิดคู่มือหลังติดตั้ง**

เพิ่มบรรทัดต่อไปนี้เป็นรายการแรกใน `[Run]`:

```iss
Filename: "{sys}\notepad.exe"; Parameters: """{app}\SETUP-GUIDE.txt"""; Description: "เปิดคู่มือการตั้งค่า Commandblock"; Flags: nowait postinstall skipifsilent
```

### Task 3: ตรวจสอบตัวติดตั้งและสร้างไฟล์ส่งมอบ

**Files:**
- Create: `dist/Commandblock-Setup.exe` (ไฟล์ build output)
- Test: `tests/commandblock-branding.test.cjs` (ปรับเฉพาะการทดสอบเชิงพฤติกรรมของ payload เมื่อตัวติดตั้ง build ได้)

**Interfaces:**
- Consumes: `installer/build-installer.ps1`, `installer/Commandblock.iss`, `docs/SETUP-GUIDE.txt`
- Produces: ตัวติดตั้ง Windows ที่คอมไพล์สำเร็จ

- [x] **Step 1: สร้างตัวติดตั้งจริง**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File installer\build-installer.ps1`

Expected: Inno Setup จบด้วย exit code 0 และสร้าง `dist\Commandblock-Setup.exe`

- [x] **Step 2: ตรวจสอบ payload ของ installer แบบไม่แกะไฟล์**

Run: `Get-Item dist\Commandblock-Setup.exe | Select-Object Name,Length,LastWriteTime`

Expected: ชื่อไฟล์เป็น `Commandblock-Setup.exe` และขนาดมากกว่า 0

- [x] **Step 3: รันทดสอบเดิมทั้งหมด**

Run: `node --test tests\*.test.cjs; cargo test`

Expected: ทุก Node test และ Rust test ผ่าน ไม่มี failure
