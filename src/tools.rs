//! เครื่องมือ (tools) ของ Commandblock — ใช้สำหรับส่งให้ LLM เรียกใช้
//! 13 อย่าง: read_file, write_file, edit_file, append_file,
//! list_directory, code_search, run_command, update_plan,
//! web_search, read_url, open_preview, list_skills, load_skill

use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// ข้อมูลไฟล์ที่ถูกแก้ไข 1 ครั้ง (สำหรับกล่องสรุปแบบ Freebuff)
#[derive(Debug, Clone)]
pub struct ChangeEntry {
    pub path: String,
    pub status: String, // "A" = สร้างใหม่, "M" = แก้ไข
    pub added: usize,
    pub deleted: usize,
}

/// ไฟล์ที่ถูกแก้ไขในเซสชันนี้ (สำหรับแท็บ Changes + กล่องสรุปในแชท)
pub static CHANGED_FILES: Mutex<Vec<ChangeEntry>> = Mutex::new(Vec::new());
/// การแก้ไฟล์ล่าสุด (ส่งเป็นเหตุการณ์ SSE "change" ให้ GUI หลังรันเครื่องมือ)
static LAST_CHANGE: Mutex<Option<ChangeEntry>> = Mutex::new(None);
/// กิจกรรมล่าสุดของเครื่องมือ (สำหรับแท็บ Queue ของ GUI)
pub static ACTIVITY: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn record_activity(name: &str, args: &Value) {
    let mut a = ACTIVITY.lock().unwrap();
    a.push(format!("{name} {}", serde_json::to_string(args).unwrap_or_default()));
    if a.len() > 50 {
        let keep = a.len() - 50;
        a.drain(..keep);
    }
}

fn record_changed(path: &str, status: &str, added: usize, deleted: usize) {
    {
        let mut c = CHANGED_FILES.lock().unwrap();
        if let Some(e) = c.iter_mut().find(|e| e.path == path) {
            // สร้างแล้วแก้ต่อ → กลายเป็น M แต่รวมจำนวนบรรทัด
            if e.status == "A" && status == "M" {
                e.status = "M".to_string();
            }
            e.added += added;
            e.deleted += deleted;
        } else {
            c.push(ChangeEntry {
                path: path.to_string(),
                status: status.to_string(),
                added,
                deleted,
            });
        }
    }
    *LAST_CHANGE.lock().unwrap() = Some(ChangeEntry {
        path: path.to_string(),
        status: status.to_string(),
        added,
        deleted,
    });
}

/// ดึงการแก้ไฟล์ล่าสุดแล้วล้าง (GUI เรียกหลังรันเครื่องมือเพื่อส่ง SSE "change")
pub fn take_last_change() -> Option<ChangeEntry> {
    LAST_CHANGE.lock().unwrap().take()
}

pub const TOOL_NAMES: &[&str] = &[
    "read_file",
    "write_file",
    "edit_file",
    "append_file",
    "list_directory",
    "code_search",
    "run_command",
    "update_plan",
    "web_search",
    "read_url",
    "open_preview",
    "list_skills",
    "load_skill",
];

const MAX_READ_LINES: usize = 5000;
const MAX_READ_BYTES: u64 = 2 * 1024 * 1024; // 2 MB
const MAX_SEARCH_HITS: usize = 100;
const MAX_CMD_OUTPUT: usize = 50_000;
const MAX_CMD_CAPTURE: usize = 150_000;

