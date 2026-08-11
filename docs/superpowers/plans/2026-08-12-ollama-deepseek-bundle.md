# ตัวติดตั้งเต็ม Ollama และ DeepSeek Coder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** สร้าง `Commandblock-Setup.exe` ที่ติดตั้ง Ollama และโมเดล `deepseek-coder:1.3b` ในเครื่อง พร้อมตัวเลือกเข้าสู่ระบบ Ollama ของผู้ใช้หลังติดตั้ง

**Architecture:** เพิ่มสคริปต์ PowerShell ที่เตรียม payload จาก Ollama ทางการและ model store จริงไว้ใต้ `installer/payload/` พร้อม manifest checksum ตัวติดตั้ง Inno Setup ตรวจพื้นที่ว่าง รักษา Ollama ที่มีอยู่แล้ว คัดลอก model store และเปิด `ollama signin` แบบโต้ตอบเมื่อผู้ใช้เลือก

**Tech Stack:** Inno Setup 6, PowerShell 5+, Ollama Windows, Node.js built-in test runner, Rust

## Global Constraints

- รวม `deepseek-coder:1.3b` ขนาดประมาณ 776 MB; ไม่รวม API key, account, token, session หรือ config ส่วนตัว
- ใช้ `OLLAMA_MODELS` เมื่อกำหนด มิฉะนั้นใช้ `%USERPROFILE%\.ollama\models`
- ไม่ติดตั้งทับ Ollama ที่มีอยู่ และไม่เขียนทับ model manifest/blob ที่มีอยู่
- การเข้าสู่ระบบเป็น `ollama signin` แบบโต้ตอบของผู้ใช้เท่านั้น และข้ามได้
- สร้างและตรวจ checksum ของ payload ก่อน build
- โปรเจกต์นี้ไม่ใช่ Git repository จึงไม่มีขั้นตอน commit

---

### Task 1: เตรียม Ollama และ DeepSeek model payload

**Files:**
- Create: `installer/prepare-ollama-bundle.ps1`
- Create: `installer/payload/ollama/OllamaSetup.exe`
- Create: `installer/payload/models/` (model store ของ Ollama)
- Create: `installer/payload/SHA256SUMS.txt`
- Create: `installer/payload/LICENSES/OLLAMA-MIT.txt`
- Create: `installer/payload/LICENSES/DEEPSEEK-CODER.txt`
- Test: `tests/ollama-bundle-validation.ps1`

**Interfaces:**
- Consumes: `ollama.exe`, `ollama pull deepseek-coder:1.3b`, `https://ollama.com/download/OllamaSetup.exe`
- Produces: `payload/models/manifests/**/deepseek-coder/1.3b`, referenced blob files, official runtime installer, and SHA-256 manifest

- [x] **Step 1: เขียน test ที่ตรวจ payload จริง**

สร้าง `tests/ollama-bundle-validation.ps1` ให้รับ `-PayloadRoot` และตรวจว่าไฟล์ runtime, SHA256SUMS, license ทั้งสอง, model manifest และ blob ทุกตัวใน manifest มีอยู่จริง จากนั้นคำนวณ `Get-FileHash -Algorithm SHA256` เทียบกับ SHA256SUMS และคืน exit code 1 เมื่อขาดหรือ hash ไม่ตรง

```powershell
param([string]$PayloadRoot = (Join-Path $PSScriptRoot '..\installer\payload'))
$errors = [System.Collections.Generic.List[string]]::new()
if (-not (Test-Path (Join-Path $PayloadRoot 'ollama\OllamaSetup.exe'))) { $errors.Add('Missing OllamaSetup.exe') }
if (-not (Test-Path (Join-Path $PayloadRoot 'SHA256SUMS.txt'))) { $errors.Add('Missing SHA256SUMS.txt') }
if ($errors.Count) { $errors | ForEach-Object { Write-Error $_ }; exit 1 }
```