pub fn tool_schemas() -> Vec<Value> {
    vec![
        json!({"type":"function","function":{
            "name":"read_file",
            "description":"อ่านไฟล์ข้อความเพื่อดูเนื้อหา (พร้อมเลขบรรทัด) ใช้ก่อนแก้ไขไฟล์เสมอ",
            "parameters":{"type":"object","properties":{
                "path":{"type":"string","description":"พาธของไฟล์ เช่น src/main.rs"},
                "offset":{"type":"integer","description":"เริ่มอ่านจากบรรทัดนี้ (0 = บรรทัดแรก) ไม่ระบุ = อ่านตั้งแต่ต้น"},
                "limit":{"type":"integer","description":"จำนวนบรรทัดที่อ่าน สูงสุด 5000 ไม่ระบุ = 5000"}
            },"required":["path"]}
        }}),
        json!({"type":"function","function":{
            "name":"write_file",
            "description":"เขียนไฟล์ใหม่ (แทนที่เนื้อหาเดิมทั้งหมด) — สร้างโฟลเดอร์ให้อัตโนมัติ ใช้เฉพาะไฟล์ที่อยากเขียนทับทั้งไฟล์",
            "parameters":{"type":"object","properties":{
                "path":{"type":"string","description":"พาธของไฟล์ที่จะเขียน"},
                "content":{"type":"string","description":"เนื้อหาทั้งไฟล์"}
            },"required":["path","content"]}
        }}),
        json!({"type":"function","function":{
            "name":"edit_file",
            "description":"แก้ไขไฟล์โดยแทนที่ข้อความเก่าด้วยข้อความใหม่ (ครั้งแรกที่เจอ ยกเว้น all=true แทนทุกที่) — ใช้สำหรับแก้เฉพาะจุดโดยไม่แตะส่วนอื่น",
            "parameters":{"type":"object","properties":{
                "path":{"type":"string","description":"พาธของไฟล์"},
                "old":{"type":"string","description":"ข้อความเดิมที่ต้องการแทนที่ (ต้องตรงกับในไฟล์เป๊ะๆ รวมช่องว่าง)"},
                "new":{"type":"string","description":"ข้อความใหม่ที่จะแทนที่"},
                "all":{"type":"boolean","description":"true = แทนที่ทุกตำแหน่งที่เจอ false = แทนที่ครั้งแรก"}
            },"required":["path","old","new"]}
        }}),
        json!({"type":"function","function":{
            "name":"append_file",
            "description":"เพิ่มข้อความต่อท้ายไฟล์ (สร้างไฟล์ใหม่ถ้ายังไม่มี)",
            "parameters":{"type":"object","properties":{
                "path":{"type":"string","description":"พาธของไฟล์"},
                "content":{"type":"string","description":"ข้อความที่จะต่อท้าย"}
            },"required":["path","content"]}
        }}),
        json!({"type":"function","function":{
            "name":"list_directory",
            "description":"แสดงรายการไฟล์และโฟลเดอร์ในไดเรกทอรี (โฟลเดอร์มาก่อน แล้วไฟล์ เรียงตามชื่อ)",
            "parameters":{"type":"object","properties":{
                "path":{"type":"string","description":"ไดเรกทอรีที่จะแสดง ไม่ระบุ = . (โฟลเดอร์ปัจจุบัน)"}
            },"required":[]}
        }}),
        json!({"type":"function","function":{
            "name":"code_search",
            "description":"ค้นหาข้อความในโค้ดทั้งโปรเจกต์ (ค้นแบบซ้ำๆ ทุกโฟลเดอร์ย่อย) คืนผลเป็น path:บรรทัด:เนื้อหา — ใช้หาว่าโค้ด/ฟังก์ชัน/ตัวแปรที่เกี่ยวข้องอยู่ที่ไหน",
            "parameters":{"type":"object","properties":{
                "pattern":{"type":"string","description":"ข้อความที่ต้องการค้นหา เช่น 'function foo' หรือ 'TODO'"},
                "cwd":{"type":"string","description":"โฟลเดอร์เริ่มต้นที่ค้น ไม่ระบุ = ."},
                "flags":{"type":"string","description":"ตัวเลือก: '-i' ไม่สนใจตัวพิมพ์ใหญ่เล็ก, '-w' เฉพาะคำเต็ม (เช่น \"-i -w\")"}
            },"required":["pattern"]}
        }}),
        json!({"type":"function","function":{
            "name":"run_command",
            "description":"รันคำสั่ง terminal (บน Windows ใช้ cmd /C) เพื่อ build/test/ตรวจสอบ/รันสคริปต์ คืน stdout+stderr — ใช้สำหรับยืนยันผลงาน เช่น npm test, cargo build, python -m py_compile, git status",
            "parameters":{"type":"object","properties":{
                "command":{"type":"string","description":"คำสั่งที่ต้องการรัน เช่น 'npm test' หรือ 'cargo build'"},
                "cwd":{"type":"string","description":"โฟลเดอร์ที่จะรันคำสั่ง ไม่ระบุ = โฟลเดอร์ที่รัน Commandblock อยู่"},
                "timeout_seconds":{"type":"integer","description":"เวลาสูงสุดที่รอ (วินาที) ค่า default 60"}
            },"required":["command"]}
        }}),
        json!({"type":"function","function":{
            "name":"update_plan",
            "description":"บันทึกแผนการทำงานหลายขั้นตอน (แสดงได้ด้วยคำสั่ง /plan) — ใช้เมื่อเริ่มงานที่ซับซ้อนเพื่อบอกผู้ใช้ว่าวางแผนทำอะไรบ้าง",
            "parameters":{"type":"object","properties":{
                "plan":{"type":"string","description":"แผนงานเป็นข้อความ มีเลขขั้นตอนชัดเจน"}
            },"required":["plan"]}
        }}),
        json!({"type":"function","function":{
            "name":"web_search",
            "description":"ค้นหาข้อมูลในอินเทอร์เน็ต (ผ่าน DuckDuckGo ไม่ต้องใช้ key) คืนรายการผลลัพธ์พร้อมหัวข้อ+URL+คำอธิบายสั้น — ใช้เมื่อผู้ใช้ถามเรื่องข้อมูลปัจจุบัน ข่าว เอกสาร/วิธีใช้ล่าสุด หรืออะไรก็ตามที่ไม่อยู่ในโปรเจกต์",
            "parameters":{"type":"object","properties":{
                "query":{"type":"string","description":"คำค้นหา (ภาษาอังกฤษให้ผลดีกว่า)"},
                "max_results":{"type":"integer","description":"จำนวนผลลัพธ์ที่ต้องการ (1-10 ค่า default 5)"}
            },"required":["query"]}
        }}),
        json!({"type":"function","function":{
            "name":"read_url",
            "description":"อ่านเนื้อหาของหน้าเว็บ (http/https) แล้วแปลงเป็นข้อความธรรมดา — ใช้ต่อจาก web_search เพื่ออ่านรายละเอียดจากลิงก์",
            "parameters":{"type":"object","properties":{
                "url":{"type":"string","description":"URL ของหน้าเว็บ เช่น https://example.com/doc"},
                "max_chars":{"type":"integer","description":"จำนวนตัวอักษรสูงสุดที่อ่าน (ค่า default 4000 สูงสุด 20000)"}
            },"required":["url"]}
        }}),
        json!({"type":"function","function":{
            "name":"open_preview",
            "description":"เปิดพรีวิวเว็บให้ผู้ใช้เห็นภาพจริงในเบราว์เซอร์ (รันเซิร์ฟเวอร์ local + เปิด browser อัตโนมัติ) — ใช้เมื่อผู้ใช้ขอ 'ดูพรีวิว/แสดงหน้าเว็บ' หรือเมื่อสร้างเว็บแอป/หน้า HTML เพื่อให้ผู้ใช้เห็นผลลัพธ์ทันที",
            "parameters":{"type":"object","properties":{
                "html":{"type":"string","description":"เนื้อหา HTML ทั้งหน้า (ใช้อย่างใดอย่างหนึ่งกับ path) — จะบันทึกเป็น preview/index.html"},
                "path":{"type":"string","description":"พาธไฟล์ .html หรือโฟลเดอร์เว็บ (ใช้อย่างใดอย่างหนึ่งกับ html)"}
            },"required":[]}
        }}),
        json!({"type":"function","function":{
            "name":"list_skills",
            "description":"ดูรายการทักษะ (skills) ที่มีอยู่ — ทักษะคือคำแนะนำ/แนวทางเฉพาะทางที่โหลดมาใช้กับงานนั้นๆ (เช่น accessibility, api-design) เหมือนผู้ช่วย AI ระดับมืออาชีพ",
            "parameters":{"type":"object","properties":{},"required":[]}
        }}),
        json!({"type":"function","function":{
            "name":"load_skill",
            "description":"โหลดเนื้อหาทักษะ (SKILL.md) มาอ่านเพื่อปฏิบัติตาม — ใช้ก่อนทำงานเฉพาะทาง เช่น ตรวจสอบความเข้าถึง (accessibility), ออกแบบ API, จัดการ Azure",
            "parameters":{"type":"object","properties":{
                "name":{"type":"string","description":"ชื่อทักษะ เช่น accessibility หรือ api-design-principles (ดูจาก list_skills)"}
            },"required":["name"]}
        }}),
    ]
}

/// รันเครื่องมือตามชื่อ รับ args (JSON object) คืนผลเป็นข้อความ (พร้อมสถานะ)
pub fn execute(name: &str, args: &Value, plan: &mut Option<String>) -> String {
    record_activity(name, args);
    *LAST_CHANGE.lock().unwrap() = None; // เครื่องมือนี้ยังไม่ใช่การแก้ไฟล์ — ล้างค่าค้างจากรอบก่อน
    match name {
        "read_file" => read_file(args),
        "write_file" => write_file(args),
        "edit_file" => edit_file(args),
        "append_file" => append_file(args),
        "list_directory" => list_directory(args),
        "code_search" => code_search(args),
        "run_command" => run_command(args),
        "update_plan" => {
            let p = args.get("plan").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            if p.is_empty() {
                "ไม่ได้รับข้อความแผน".to_string()
            } else {
                *plan = Some(p.clone());
                format!("บันทึกแผนแล้ว:\n{p}")
            }
        }
        "web_search" => web_search(args),
        "read_url" => read_url(args),
        "open_preview" => open_preview(args),
        "list_skills" => list_skills(),
        "load_skill" => load_skill(args),
        other => format!("[เครื่องมือ] ไม่รู้จักเครื่องมือ '{other}'"),
    }
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str()).map(|s| s.trim())
}

// ---------- read_file ----------

fn read_file(args: &Value) -> String {
    let Some(path) = arg_str(args, "path") else {
        return "[read_file] ต้องระบุ path".to_string();
    };
    if path.contains("..") && Path::new(path).is_absolute() {
        return "[read_file] path ต้องเป็นพาธสัมพัทธ์ภายในโปรเจกต์".to_string();
    }

    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => return format!("[read_file] อ่าน metadata ไม่ได้: {e}"),
    };
    if meta.is_dir() {
        return "[read_file] พาธนี้เป็นโฟลเดอร์ ใช้ list_directory แทน".to_string();
    }
    if meta.len() > MAX_READ_BYTES {
        return format!(
            "[read_file] ไฟล์ใหญ่เกิน ({:.1} MB) เกินขีดจำกัด {:.0} MB — ใช้ run_command เช่น 'tail -100 <ไฟล์>' หรือ 'head -100 <ไฟล์>' แทน",
            meta.len() as f64 / 1e6,
            MAX_READ_BYTES as f64 / 1e6
        );
    }

    // ตรวจไฟล์ไบนารี
    let mut probe = [0u8; 8192];
    let n = fs::File::open(path).and_then(|mut f| f.read(&mut probe)).unwrap_or(0);
    if probe[..n].contains(&0) {
        return "[read_file] ดูเหมือนไฟล์ไบนารี (มี byte 0) — ไม่แสดงเนื้อหา".to_string();
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return format!("[read_file] อ่านไฟล์ไม่สำเร็จ: {e}"),
    };

    let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| (n as usize).min(MAX_READ_LINES))
        .unwrap_or(MAX_READ_LINES);

    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();
    if offset >= total {
        return format!("[read_file] บรรทัดเริ่มต้น ({offset}) เกินจำนวนบรรทัดทั้งหมด ({total})");
    }
    let end = (offset + limit).min(total);
    let mut out = String::new();
    for (i, line) in lines[offset..end].iter().enumerate() {
        out.push_str(&format!("{:>6} │ {}\n", offset + i + 1, line));
    }
    let mut note = format!("[read_file] {} ({} บรรทัด, แสดง {}-{})", path, total, offset + 1, end);
    if end < total {
        note.push_str(&format!(" — ยังมีอีก {} บรรทัด ใช้ offset={} อ่านต่อ", total - end, end));
    }
    out.push_str(&note);
    out
}

// ---------- write_file ----------

fn write_file(args: &Value) -> String {
    let (Some(path), Some(content)) = (arg_str(args, "path"), args.get("content").and_then(|v| v.as_str())) else {
        return "[write_file] ต้องระบุ path และ content".to_string();
    };
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }
    let existed = Path::new(path).exists();
    let old_lines = if existed {
        fs::read_to_string(path).map(|c| c.lines().count()).unwrap_or(0)
    } else {
        0
    };
    match fs::write(path, content) {
        Ok(()) => {
            let new_lines = content.lines().count();
            let (status, added, deleted) = if existed {
                ("M", new_lines, old_lines)
            } else {
                ("A", new_lines, 0)
            };
            record_changed(path, status, added, deleted);
            format!("[write_file] เขียนไฟล์แล้ว ({} ไบต์): {path}", content.len())
        }
        Err(e) => format!("[write_file] เขียนไม่สำเร็จ: {e}"),
    }
}

// ---------- edit_file ----------