- [x] **Step 2: รัน test ให้ fail ก่อนมี payload**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File tests\ollama-bundle-validation.ps1`

Expected: FAIL ด้วย `Missing OllamaSetup.exe`

- [x] **Step 3: เขียนสคริปต์เตรียม payload แบบตรวจสอบได้**

`installer/prepare-ollama-bundle.ps1` ต้องสร้างโฟลเดอร์ payload ใหม่แบบปลอดภัย, ดาวน์โหลด `https://ollama.com/download/OllamaSetup.exe` ไปยัง `payload\ollama\OllamaSetup.exe`, รัน `ollama pull deepseek-coder:1.3b`, อ่าน manifest ใต้ `$env:USERPROFILE\.ollama\models\manifests`, คัดลอก manifest และ blob ที่ manifest อ้างถึงไปยัง `payload\models`, คัดลอก license ของ Ollama และ DeepSeek Coderไปยัง `payload\LICENSES`, แล้วสร้าง SHA256SUMS สำหรับไฟล์ทั้งหมด ยกเว้น SHA256SUMS เอง

```powershell
& ollama pull 'deepseek-coder:1.3b'
if ($LASTEXITCODE -ne 0) { throw 'Ollama could not download deepseek-coder:1.3b.' }
Invoke-WebRequest -Uri 'https://ollama.com/download/OllamaSetup.exe' -OutFile $runtimePath
```

- [x] **Step 4: รันสคริปต์เตรียม payload**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File installer\prepare-ollama-bundle.ps1`

Expected: สร้าง runtime, model manifest, blobs, licenses และ SHA256SUMS ใต้ `installer\payload`

- [x] **Step 5: รัน test payload ให้ผ่าน**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File tests\ollama-bundle-validation.ps1`

Expected: PASS พร้อมชื่อ model manifest และจำนวน blob ที่ตรวจแล้ว

### Task 2: เพิ่ม payload installer และการติดตั้งแบบไม่ทับข้อมูล

**Files:**
- Modify: `installer/Commandblock.iss`
- Modify: `installer/build-installer.ps1`
- Test: `tests/ollama-bundle-validation.ps1`

**Interfaces:**
- Consumes: `installer/payload/` ที่ผ่าน checksum test
- Produces: `Commandblock-Setup.exe` ที่พก Ollama installer, model store และ notices

- [x] **Step 1: เพิ่ม pre-build guard ใน build script**

ให้ `build-installer.ps1` เรียก `tests\ollama-bundle-validation.ps1` ก่อน cargo build และหยุดทันทีหาก payload ไม่ครบหรือ hash ไม่ตรง

```powershell
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot '..\tests\ollama-bundle-validation.ps1')
if ($LASTEXITCODE -ne 0) { throw 'Ollama bundle validation failed.' }
```

- [x] **Step 2: เพิ่ม payload ใน Inno Setup**

เพิ่ม `[Files]` rules สำหรับ runtime ไป `{tmp}`, model store ไป `{code:ModelStoreDir}`, และ notices ไป `{app}\licenses` โดย model files ต้องมี `recursesubdirs createallsubdirs onlyifdoesntexist ignoreversion`

```iss
Source: "payload\ollama\OllamaSetup.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall ignoreversion
Source: "payload\models\*"; DestDir: "{code:ModelStoreDir}"; Flags: recursesubdirs createallsubdirs onlyifdoesntexist ignoreversion
Source: "payload\LICENSES\*"; DestDir: "{app}\licenses"; Flags: recursesubdirs createallsubdirs ignoreversion
```

- [x] **Step 3: เพิ่ม Inno Setup code สำหรับ path, disk space และ runtime detection**

สร้าง `ModelStoreDir(): String` ที่คืน `GetEnv('OLLAMA_MODELS')` เมื่อไม่ว่าง มิฉะนั้น `ExpandConstant('{userprofile}\.ollama\models')`; สร้าง `OllamaExe(): String` ที่คืน `{localappdata}\Programs\Ollama\ollama.exe`; และใช้ `PrepareToInstall` ตรวจว่าปลายทางมีพื้นที่ว่างอย่างน้อยขนาด payload บวก 1 GB ก่อนคัดลอก

```pascal
function ModelStoreDir(): String;
begin
  Result := GetEnv('OLLAMA_MODELS');
  if Result = '' then Result := ExpandConstant('{userprofile}\.ollama\models');
end;
```

- [x] **Step 4: เรียก Ollama installer เมื่อเครื่องยังไม่มี runtime**