fn edit_file(args: &Value) -> String {
    let (Some(path), Some(old), Some(new)) = (
        arg_str(args, "path"),
        args.get("old").and_then(|v| v.as_str()),
        args.get("new").and_then(|v| v.as_str()),
    ) else {
        return "[edit_file] ต้องระบุ path, old และ new".to_string();
    };
    let all = args.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return format!("[edit_file] อ่านไฟล์ไม่สำเร็จ: {e}"),
    };
    if !content.contains(old) {
        return format!("[edit_file] ไม่พบข้อความที่จะแทนที่ใน {path} — ตรวจว่า old ตรงกับในไฟล์เป๊ะๆ (รวมช่องว่าง)");
    }
    let count = if all {
        content.matches(old).count()
    } else {
        1
    };
    let new_content = if all {
        content.replace(old, new)
    } else {
        content.replacen(old, new, 1)
    };
    // นับบรรทัดที่เพิ่ม/ลบ (แบบประมาณ: บรรทัดที่ไม่ซ้ำกัน)
    let old_set: std::collections::HashSet<&str> = content.lines().collect();
    let new_set: std::collections::HashSet<&str> = new_content.lines().collect();
    let added = new_set.difference(&old_set).count();
    let deleted = old_set.difference(&new_set).count();
    match fs::write(path, &new_content) {
        Ok(()) => {
            record_changed(path, "M", added, deleted);
            format!("[edit_file] แก้ไข {path} แล้ว ({count} ตำแหน่ง)")
        }
        Err(e) => format!("[edit_file] เขียนไม่สำเร็จ: {e}"),
    }
}

// ---------- append_file ----------

fn append_file(args: &Value) -> String {
    let (Some(path), Some(content)) = (arg_str(args, "path"), args.get("content").and_then(|v| v.as_str())) else {
        return "[append_file] ต้องระบุ path และ content".to_string();
    };
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }
    let existed = Path::new(path).exists();
    match fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut f) => {
            use std::io::Write;
            match f.write_all(content.as_bytes()) {
                Ok(()) => {
                    let added = content.lines().count();
                    record_changed(path, if existed { "M" } else { "A" }, added, 0);
                    format!("[append_file] เพิ่ม {} ไบต์ต่อท้าย {path}", content.len())
                }
                Err(e) => format!("[append_file] เขียนไม่สำเร็จ: {e}"),
            }
        }
        Err(e) => format!("[append_file] เปิดไฟล์ไม่ได้: {e}"),
    }
}

// ---------- list_directory ----------

fn list_directory(args: &Value) -> String {
    let path = arg_str(args, "path").unwrap_or(".");
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => return format!("[list_directory] เปิดโฟลเดอร์ไม่ได้: {e}"),
    };

    let mut items: Vec<(String, bool, u64)> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        items.push((name, is_dir, size));
    }
    items.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let cap = 300;
    let mut out = String::new();
    if path != "." {
        out.push_str(&format!("[list_directory] {path}\n"));
    }
    for (name, is_dir, size) in items.iter().take(cap) {
        if *is_dir {
            out.push_str(&format!("[dir ] {name}/\n"));
        } else {
            out.push_str(&format!("[file] {name} ({} B)\n", fmt_size(*size)));
        }
    }
    if items.len() > cap {
        out.push_str(&format!("… (แสดง {} จาก {} รายการ)", cap, items.len()));
    }
    out
}

fn fmt_size(b: u64) -> String {
    if b >= 1_000_000 {
        format!("{:.1} MB", b as f64 / 1e6)
    } else if b >= 1000 {
        format!("{:.1} KB", b as f64 / 1e3)
    } else {
        b.to_string()
    }
}

// ---------- code_search ----------

const SKIP_DIRS: &[&str] = &[
    ".git", "node_modules", "target", "dist", "build", "vendor", ".venv", "venv",
    "__pycache__", ".next", ".nuxt", ".freebuff", ".idea", ".vscode", ".worktrees",
    ".cache", "out", "bin", "obj", ".pytest_cache", "coverage",
];

fn code_search(args: &Value) -> String {
    let Some(pattern) = arg_str(args, "pattern") else {
        return "[code_search] ต้องระบุ pattern".to_string();
    };
    let cwd = arg_str(args, "cwd").unwrap_or(".");
    let flags = arg_str(args, "flags").unwrap_or("");
    let case_insensitive = flags.contains("-i");
    let whole_word = flags.contains("-w");

    let hay = if case_insensitive {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };

    let mut hits: Vec<String> = Vec::new();
    let mut scanned: u64 = 0;
    walk(
        Path::new(cwd),
        &hay,
        whole_word,
        case_insensitive,
        &mut hits,
        &mut scanned,
    );

    if hits.is_empty() {
        format!("[code_search] ไม่พบ '{pattern}' ใน {cwd}")
    } else {
        let mut out = format!("[code_search] พบ {} ตำแหน่งของ '{pattern}':\n", hits.len());
        for h in hits {
            out.push_str(&h);
            out.push('\n');
        }
        out.push_str(&format!("(สแกน {} ไฟล์)", scanned));
        out
    }
}

fn walk(
    dir: &Path,
    pattern: &str,
    whole_word: bool,
    ci: bool,
    hits: &mut Vec<String>,
    scanned: &mut u64,
) {
    if hits.len() >= MAX_SEARCH_HITS {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if hits.len() >= MAX_SEARCH_HITS {
            return;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_dir() {
            if SKIP_DIRS.contains(&name.as_str()) || name.starts_with('.') {
                continue;
            }
            walk(&path, pattern, whole_word, ci, hits, scanned);
        } else if ft.is_file() {
            // ข้ามไฟล์ใหญ่เกิน 5 MB
            if entry.metadata().map(|m| m.len() > 5_000_000).unwrap_or(false) {
                continue;
            }
            // ข้ามไบนารี
            let mut probe = [0u8; 4096];
            let n = fs::File::open(&path).and_then(|mut f| f.read(&mut probe)).unwrap_or(0);
            if probe[..n].contains(&0) {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            *scanned += 1;
            let target = if ci { content.to_lowercase() } else { content.clone() };
            if !target.contains(pattern) {
                continue;
            }
            for (i, line) in content.lines().enumerate() {
                if hits.len() >= MAX_SEARCH_HITS {
                    break;
                }
                let l = if ci { line.to_lowercase() } else { line.to_string() };
                if l.contains(pattern) && (!whole_word || is_whole_word(&l, pattern)) {
                    let trimmed: String = line.trim().chars().take(200).collect();
                    hits.push(format!("{}:{}: {}", path.display(), i + 1, trimmed));
                }
            }
        }
    }
}

fn is_whole_word(line: &str, pattern: &str) -> bool {
    line.match_indices(pattern)
        .any(|(start, _)| {
            let before_ok = start == 0 || !line.as_bytes()[start - 1].is_ascii_alphanumeric();
            let end = start + pattern.len();
            let after_ok = end >= line.len() || !line.as_bytes()[end].is_ascii_alphanumeric();
            before_ok && after_ok
        })
}

// ---------- run_command ----------

fn run_command(args: &Value) -> String {
    let Some(command) = arg_str(args, "command") else {
        return "[run_command] ต้องระบุ command".to_string();
    };
    let cwd = arg_str(args, "cwd");
    let timeout = args
        .get("timeout_seconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(60)
        .max(1)
        .min(3600);

    #[cfg(windows)]
    let (program, prefix) = ("cmd", "/C");
    #[cfg(not(windows))]
    let (program, prefix) = ("/bin/sh", "-c");

    let mut cmd = Command::new(program);
    cmd.arg(prefix).arg(command);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW — ไม่เปิดหน้าต่าง cmd ใหม่
    }
    if let Some(c) = cwd {
        cmd.current_dir(c);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return format!("[run_command] เริ่มคำสั่งไม่ได้: {e}"),
    };

    let out = Arc::new(Mutex::new(Vec::<u8>::new()));
    let mut handles = Vec::new();

    if let Some(stdout) = child.stdout.take() {
        let buf = Arc::clone(&out);
        handles.push(std::thread::spawn(move || drain(stdout, buf)));
    }
    if let Some(stderr) = child.stderr.take() {
        let buf = Arc::clone(&out);
        handles.push(std::thread::spawn(move || drain(stderr, buf)));
    }

    let deadline = Instant::now() + Duration::from_secs(timeout);
    let status: Option<std::process::ExitStatus> = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return format!("[run_command] รอคำสั่งไม่สำเร็จ: {e}");
            }
        }
    };

    for h in handles {
        let _ = h.join();
    }

    let bytes = out.lock().map(|g| g.clone()).unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes).to_string();
    let text = truncate_tail(&text, MAX_CMD_OUTPUT);

    match status {
        Some(s) => {
            let code = s.code().map(|c| c.to_string()).unwrap_or_else(|| "?".to_string());
            format!("[run_command] รหัสจบ: {code}\n{text}")
        }
        None => format!("[run_command] ใช้เวลาเกิน {timeout} วินาที ถูกยกเลิก\n{text}"),
    }
}

fn drain(mut reader: impl Read, buf: Arc<Mutex<Vec<u8>>>) {
    let mut chunk = [0u8; 8192];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                let mut g = buf.lock().unwrap();
                if g.len() < MAX_CMD_CAPTURE {
                    g.extend_from_slice(&chunk[..n]);
                }
            }
            Err(_) => break,
        }
    }
}

fn truncate_tail(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut start = s.len() - max;
    while !s.is_char_boundary(start) {
        start += 1;
    }
    format!("…(ตัดผลลัพธ์)\n{}", &s[start..])
}

// ---------- web_search ----------

const SEARCH_ENDPOINT: &str = "https://html.duckduckgo.com/html/";
const WEB_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36";

fn web_search(args: &Value) -> String {
    let Some(query) = arg_str(args, "query") else {
        return "[web_search] ต้องระบุ query".to_string();
    };
    let max = args
        .get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(5)
        .clamp(1, 10) as usize;

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(25))
        .redirects(5)
        .build();
    let resp = match agent
        .get(SEARCH_ENDPOINT)
        .query("q", query)
        .set("User-Agent", WEB_UA)
        .call()
    {
        Ok(r) => r,
        Err(e) => return format!("[web_search] ค้นหาไม่สำเร็จ: {e}"),
    };
    let html = match resp.into_string() {
        Ok(h) => h,
        Err(e) => return format!("[web_search] อ่านผลลัพธ์ไม่สำเร็จ: {e}"),
    };
    parse_ddg_results(&html, max, query)
}

/// แยกผลลัพธ์จากหน้า HTML ของ DuckDuckGo (ไม่มี key ฟรี)
fn parse_ddg_results(html: &str, max: usize, query: &str) -> String {
    // หา title anchors (result__a) และ snippet anchors (result__snippet) พร้อมตำแหน่ง
    let mut titles: Vec<(usize, String, String)> = Vec::new(); // (pos, title, url)
    let mut pos = 0usize;
    while let Some(rel) = html[pos..].find("result__a") {
        let abs = pos + rel;
        if let Some((title, url)) = extract_anchor(&html[abs..]) {
            titles.push((abs, title, url));
        }
        pos = abs + "result__a".len();
    }
    let mut snippets: Vec<(usize, String)> = Vec::new();
    pos = 0;
    while let Some(rel) = html[pos..].find("result__snippet") {
        let abs = pos + rel;
        if let Some(snip) = extract_anchor_text(&html[abs..]) {
            snippets.push((abs, snip));
        }
        pos = abs + "result__snippet".len();
    }

    let mut out = format!("[web_search] ผลการค้นหา '{query}' ({} รายการ):\n", titles.len().min(max));
    for (i, (tpos, title, url)) in titles.iter().take(max).enumerate() {
        let snip = snippets
            .iter()
            .find(|(spos, _)| *spos > *tpos)
            .map(|(_, s)| s.as_str())
            .unwrap_or("");
        out.push_str(&format!("\n{}. {}\n   URL: {}\n   {}\n", i + 1, title, url, snip));
    }
    if titles.is_empty() {
        out.push_str("   (ไม่พบผลลัพธ์ — ลองเปลี่ยนคำค้นหาเป็นภาษาอังกฤษ หรือค้นให้เฉพาะเจาะจงขึ้น)\n");
    }
    out
}