ใน `CurStepChanged(ssPostInstall)` เรียก `Exec(ExpandConstant('{tmp}\OllamaSetup.exe'), '', '', SW_SHOW, ewWaitUntilTerminated, ResultCode)` เฉพาะเมื่อ `OllamaExe()` ยังไม่มีไฟล์ และแจ้งข้อความไทยหากการติดตั้งไม่สำเร็จ; ถ้ามี runtime เดิมอยู่แล้วต้องไม่เรียก installer

- [x] **Step 5: เพิ่มทางลัด license และการตรวจ model**

เพิ่ม Start Menu shortcut เปิด `{app}\licenses\DEEPSEEK-CODER.txt`; หลังติดตั้งให้รัน `ollama show deepseek-coder:1.3b` ผ่าน `OllamaExe()` แบบ hidden โดยผลล้มเหลวเป็นข้อความแนะนำ ไม่ทำลายการติดตั้ง

### Task 3: ตั้งค่า Commandblock และเข้าสู่ระบบ Ollama

**Files:**
- Modify: `installer/Commandblock.iss`
- Modify: `docs/SETUP-GUIDE.txt`
- Modify: `config.json` (template เฉพาะที่ไม่มี secret หากแอปต้องใช้)

**Interfaces:**
- Consumes: runtime และ model store จาก Task 2
- Produces: โมเดล DeepSeek แบบ local ที่เลือกได้และทางเลือก sign-in ที่ไม่เก็บ credential

- [x] **Step 1: เพิ่ม config template แบบไม่มี key**

เพิ่ม `deepseek-coder:1.3b` ในรายการ Ollama ด้วย `base_url` เป็น `http://localhost:11434/v1`, `api_key` เป็นสตริงว่าง และรักษาโมเดลเดิมทั้งหมดไว้

```json
{ "model": "deepseek-coder:1.3b", "base_url": "http://localhost:11434/v1", "api_key": "" }
```

- [x] **Step 2: เพิ่มตัวเลือก sign-in หลังติดตั้ง**

เพิ่ม `[Run]` item เรียก `{localappdata}\Programs\Ollama\ollama.exe signin` หลังติดตั้ง พร้อม Description ภาษาไทย “เข้าสู่ระบบ Ollama (สำหรับโมเดล Cloud — ข้ามได้)” และ flags `postinstall nowait skipifsilent unchecked`; ต้องอยู่ก่อนการเปิด Commandblock

- [x] **Step 3: อัปเดตคู่มือ**

เพิ่มหัวข้อ DeepSeek Coder ที่อธิบายว่าโมเดลในเครื่องพร้อมใช้หลัง Setup, วิธีเลือกจากปุ่มโมเดล, เป็นโมเดลฟรีไม่มี API key, และว่า Ollama sign-in เป็นตัวเลือกสำหรับ `:cloud` ไม่ใช่เงื่อนไขของ DeepSeek Coder ในเครื่อง

### Task 4: สร้างและยืนยันตัวติดตั้งเต็ม

**Files:**
- Create: `dist/Commandblock-Setup.exe`
- Test: `tests/ollama-bundle-validation.ps1`, `tests/*.test.cjs`, Rust unit tests

**Interfaces:**
- Consumes: payload ที่ผ่าน checksum, Inno Setup script, Commandblock release executable
- Produces: installer ขนาดเต็มที่ deploy ได้

- [x] **Step 1: สร้าง Setup แบบเต็ม**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File installer\build-installer.ps1`

Expected: Inno Setup exit 0 และ log ระบุว่า compresses `OllamaSetup.exe` และ model payload

- [x] **Step 2: ตรวจผลลัพธ์**

Run: `Get-Item dist\Commandblock-Setup.exe | Select-Object Name,Length,LastWriteTime`

Expected: ขนาดมากกว่า 700 MB และชื่อ `Commandblock-Setup.exe`

- [x] **Step 3: รันทดสอบทั้งหมด**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File tests\ollama-bundle-validation.ps1; node --test tests\*.test.cjs; cargo test`

Expected: ทุก test ผ่าน ไม่มี API key อยู่ใน payload หรือ installer script