/// จาก HTML หลัง "result__a" ดึง (title, url จริง) ของ anchor
fn extract_anchor(seg: &str) -> Option<(String, String)> {
    let href_start = seg.find("href=\"")? + 6;
    let href_end = seg[href_start..].find('"')? + href_start;
    let href = &seg[href_start..href_end];
    let gt = seg.find('>')?;
    let text_start = gt + 1;
    let close = seg[text_start..].find("</a>")? + text_start;
    let title = html_to_text(&seg[text_start..close]);
    if title.trim().is_empty() {
        return None;
    }
    Some((collapse_ws(&title), decode_ddg_href(href)))
}

/// จาก HTML หลัง "result__snippet" ดึงข้อความของ anchor
fn extract_anchor_text(seg: &str) -> Option<String> {
    let gt = seg.find('>')?;
    let text_start = gt + 1;
    let close = seg[text_start..].find("</a>")? + text_start;
    let t = html_to_text(&seg[text_start..close]);
    if t.trim().is_empty() {
        None
    } else {
        Some(collapse_ws(&t))
    }
}

/// ลิงก์ของ DuckDuckGo เป็น URL เปลี่ยนทาง (uddg=...) — ถอดกลับเป็น URL จริง
fn decode_ddg_href(href: &str) -> String {
    let Some(ui) = href.find("uddg=") else {
        return href.to_string();
    };
    let rest = &href[ui + 5..];
    let end = rest.find('&').unwrap_or(rest.len());
    percent_decode(&rest[..end])
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex_val(b[i + 1]), hex_val(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ---------- read_url ----------

fn read_url(args: &Value) -> String {
    let Some(url) = arg_str(args, "url") else {
        return "[read_url] ต้องระบุ url".to_string();
    };
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return "[read_url] อนุญาตเฉพาะ URL แบบ http:// หรือ https://".to_string();
    }
    let max = args
        .get("max_chars")
        .and_then(|v| v.as_u64())
        .unwrap_or(4000)
        .clamp(1000, 20000) as usize;

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(25))
        .redirects(5)
        .build();
    let resp = match agent.get(url).set("User-Agent", WEB_UA).call() {
        Ok(r) => r,
        Err(e) => return format!("[read_url] เปิดหน้าเว็บไม่สำเร็จ: {e}"),
    };
    let body = match resp.into_string() {
        Ok(b) => b,
        Err(e) => return format!("[read_url] อ่านเนื้อหาไม่สำเร็จ: {e}"),
    };
    if body.as_bytes().contains(&0) {
        return format!("[read_url] {url} — ดูเหมือนไม่ใช่หน้าเว็บ HTML (อาจเป็น PDF/ไฟล์ไบนารี) อ่านเป็นข้อความไม่ได้");
    }
    let text = html_to_text(&body);
    let text = collapse_ws(&text);
    let mut out = format!("[read_url] {url} ({} ตัวอักษร)\n", text.len());
    let clipped = truncate_chars(&text, max);
    out.push_str(&clipped);
    if text.len() > max {
        out.push_str(&format!("\n… (เหลืออีก {} ตัวอักษร ใช้ max_chars มากขึ้นเพื่ออ่านเพิ่ม)", text.len() - max));
    }
    out
}

// ---------- HTML → ข้อความ ----------

/// แปลง HTML คร่าวๆ เป็นข้อความธรรมดา (ลบแท็ก, ขึ้นบรรทัดใหม่ตาม block tag, ถอด entity)
fn html_to_text(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    let mut in_block: Option<&'static str> = None;
    while !rest.is_empty() {
        let Some(i) = rest.find('<') else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let Some(gt) = rest.find('>') else {
            break;
        };
        let tag = &rest[..=gt];
        rest = &rest[gt + 1..];
        let lower = tag.to_lowercase();
        if let Some(end) = in_block {
            if lower.starts_with(end) {
                in_block = None;
            }
            continue;
        }
        if lower.starts_with("<script") {
            in_block = Some("</script>");
        } else if lower.starts_with("<style") {
            in_block = Some("</style>");
        } else if is_block_tag(&lower) {
            out.push('\n');
        }
    }
    decode_entities(&out)
}

fn is_block_tag(lower: &str) -> bool {
    const BLOCKS: &[&str] = &[
        "</p", "</div", "</li", "</ul", "</ol", "</h1", "</h2", "</h3", "</h4", "</h5", "</h6",
        "</tr", "</table", "</section", "</article", "</header", "</footer", "</main", "</pre",
        "<br", "</td", "</th", "</blockquote", "</figure", "</nav", "</form",
    ];
    BLOCKS.iter().any(|b| lower.starts_with(b))
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
}

/// รวมช่องว่าง/ขึ้นบรรทัดใหม่หลายๆ ตัวให้เหลือบรรทัดเดียว (สำหรับ title/snippet)
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

// ---------- open_preview (ดูพรีวิวเว็บในเบราว์เซอร์) ----------

static PREVIEW_URL: OnceLock<String> = OnceLock::new();

pub fn last_preview_url() -> Option<String> {
    PREVIEW_URL.get().cloned()
}

/// สำหรับคำสั่ง /skills ใน REPL
pub fn list_skills_public() -> String {
    list_skills()
}

/// รายการทักษะแบบ structured (สำหรับหน้าตั้งค่า): [{name, description}]
/// — อ่านจากโฟลเดอร์ทักษะทุกที่ เรียงตามชื่อ ไม่ซ้ำกัน
pub fn list_skills_structured() -> Vec<serde_json::Value> {
    let mut seen: Vec<(String, String)> = Vec::new();
    for base in skill_dirs() {
        if !base.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&base) else {
            continue;
        };
        for e in entries.flatten() {
            let sp = e.path().join("SKILL.md");
            if !sp.is_file() {
                continue;
            }
            let (name, desc) = read_skill_meta(&sp);
            if name.is_empty() {
                continue;
            }
            if !seen.iter().any(|(n, _)| *n == name) {
                seen.push((name, desc));
            }
        }
    }
    seen.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    seen.into_iter()
        .map(|(name, desc)| serde_json::json!({ "name": name, "description": desc }))
        .collect()
}

/// อ่านเนื้อหา SKILL.md ของทักษะ (สำหรับใส่ลง system prompt ให้มีผลจริง)
pub fn load_skill_content(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    for base in skill_dirs() {
        let candidates = [base.join(name).join("SKILL.md"), base.join(&lower).join("SKILL.md")];
        for sp in candidates {
            if sp.is_file() {
                return fs::read_to_string(&sp).ok();
            }
        }
        // สุดท้ายไล่หาแบบ case-insensitive
        if base.is_dir() {
            if let Ok(entries) = fs::read_dir(&base) {
                for e in entries.flatten() {
                    if e.file_name().to_string_lossy().to_lowercase() == lower {
                        let sp = e.path().join("SKILL.md");
                        if sp.is_file() {
                            return fs::read_to_string(&sp).ok();
                        }
                    }
                }
            }
        }
    }
    None
}

/// เปิด URL ในเบราว์เซอร์เริ่มต้นของผู้ใช้ (ใช้จากคำสั่ง /preview ได้ด้วย)
pub fn reopen_preview() -> String {
    match last_preview_url() {
        Some(u) => {
            open_browser(&u);
            format!("เปิดพรีวิวอีกครั้ง: {u}")
        }
        None => "ยังไม่มีพรีวิว — บอกให้ Commandblock สร้างหน้าเว็บแล้วใช้ open_preview ก่อน".to_string(),
    }
}

fn open_preview(args: &Value) -> String {
    let html = args.get("html").and_then(|v| v.as_str());
    let path = arg_str(args, "path");
    if html.is_none() && path.is_none() {
        return "[open_preview] ต้องระบุ html หรือ path".to_string();
    }

    let (root, open_file) = if let Some(h) = html {
        let _ = fs::create_dir_all("preview");
        if let Err(e) = fs::write("preview/index.html", h) {
            return format!("[open_preview] เขียน preview/index.html ไม่สำเร็จ: {e}");
        }
        (std::path::PathBuf::from("preview"), "index.html".to_string())
    } else if let Some(p) = path {
        let pb = Path::new(p);
        if pb.is_file() {
            let name = pb
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "index.html".into());
            let root = pb
                .parent()
                .map(|x| x.to_path_buf())
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            (root, name)
        } else if pb.is_dir() {
            // ถ้าไม่มี index.html ให้เปิดไฟล์ .html ตัวแรกที่เจอ
            let open = if pb.join("index.html").is_file() {
                "index.html".to_string()
            } else {
                first_html(pb)
            };
            (pb.to_path_buf(), open)
        } else {
            return format!("[open_preview] ไม่พบไฟล์/โฟลเดอร์: {p}");
        }
    } else {
        return "[open_preview] ต้องระบุ html หรือ path".to_string();
    };

    let port = match start_preview_server(root.clone()) {
        Some(p) => p,
        None => return "[open_preview] เปิดเซิร์ฟเวอร์พรีวิวไม่สำเร็จ (พอร์ตชน/ข้อผิดพลาด)".to_string(),
    };
    let url = format!("http://127.0.0.1:{port}/{open_file}");
    open_browser(&url);
    let _ = PREVIEW_URL.set(url.clone());
    format!(
        "[open_preview] เปิดพรีวิวในเบราว์เซอร์แล้ว: {url}\n(เซิร์ฟเวอร์รันที่ 127.0.0.1:{port} อยู่จนกว่าจะปิด Commandblock — ไฟล์อยู่ในโฟลเดอร์ '{}' ใช้ /preview เพื่อเปิดซ้ำได้)",
        root.display()
    )
}

/// รันเซิร์ฟเวอร์ static ง่ายๆ บน 127.0.0.1:พอร์ตสุ่ม คืนพอร์ตที่ใช้
fn start_preview_server(root: std::path::PathBuf) -> Option<u16> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).ok()?;
    let port = listener.local_addr().ok()?.port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let root = root.clone();
                    std::thread::spawn(move || serve_one(s, &root));
                }
                Err(_) => continue,
            }
        }
    });
    Some(port)
}

fn serve_one(mut stream: TcpStream, root: &Path) {
    let mut buf = [0u8; 8192];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return,
    };
    let req = String::from_utf8_lossy(&buf[..n]).to_string();
    let first = req.lines().next().unwrap_or("");
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");
    if method != "GET" {
        respond(&mut stream, 405, "text/plain", b"Method Not Allowed");
        return;
    }
    let decoded = percent_decode(target);
    let rel = decoded.trim_start_matches('/');
    let rel = if rel.is_empty() || rel.ends_with('/') {
        format!("{rel}index.html")
    } else {
        rel.to_string()
    };
    let full = root.join(&rel);
    // กัน path traversal ออกนอกโฟลเดอร์พรีวิว
    if !full.starts_with(root) {
        respond(&mut stream, 403, "text/plain", b"Forbidden");
        return;
    }
    if full.is_dir() {
        let listing = dir_listing(&full);
        respond(&mut stream, 200, "text/html; charset=utf-8", listing.as_bytes());
        return;
    }
    match fs::read(&full) {
        Ok(data) => {
            let ext = full.extension().and_then(|e| e.to_str()).unwrap_or("");
            let ct = content_type(ext);
            respond(&mut stream, 200, ct, &data);
        }
        Err(_) => {
            // ไฟล์ไม่เจอ — ถ้าเป็นหน้าที่หลัก (root/index) ให้แสดงรายการไฟล์แทน
            if rel == "index.html" || rel.is_empty() {
                let listing = dir_listing(root);
                respond(&mut stream, 200, "text/html; charset=utf-8", listing.as_bytes());
            } else {
                respond(&mut stream, 404, "text/plain", format!("404 Not Found: {rel}").as_bytes());
            }
        }
    }
}

/// หาไฟล์ .html ตัวแรกในโฟลเดอร์ (เรียงตามชื่อ) — ใช้เมื่อไม่มี index.html
fn first_html(dir: &Path) -> String {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut names: Vec<String> = entries
            .flatten()
            .filter(|e| {
                e.file_type().map(|t| t.is_file()).unwrap_or(false)
                    && e.path().extension().and_then(|x| x.to_str()).map(|x| x.eq_ignore_ascii_case("html") || x.eq_ignore_ascii_case("htm")).unwrap_or(false)
            })
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        names.sort();
        if let Some(n) = names.into_iter().next() {
            return n;
        }
    }
    "index.html".to_string()
}

fn respond(stream: &mut TcpStream, status: u16, ctype: &str, body: &[u8]) {
    let reason = match status {
        200 => "OK",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

fn dir_listing(dir: &Path) -> String {
    let mut items = String::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let label = if is_dir { format!("{name}/") } else { name.clone() };
            items.push_str(&format!("<li><a href=\"{label}\">{label}</a></li>\n"));
        }
    }
    format!("<!DOCTYPE html><html><head><meta charset=utf-8><title>Commandblock Preview</title></head><body><h1>📁 {}</h1><ul>\n{items}</ul></body></html>", dir.display())
}

fn content_type(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

pub fn open_browser(url: &str) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = Command::new("cmd")
            .args(["/C", "start", "", url])
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(url).spawn();
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

// ---------- ทักษะ (skills) — โหลดคำแนะนำเฉพาะทางเหมือนผู้ช่วย AI ระดับมืออาชีพ ----------

/// โฟลเดอร์ที่ค้นหาทักษะ: env BUFF_SKILLS_DIR → ./skills → ~/.buff/skills → ~/.agents/skills
fn skill_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(env_dir) = std::env::var("BUFF_SKILLS_DIR") {
        if !env_dir.trim().is_empty() {
            dirs.push(std::path::PathBuf::from(env_dir.trim()));
        }
    }
    dirs.push(std::path::PathBuf::from("skills"));
    if let Some(home) = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|h| std::path::PathBuf::from(h))
    {
        dirs.push(home.join(".buff").join("skills"));
        dirs.push(home.join(".agents").join("skills"));
    }
    dirs
}

fn list_skills() -> String {
    let mut found = 0usize;
    let mut out = String::new();
    for base in skill_dirs() {
        if !base.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&base) else {
            continue;
        };
        for e in entries.flatten() {
            let sp = e.path().join("SKILL.md");
            if !sp.is_file() {
                continue;
            }
            let (name, desc) = read_skill_meta(&sp);
            if name.is_empty() {
                continue;
            }
            found += 1;
            out.push_str(&format!("  • {name} — {}\n", if desc.is_empty() { "(ไม่มีคำอธิบาย)" } else { &desc }));
        }
    }
    if found == 0 {
        "ยังไม่มีทักษะ — สร้างโฟลเดอร์ skills/<ชื่อ>/SKILL.md ในโปรเจกต์ หรือวางใน ~/.buff/skills/ หรือตั้งตัวแปร BUFF_SKILLS_DIR".to_string()
    } else {
        format!("[list_skills] พบทักษะ {found} อย่าง:\n{out}")
    }
}

fn load_skill(args: &Value) -> String {
    let Some(name) = arg_str(args, "name") else {
        return "[load_skill] ต้องระบุ name".to_string();
    };
    let lower = name.to_lowercase();
    for base in skill_dirs() {
        // ลองตรงก่อน แล้วค่อยแบบไม่สนตัวพิมพ์เล็ก/ใหญ่
        let candidates = [
            base.join(name).join("SKILL.md"),
            base.join(&lower).join("SKILL.md"),
        ];
        for sp in candidates {
            if !sp.is_file() {
                continue;
            }
            return match fs::read_to_string(&sp) {
                Ok(t) => format!("[load_skill] '{name}' (จาก {})\n{}\n{}", sp.display(), "---".repeat(30), t),
                Err(e) => format!("[load_skill] อ่าน {sp:?} ไม่สำเร็จ: {e}"),
            };
        }
        // สุดท้ายไล่หาแบบ case-insensitive
        if base.is_dir() {
            if let Ok(entries) = fs::read_dir(&base) {
                for e in entries.flatten() {
                    if e.file_name().to_string_lossy().to_lowercase() == lower {
                        let sp = e.path().join("SKILL.md");
                        if sp.is_file() {
                            return match fs::read_to_string(&sp) {
                                Ok(t) => format!("[load_skill] '{name}' (จาก {})\n{}\n{}", sp.display(), "---".repeat(30), t),
                                Err(e) => format!("[load_skill] อ่าน {sp:?} ไม่สำเร็จ: {e}"),
                            };
                        }
                    }
                }
            }
        }
    }
    format!("[load_skill] ไม่พบทักษะ '{name}' — ใช้ list_skills เพื่อดูรายชื่อที่มี")
}

/// อ่านชื่อ + คำอธิบายจาก frontmatter (--- name: ... description: ... ---) ของ SKILL.md
fn read_skill_meta(path: &Path) -> (String, String) {
    let Ok(text) = fs::read_to_string(path) else {
        return (String::new(), String::new());
    };
    let mut name = String::new();
    let mut desc = String::new();
    if let Some(body) = text.strip_prefix("---") {
        if let Some(end) = body.find("---") {
            for line in body[..end].lines() {
                let line = line.trim();
                if let Some(v) = line.strip_prefix("name:") {
                    name = v.trim().trim_matches('"').to_string();
                } else if let Some(v) = line.strip_prefix("description:") {
                    desc = v.trim().trim_matches('"').to_string();
                }
            }
        }
    }
    if name.is_empty() {
        name = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
    }
    (name, desc)
}
