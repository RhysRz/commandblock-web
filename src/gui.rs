//! GUI ของ Commandblock — เซิร์ฟเวอร์ local + หน้าแชทในเบราว์เซอร์ (หน้าตาเหมือนแชท AI)
//!
//! Commandblock.exe (ไม่มีอาร์กิวเมนต์) → รันเซิร์ฟเวอร์นี้แล้วเปิดเบราว์เซอร์อัตโนมัติ
//! - GET  /            หน้าแชท (ui.html)
//! - GET  /api/state   สถานะแบ็กเอนด์/โมเดล/จำนวนข้อความ
//! - POST /api/chat    ส่งข้อความ → สตรีมคำตอบกลับเป็น SSE (content/tool/note/done)

use crate::{config, connector, remote, tools, update, TurnSink};
use image::GenericImageView;
use serde_json::{json, Value};
use std::io::{BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

const UI_HTML: &str = include_str!("ui.html");
const COMMAND_BLOCK_ICON_PNG: &[u8] = include_bytes!("../assets/buff-command-block.png");
const SETTINGS_FILE: &str = ".freebuff/settings.json";
const STARTUP_LOG_FILE: &str = ".freebuff/startup_log.json";
const STARTUP_LOG_MAX: usize = 60; // เก็บประวัติสูงสุด 60 รายการ

struct Shared {
    history: Vec<Value>,
    plan: Option<String>,
    eff: config::Effective,              // ค่าฐานจาก config.json
    cfg_models: Vec<config::ModelEntry>, // รายการโมเดลจาก config.json
    model: String,                       // โมเดลที่กำลังใช้ (สลับได้ runtime)
    base_url: String,                    // base_url ที่กำลังใช้ (สลับได้ runtime)
    // Project settings (Project settings modal → .freebuff/settings.json)
    startup_script: String,       // คำสั่งที่รันเทิร์นแรกของแต่ละวัน
    startup_last_run: String,     // วันที่ (YYYY-MM-DD) ที่รันล่าสุด — จดใน settings.json
    startup_note: Option<String>, // ผลลัพธ์ของ startup script (ใส่ลง system prompt)
    skills: Vec<String>,          // ทักษะที่เปิดใช้ (preloaded) — SKILL.md ถูกใส่ลง system prompt
    scan_context: Option<String>, // บริบทโฟลเดอร์ที่ผู้ใช้เปิด (อ่านทั้งโฟลเดอร์)
    folder_name: String,          // ชื่อโฟลเดอร์ปัจจุบัน (แสดงใต้ช่องส่ง) — เริ่มจากโฟลเดอร์โปรเจกต์
    folder_path: String,          // พาธเต็มของโฟลเดอร์ปัจจุบัน
}

impl Shared {
    /// ค่าที่ใช้จริงตอนนี้ (โมเดล/base_url อาจถูกสลับแล้ว)
    fn current_eff(&self) -> config::Effective {
        let mut e = self.eff.clone();
        if !self.model.is_empty() {
            e.model = self.model.clone();
        }
        if !self.base_url.is_empty() {
            e.base_url = self.base_url.clone();
        }
        e
    }

    /// สร้าง system prompt ใหม่ = พื้นฐาน + ทักษะที่เปิดใช้ (อ่าน SKILL.md จริง) + บริบทโฟลเดอร์ + ผล startup
    fn rebuild_system(&mut self) {
        if let Some(first) = self.history.first_mut() {
            first["content"] = json!(system_with_extras(
                &self.skills,
                self.scan_context.as_deref(),
                self.startup_note.as_deref()
            ));
        }
    }
}

/// โหลด Project settings จาก .freebuff/settings.json → (startup_script, skills, startup_last_run)
fn load_settings() -> (String, Vec<String>, String) {
    let Ok(text) = std::fs::read_to_string(SETTINGS_FILE) else {
        return (String::new(), Vec::new(), String::new());
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return (String::new(), Vec::new(), String::new());
    };
    let script = v
        .get("startup_script")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let skills: Vec<String> = v
        .get("skills")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let last_run = v
        .get("startup_last_run")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    (script, skills, last_run)
}

/// อ่านประวัติ startup script (รายการล่าสุดอยู่ท้ายสุด)
fn load_startup_log() -> Vec<Value> {
    let Ok(text) = std::fs::read_to_string(STARTUP_LOG_FILE) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<Value>>(&text).unwrap_or_default()
}

/// เพิ่มรายการลงประวัติ startup script (เก็บสูงสุด STARTUP_LOG_MAX รายการ)
fn append_startup_log(date: &str, script: &str, output: &str, ok: bool) {
    let mut log = load_startup_log();
    log.push(json!({
        "date": date,
        "time": now_hhmm(),
        "script": script,
        "output": output,
        "ok": ok,
    }));
    if log.len() > STARTUP_LOG_MAX {
        let start = log.len() - STARTUP_LOG_MAX;
        log = log[start..].to_vec();
    }
    let _ = std::fs::create_dir_all(".freebuff");
    if let Ok(text) = serde_json::to_string_pretty(&log) {
        let _ = std::fs::write(STARTUP_LOG_FILE, text);
    }
}

/// บันทึก Project settings ลง .freebuff/settings.json (รวมวันที่ startup รันล่าสุด)
fn write_settings(script: &str, skills: &[String], last_run: &str) -> bool {
    let _ = std::fs::create_dir_all(".freebuff");
    let v = json!({ "startup_script": script, "skills": skills, "startup_last_run": last_run });
    std::fs::write(
        SETTINGS_FILE,
        serde_json::to_string_pretty(&v).unwrap_or_default(),
    )
    .is_ok()
}

/// วัน/เวลาท้องถิ่น (ปี, เดือน, วัน, ชั่วโมง, นาที)
/// Windows: ใช้ GetLocalTime (kernel32, ไม่ต้องมี dependency), อื่นๆ: คำนวณจาก epoch (UTC)
fn local_parts() -> (u16, u16, u16, u16, u16) {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct SystemTime {
            year: u16,
            month: u16,
            dow: u16,
            day: u16,
            hour: u16,
            min: u16,
            sec: u16,
            ms: u16,
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetLocalTime(t: *mut SystemTime);
        }
        let mut t: SystemTime = unsafe { std::mem::zeroed() };
        unsafe { GetLocalTime(&mut t) };
        return (t.year, t.month, t.day, t.hour, t.min);
    }
    #[cfg(not(windows))]
    {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let (y, m, d) = civil_from_epoch_days((secs / 86400) as i64);
        let h = ((secs % 86400) / 3600) as u16;
        let mi = ((secs % 3600) / 60) as u16;
        (y, m, d, h, mi)
    }
}

/// Howard Hinnant civil_from_days — แปลงวันนับจาก epoch → (ปี, เดือน, วัน)
#[cfg(not(windows))]
fn civil_from_epoch_days(days: i64) -> (u16, u16, u16) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y } as u32;
    (y as u16, m as u16, d as u16)
}

/// วันที่ท้องถิ่นวันนี้ (YYYY-MM-DD)
fn today_local() -> String {
    let (y, m, d, _, _) = local_parts();
    format!("{y:04}-{m:02}-{d:02}")
}

/// เวลาท้องถิ่นตอนนี้ (HH:MM)
fn now_hhmm() -> String {
    let (_, _, _, h, mi) = local_parts();
    format!("{h:02}:{mi:02}")
}

/// system prompt พื้นฐาน + ส่วนเสริม (ทักษะ/บริบทโฟลเดอร์/startup) — ให้ทักษะมีผลจริงกับ AI
fn system_with_extras(skills: &[String], scan: Option<&str>, startup: Option<&str>) -> String {
    let mut s = crate::system_prompt();
    if !skills.is_empty() {
        s.push_str("\n\n## ทักษะที่เปิดใช้งาน (Preloaded skills)\n");
        s.push_str("ผู้ใช้เปิดใช้ทักษะเหล่านี้อยู่ — อ่านและปฏิบัติตามคำแนะนำของมันโดยอัตโนมัติเมื่องานเกี่ยวข้อง (ห้ามโหลดซ้ำด้วย load_skill):\n");
        for name in skills {
            let content = tools::load_skill_content(name)
                .map(|c| clip_skill(&c))
                .unwrap_or_else(|| "(หา SKILL.md ไม่พบ — ลอง list_skills ดู)".to_string());
            s.push_str(&format!("\n### ทักษะ: {name}\n{content}\n"));
        }
    }
    if let Some(ctx) = scan {
        if !ctx.trim().is_empty() {
            s.push_str(&format!(
                "\n\n## บริบทโฟลเดอร์ที่ผู้ใช้เปิด (อ่านทั้งโฟลเดอร์แล้ว)\n{ctx}\n"
            ));
        }
    }
    if let Some(n) = startup {
        if !n.trim().is_empty() {
            s.push_str(&format!("\n\n## Startup script (รันไปแล้ว)\n{n}\n"));
        }
    }
    s
}

/// ตัดเนื้อหา SKILL.md ให้พอดีกับ context (กันบวม)
fn clip_skill(s: &str) -> String {
    const MAX: usize = 3500;
    if s.len() <= MAX {
        return s.to_string();
    }
    let mut end = MAX;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…(เนื้อหาถูกตัด)", &s[..end])
}

/// เดินโฟลเดอร์จริงบนดิสก์ → [{path, content}] สำหรับ build_scan_context
/// (ข้ามโฟลเดอร์ใหญ่/ซ่อน, ข้ามไฟล์ไบนารี/ใหญ่ — อ่านเป็นข้อความไม่ได้จะถูกข้ามเอง)
fn walk_folder_files(root: &std::path::Path) -> Vec<Value> {
    const MAX_FILES: usize = 150;
    const MAX_BYTES: u64 = 300 * 1024;
    const MAX_PER: usize = 8000;
    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    fn walk(
        dir: &std::path::Path,
        prefix: &str,
        root_name: &str,
        out: &mut Vec<Value>,
        depth: usize,
    ) {
        if out.len() >= MAX_FILES || depth > 8 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut list: Vec<_> = entries.flatten().collect();
        list.sort_by_key(|e| e.file_name());
        for e in list {
            if out.len() >= MAX_FILES {
                break;
            }
            let name = e.file_name().to_string_lossy().to_string();
            if FILE_SKIP_DIRS.contains(&name.as_str()) || name.starts_with('.') {
                continue;
            }
            let rel = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            let Ok(ft) = e.file_type() else {
                continue;
            };
            if ft.is_dir() {
                walk(&e.path(), &rel, root_name, out, depth + 1);
            } else if ft.is_file() {
                let Ok(meta) = e.metadata() else {
                    continue;
                };
                if meta.len() > MAX_BYTES {
                    continue; // ใหญ่เกิน — ข้าม (กันไบนารี/ไฟล์โหลดมาก)
                }
                let Ok(content) = std::fs::read_to_string(e.path()) else {
                    continue; // อ่านเป็นข้อความไม่ได้ (ไบนารี) — ข้าม
                };
                let clipped: String = content.chars().take(MAX_PER).collect();
                out.push(json!({ "path": format!("{root_name}/{rel}"), "content": clipped }));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, "", &root_name, &mut out, 0);
    out
}

/// สร้างบริบทจากไฟล์ที่เลือกในโฟลเดอร์ (จาก webkitdirectory picker) → (context, root, file_count, char_count)
/// - ไฟล์ภายในโปรเจกต์ปัจจุบัน (cwd): ใช้พาธสัมพัทธ์ → Commandblock แก้ไฟล์ต่อได้เลย
/// - ไฟล์นอกโปรเจกต์: เก็บเป็นบริบทอ่านอย่างเดียว
fn build_scan_context(files: &[Value]) -> (String, String, usize, usize) {
    const MAX_FILES: usize = 150;
    const MAX_PER_FILE: usize = 4000;
    let mut paths: Vec<String> = Vec::new();
    let mut contents: Vec<(String, String)> = Vec::new();
    let mut root = String::new();
    let mut total_chars = 0usize;
    let mut in_project = false;
    for f in files.iter().take(MAX_FILES) {
        let p = f
            .get("path")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let c = f
            .get("content")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        if p.is_empty() {
            continue;
        }
        if root.is_empty() {
            root = p.split(['/', '\\']).next().unwrap_or("").to_string();
            // ตรวจว่าโฟลเดอร์ที่เปิดอยู่ในโปรเจกต์ปัจจุบันหรือไม่ (ตัดชื่อ root ออกแล้วอยู่ใน cwd)
            let rest = p.splitn(2, ['/', '\\']).nth(1).unwrap_or("").to_string();
            in_project = !rest.is_empty() && std::path::Path::new(&rest).exists()
                || std::path::Path::new(&p).exists();
        }
        paths.push(p.clone());
        if !c.is_empty() {
            // ลองพาธตรงก่อน ถ้าไม่อยู่ใน cwd ให้ตัด segment แรก (ชื่อโฟลเดอร์ที่เลือก) ออก
            let mut rel = p.clone();
            if !std::path::Path::new(&rel).exists() {
                let parts: Vec<&str> = p.split(['/', '\\']).collect();
                if parts.len() > 1 {
                    let rest = parts[1..].join("/");
                    if std::path::Path::new(&rest).exists() {
                        rel = rest;
                    }
                }
            }
            let clipped = if c.len() > MAX_PER_FILE {
                let mut end = MAX_PER_FILE;
                while !c.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}…(ตัด)", &c[..end])
            } else {
                c.clone()
            };
            total_chars += clipped.len();
            contents.push((rel, clipped));
        }
    }
    let mut out = String::new();
    out.push_str(&format!(
        "โฟลเดอร์: {} — อ่านแล้ว {} ไฟล์ (เนื้อหา {} ตัวอักษร)\n",
        if root.is_empty() {
            "(ไม่ทราบชื่อ)"
        } else {
            &root
        },
        paths.len(),
        total_chars
    ));
    out.push_str(&format!(
        "อยู่ในโปรเจกต์ปัจจุบัน: {} — แก้ไฟล์ในโฟลเดอร์นี้ได้ด้วยพาธสัมพัทธ์ (อ่านไฟล์ก่อนแก้เสมอ)\n",
        if in_project {
            "ใช่"
        } else {
            "ไม่ (อ่านเป็นบริบทอย่างเดียว — การแก้ไฟล์ทำงานในโฟลเดอร์โปรเจกต์ปัจจุบัน)"
        }
    ));
    out.push_str("\nรายการไฟล์ทั้งหมด:\n");
    for p in &paths {
        out.push_str(&format!("- {p}\n"));
    }
    if !contents.is_empty() {
        out.push_str("\nเนื้อหาไฟล์ (อ่านเพื่อทำความเข้าใจโครงสร้างแล้วตอบ/แก้ต่อได้):\n");
        for (rel, c) in &contents {
            out.push_str(&format!("\n===== {rel} =====\n{c}\n"));
        }
    }
    (out, root, paths.len(), total_chars)
}

/// รัน GUI เป็นแอปเดสก์ท็อปจริง (หน้าต่างของระบบ ไม่ใช่แท็บเบราว์เซอร์)
/// เซิร์ฟเวอร์ local รันใน thread พื้นหลัง — หน้าต่างปิด = ปิดโปรแกรม
pub fn serve(
    agent: ureq::Agent,
    eff: &config::Effective,
    cfg_models: Vec<config::ModelEntry>,
) -> ! {
    let (listener, port) = bind_free();
    let url = format!("http://127.0.0.1:{port}/");

    let mut history = vec![json!({"role": "system", "content": crate::system_prompt()})];
    history.extend(crate::load_session());
    let (startup_script, skills, startup_last_run) = load_settings();
    // ชื่อ/พาธโฟลเดอร์โปรเจกต์ (cwd) — ใช้เป็นค่าเริ่มต้นของตัวแสดงโฟลเดอร์
    let (cwd_name, cwd_path) = {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let name = cwd
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "โฟลเดอร์โปรเจกต์".to_string());
        (name, cwd.to_string_lossy().to_string())
    };
    let shared = Arc::new(Mutex::new(Shared {
        history,
        plan: None,
        eff: eff.clone(),
        cfg_models,
        model: eff.model.clone(),
        base_url: eff.base_url.clone(),
        startup_script,
        startup_last_run,
        startup_note: None,
        skills,
        scan_context: None,
        folder_name: cwd_name,
        folder_path: cwd_path,
    }));
    // ใส่ทักษะที่ตั้งไว้ (preloaded) + บริบทโฟลเดอร์ลงใน system prompt ตั้งแต่เริ่ม
    shared.lock().unwrap().rebuild_system();

    // เซิร์ฟเวอร์รันใน thread (ไม่บล็อกหน้าต่าง)
    let eff_owned = eff.clone();
    let srv_shared = Arc::clone(&shared);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let shared = Arc::clone(&srv_shared);
                    let agent = agent.clone();
                    let eff = eff_owned.clone();
                    std::thread::spawn(move || handle(s, &agent, &eff, shared));
                }
                Err(_) => continue,
            }
        }
    });

    // เปิดหน้าต่างเดสก์ท็อป (ถ้า WebView2 ไม่มี ให้เปิดเบราว์เซอร์แทน)
    match run_desktop_window(&url) {
        Ok(()) => {
            std::process::exit(0);
        }
        Err(e) => {
            println!();
            println!("⚠️ เปิดหน้าต่างแอปไม่ได้ ({e}) — เปิดในเบราว์เซอร์แทน: {url}");
            println!("   (ปิดหน้าต่างนี้หรือกด Ctrl+C เพื่อปิด CommandBlock)\n");
            tools::open_browser(&url);
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
    }
}

/// เปิดหน้าต่างแอปเดสก์ท็อปจริงด้วย WebView2 (Windows) / WebKit (macOS/Linux)
fn run_desktop_window(url: &str) -> Result<(), String> {
    use winit::application::ApplicationHandler;
    use winit::dpi::LogicalSize;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    use winit::window::{Icon, Window, WindowId};

    struct App {
        url: String,
        icon: Option<Icon>,
        window: Option<Window>,
        webview: Option<wry::WebView>,
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let attrs = Window::default_attributes()
                .with_title("🤖 CommandBlock — ผู้ช่วยพัฒนาโค้ด AI")
                .with_inner_size(LogicalSize::new(1000.0, 720.0))
                .with_min_inner_size(LogicalSize::new(480.0, 360.0));
            let attrs = match &self.icon {
                Some(i) => attrs.with_window_icon(Some(i.clone())),
                None => attrs,
            };
            let window = match event_loop.create_window(attrs) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("สร้างหน้าต่างไม่สำเร็จ: {e}");
                    event_loop.exit();
                    return;
                }
            };
            // เก็บข้อมูล WebView2 ไว้ที่ AppData (ไม่ให้รกโฟลเดอร์โปรเจกต์)
            let mut ctx_holder = user_data_dir().map(|d| wry::WebContext::new(Some(d)));
            let builder = match ctx_holder.as_mut() {
                Some(ctx) => wry::WebViewBuilder::new_with_web_context(ctx),
                None => wry::WebViewBuilder::new(),
            };
            let webview = match builder.with_url(&self.url).build(&window) {
                Ok(wv) => wv,
                Err(e) => {
                    eprintln!("สร้าง WebView ไม่สำเร็จ: {e}");
                    event_loop.exit();
                    return;
                }
            };
            self.window = Some(window);
            self.webview = Some(webview);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _id: WindowId,
            event: WindowEvent,
        ) {
            if let WindowEvent::CloseRequested = event {
                event_loop.exit();
            }
        }
    }

    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        url: url.to_string(),
        icon: build_icon(),
        window: None,
        webview: None,
    };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    Ok(())
}

/// โฟลเดอร์เก็บข้อมูล WebView2 — %LOCALAPPDATA%\\buff\\webview
fn user_data_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("TEMP"))
        .map(|d| std::path::PathBuf::from(d).join("buff").join("webview"))
}

/// โหลดไอคอน Command Block สีส้มจาก PNG ที่ฝังมากับโปรแกรม
fn build_icon() -> Option<winit::window::Icon> {
    let image =
        image::load_from_memory_with_format(COMMAND_BLOCK_ICON_PNG, image::ImageFormat::Png)
            .ok()?;
    let (width, height) = image.dimensions();
    winit::window::Icon::from_rgba(image.to_rgba8().into_raw(), width, height).ok()
}

/// ข้อความแสดงผลและ allowlist ของปุ่ม Quick action ใน Terminal
fn desktop_mode_name(mode: &str) -> Result<&'static str, String> {
    match mode {
        "connector" => Ok("Desktop Connector"),
        "remote" => Ok("Remote PC"),
        _ => Err("โหมด Desktop ไม่ถูกต้อง".to_string()),
    }
}

/// เปิด sidecar เทียบเท่ากับคำสั่ง Commandblock.exe --connector / --remote
fn launch_desktop_mode(mode: &str) -> Result<&'static str, String> {
    let name = desktop_mode_name(mode)?;
    match mode {
        "connector" => connector::launch_sidecar()?,
        "remote" => remote::launch_sidecar()?,
        _ => unreachable!("desktop_mode_name already rejected this mode"),
    }
    Ok(name)
}

#[cfg(test)]
mod icon_tests {
    use super::{build_icon, desktop_mode_name};

    #[test]
    fn embeds_a_valid_command_block_window_icon() {
        assert!(build_icon().is_some());
    }

    #[test]
    fn desktop_launcher_accepts_only_connector_and_remote() {
        assert_eq!(desktop_mode_name("connector").unwrap(), "Desktop Connector");
        assert_eq!(desktop_mode_name("remote").unwrap(), "Remote PC");
        assert!(desktop_mode_name("cmd /c whoami").is_err());
    }
}

fn bind_free() -> (TcpListener, u16) {
    for port in 8787..8900 {
        if let Ok(l) = TcpListener::bind(("127.0.0.1", port)) {
            return (l, port);
        }
    }
    let l = TcpListener::bind(("127.0.0.1", 0)).expect("ไม่สามารถเปิดพอร์ตเซิร์ฟเวอร์ GUI ได้");
    let port = l.local_addr().map(|a| a.port()).unwrap_or(0);
    (l, port)
}

// ---------- HTTP handler ----------

fn handle(
    mut stream: TcpStream,
    agent: &ureq::Agent,
    eff: &config::Effective,
    shared: Arc<Mutex<Shared>>,
) {
    // อ่าน header + body (จำกัดขนาดกันระเบิด)
    let mut head = Vec::new();
    let mut buf = [0u8; 4096];
    let header_end;
    loop {
        match stream.read(&mut buf) {
            Ok(0) => return,
            Ok(n) => {
                head.extend_from_slice(&buf[..n]);
                if let Some(p) = find_subslice(&head, b"\r\n\r\n") {
                    header_end = p;
                    break;
                }
                if head.len() > 64 * 1024 {
                    return;
                }
            }
            Err(_) => return,
        }
    }

    let header = String::from_utf8_lossy(&head[..header_end]).to_string();
    let mut lines = header.lines();
    let first = lines.next().unwrap_or("");
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path_q = parts.next().unwrap_or("/");
    let path = path_q.split('?').next().unwrap_or("/");

    let mut content_length = 0usize;
    for l in lines {
        let lower = l.to_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    if content_length > 1024 * 1024 {
        return; // body ใหญ่เกิน
    }

    let mut body = head[header_end + 4..].to_vec();
    while body.len() < content_length {
        let mut b = [0u8; 8192];
        match stream.read(&mut b) {
            Ok(0) => break,
            Ok(n) => body.extend_from_slice(&b[..n]),
            Err(_) => break,
        }
    }
    let body_str = String::from_utf8_lossy(&body).to_string();

    let mut out = BufWriter::new(stream);
    match (method.as_str(), path) {
        ("GET", "/") => respond(
            &mut out,
            200,
            "text/html; charset=utf-8",
            UI_HTML.as_bytes(),
        ),
        ("GET", "/assets/buff-command-block.png") => {
            respond(&mut out, 200, "image/png", COMMAND_BLOCK_ICON_PNG)
        }
        ("GET", "/favicon.ico") => respond(&mut out, 204, "text/plain", b""),
        ("GET", "/api/state") => {
            let g = shared.lock().unwrap();
            let cur = g.current_eff();
            let state = json!({
                "version": crate::VERSION,
                "backend": eff.backend.label(),
                "model": cur.model,
                "base_url": cur.base_url,
                "session_messages": g.history.len().saturating_sub(1),
                "skill_count": skill_count(),
                "preview_url": tools::last_preview_url().unwrap_or_default(),
                "changed_count": tools::CHANGED_FILES.lock().unwrap().len(),
                "plan": g.plan.clone().unwrap_or_default(),
                "folder": g.folder_name.clone(),
                "folder_path": g.folder_path.clone(),
            });
            respond(
                &mut out,
                200,
                "application/json",
                state.to_string().as_bytes(),
            );
        }
        ("GET", "/api/models") => {
            let list = build_models_list(agent, &shared);
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "models": list }).to_string().as_bytes(),
            );
        }
        ("GET", "/api/update") => {
            let state = update::status_json();
            respond(
                &mut out,
                200,
                "application/json",
                state.to_string().as_bytes(),
            );
        }
        ("POST", "/api/update") => {
            let action = serde_json::from_str::<Value>(&body_str)
                .ok()
                .and_then(|value| {
                    value
                        .get("action")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .unwrap_or_default();
            let result = if action == "download" {
                update::download_available_release_async().map(|_| update::status_json())
            } else {
                Err("ไม่รู้จักคำสั่งอัปเดต".to_string())
            };
            match result {
                Ok(state) => respond(
                    &mut out,
                    202,
                    "application/json",
                    state.to_string().as_bytes(),
                ),
                Err(message) => respond(
                    &mut out,
                    409,
                    "application/json",
                    json!({"error": message}).to_string().as_bytes(),
                ),
            }
        }
        ("POST", "/api/model") => {
            let sel = serde_json::from_str::<Value>(&body_str)
                .ok()
                .and_then(|v| v["model"].as_str().map(|s| s.trim().to_string()))
                .unwrap_or_default();
            let sel_base = serde_json::from_str::<Value>(&body_str)
                .ok()
                .and_then(|v| v["base_url"].as_str().map(|s| s.trim().to_string()))
                .unwrap_or_default();
            let mut g = shared.lock().unwrap();
            let mut list = build_models_list_inner(agent, &g);
            let found = list.iter_mut().find(|m| m["name"] == sel);
            if let Some(f) = found {
                g.model = sel.clone();
                g.base_url = f["base_url"].as_str().unwrap_or("").to_string();
                // ถ้าโมเดลนั้นมี api_key ของตัวเอง (เช่น Gemini/Groq) → ใช้ key นั้นแทน key หลัก
                if let Some(cm) = g
                    .cfg_models
                    .iter()
                    .find(|m| m.name == sel && m.base_url == g.base_url)
                {
                    if !cm.api_key.is_empty() {
                        g.eff.api_key = cm.api_key.clone();
                    }
                }
                g.eff.backend = config::Backend::OpenAI;
            } else if !sel.is_empty() {
                // อนุญาตเลือกอิสระ (เช่น อยากพิมพ์ชื่อเอง)
                g.model = sel;
                if !sel_base.is_empty() {
                    g.base_url = sel_base;
                }
                g.eff.backend = config::Backend::OpenAI;
            } else {
                respond(
                    &mut out,
                    400,
                    "application/json",
                    json!({"ok": false, "error": "ไม่พบโมเดล" })
                        .to_string()
                        .as_bytes(),
                );
                return;
            }
            let cur = g.current_eff();
            respond(&mut out, 200, "application/json", json!({"ok": true, "backend": cur.backend.label(), "model": g.model, "base_url": g.base_url }).to_string().as_bytes());
        }
        ("GET", "/api/history") => {
            let g = shared.lock().unwrap();
            let prompts: Vec<String> = g
                .history
                .iter()
                .filter(|m| m["role"] == "user")
                .filter_map(|m| m["content"].as_str().map(|s| s.to_string()))
                .collect();
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "prompts": prompts }).to_string().as_bytes(),
            );
        }
        ("GET", "/api/files") => {
            let files = list_project_files();
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "files": files }).to_string().as_bytes(),
            );
        }
        ("GET", "/api/read") => {
            let path = path_q
                .split('?')
                .nth(1)
                .map(|q| {
                    q.split('&')
                        .find(|kv| kv.starts_with("path="))
                        .map(|kv| percent_decode_gui(&kv[5..]))
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let result = if path.is_empty() {
                "ต้องระบุ path".to_string()
            } else {
                tools::execute("read_file", &json!({ "path": path }), &mut None)
            };
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "path": path, "content": result })
                    .to_string()
                    .as_bytes(),
            );
        }
        ("GET", "/api/changes") => {
            let changed = tools::CHANGED_FILES.lock().unwrap().clone();
            let changes: Vec<Value> = changed
                .iter()
                .map(|e| {
                    json!({
                        "path": e.path,
                        "status": e.status,
                        "added": e.added,
                        "deleted": e.deleted
                    })
                })
                .collect();
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "changes": changes }).to_string().as_bytes(),
            );
        }
        ("GET", "/api/queue") => {
            let activity = tools::ACTIVITY.lock().unwrap().clone();
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "activity": activity }).to_string().as_bytes(),
            );
        }
        ("GET", "/api/notes") => {
            let notes = std::fs::read_to_string("notes.md").unwrap_or_default();
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "notes": notes }).to_string().as_bytes(),
            );
        }
        ("POST", "/api/notes") => {
            let notes = serde_json::from_str::<Value>(&body_str)
                .ok()
                .and_then(|v| v["notes"].as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let ok = std::fs::write("notes.md", notes).is_ok();
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "saved": ok }).to_string().as_bytes(),
            );
        }
        ("GET", "/api/settings") => {
            let g = shared.lock().unwrap();
            respond(
                &mut out,
                200,
                "application/json",
                json!({
                    "startup_script": g.startup_script,
                    "skills": g.skills,
                    "available_skills": tools::list_skills_structured(),
                    "path": SETTINGS_FILE,
                    "scan": g.scan_context.as_ref().map(|c| c.lines().next().unwrap_or("").to_string()).unwrap_or_default(),
                })
                .to_string()
                .as_bytes(),
            );
        }
        ("POST", "/api/settings") => {
            let v: Value = serde_json::from_str(&body_str).unwrap_or(Value::Null);
            let script = v
                .get("startup_script")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let skills: Vec<String> = v
                .get("skills")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            let mut g = shared.lock().unwrap();
            let saved = write_settings(&script, &skills, &g.startup_last_run);
            g.startup_script = script;
            g.skills = skills;
            g.rebuild_system();
            respond(
                &mut out,
                200,
                "application/json",
                json!({"ok": saved, "saved": saved}).to_string().as_bytes(),
            );
        }
        ("GET", "/api/startup-log") => {
            let log = load_startup_log();
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "log": log }).to_string().as_bytes(),
            );
        }
        ("POST", "/api/pick-folder") => {
            // กล่องเลือกโฟลเดอร์แบบ native (ของ Windows — ไม่ใช่ป๊อปอัปเว็บ)
            let picked = rfd::FileDialog::new()
                .set_title("เลือกโฟลเดอร์ — CommandBlock จะอ่านไฟล์ทั้งโฟลเดอร์")
                .pick_folder();
            match picked {
                Some(dir) => {
                    let files = walk_folder_files(&dir);
                    let (ctx, root, file_count, char_count) = build_scan_context(&files);
                    let mut g = shared.lock().unwrap();
                    g.scan_context = Some(ctx);
                    g.rebuild_system();
                    g.folder_name = dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| root.clone());
                    g.folder_path = dir.to_string_lossy().to_string();
                    respond(
                        &mut out,
                        200,
                        "application/json",
                        json!({
                            "ok": true,
                            "path": dir.to_string_lossy().to_string(),
                            "root": root,
                            "files": file_count,
                            "chars": char_count,
                        })
                        .to_string()
                        .as_bytes(),
                    );
                }
                None => respond(
                    &mut out,
                    200,
                    "application/json",
                    json!({"ok": false, "cancelled": true})
                        .to_string()
                        .as_bytes(),
                ),
            }
        }
        ("POST", "/api/scan") => {
            let v: Value = serde_json::from_str(&body_str).unwrap_or(Value::Null);
            let files = v
                .get("files")
                .and_then(|x| x.as_array())
                .cloned()
                .unwrap_or_default();
            let (ctx, root, file_count, char_count) = build_scan_context(&files);
            let mut g = shared.lock().unwrap();
            g.scan_context = Some(ctx);
            g.rebuild_system();
            respond(
                &mut out,
                200,
                "application/json",
                json!({"ok": true, "root": root, "files": file_count, "chars": char_count})
                    .to_string()
                    .as_bytes(),
            );
        }
        ("POST", "/api/exec") => {
            let cmd = serde_json::from_str::<Value>(&body_str)
                .ok()
                .and_then(|v| v["command"].as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let output = if cmd.is_empty() {
                "(ว่าง)".to_string()
            } else {
                tools::execute(
                    "run_command",
                    &json!({ "command": cmd, "timeout_seconds": 60 }),
                    &mut None,
                )
            };
            respond(
                &mut out,
                200,
                "application/json",
                json!({ "output": output }).to_string().as_bytes(),
            );
        }
        ("POST", "/api/desktop-mode") => {
            let mode = serde_json::from_str::<Value>(&body_str)
                .ok()
                .and_then(|value| value.get("mode").and_then(Value::as_str).map(str::to_owned))
                .unwrap_or_default();
            match launch_desktop_mode(&mode) {
                Ok(name) => respond(
                    &mut out,
                    200,
                    "application/json",
                    json!({"ok": true, "message": format!("เปิด {name} แล้ว — ลงชื่อเข้าใช้ในหน้าต่างที่เปิดขึ้น")}).to_string().as_bytes(),
                ),
                Err(error) => respond(
                    &mut out,
                    400,
                    "application/json",
                    json!({"ok": false, "error": error}).to_string().as_bytes(),
                ),
            }
        }
        ("POST", "/api/chat") => handle_chat(&mut out, agent, eff, &body_str, &shared),
        _ => respond(&mut out, 404, "text/plain", b"404 Not Found"),
    }
}

fn handle_chat(
    out: &mut BufWriter<TcpStream>,
    agent: &ureq::Agent,
    eff: &config::Effective,
    body: &str,
    shared: &Mutex<Shared>,
) {
    let message = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["message"].as_str().map(|s| s.trim().to_string()))
        .unwrap_or_default();
    // รูปที่แนบ/วาง (Ctrl+V): [{mime, data(base64)}] → ส่งเป็น multimodal (content array)
    let images: Vec<(String, String)> = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["images"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .take(4) // จำกัด 4 รูป/ครั้ง
        .filter_map(|i| {
            let mime = i
                .get("mime")
                .and_then(|x| x.as_str())
                .unwrap_or("image/png")
                .to_string();
            let data = i
                .get("data")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            if data.is_empty() {
                None
            } else {
                Some((mime, data))
            }
        })
        .collect();
    if message.is_empty() && images.is_empty() {
        respond(out, 400, "text/plain", "ต้องระบุ message".as_bytes());
        return;
    }

    // ตอบ SSE
    write!(out, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nX-Accel-Buffering: no\r\nConnection: close\r\n\r\n").ok();
    let _ = out.flush();

    // คำสั่ง / — ตอบตรงโดยไม่ต้องใช้ AI
    if let Some(text) = try_command(&message, eff, shared) {
        let _ = sse(out, "content", json!({"t": text}));
        let _ = sse(out, "done", json!({"ok": true}));
        return;
    }

    // Startup script (ตั้งค่าใน Project settings) — รันเทิร์นแรกของแต่ละวัน
    // (เช็ควันที่ใน settings.json — ถ้ายังไม่เคยรันวันนี้ให้รัน แล้วจดวันที่ไว้)
    {
        let today = today_local();
        let mut g = shared.lock().unwrap();
        let should_run = !g.startup_script.trim().is_empty() && g.startup_last_run != today;
        let cmd = g.startup_script.clone();
        if should_run {
            g.startup_last_run = today.clone();
        }
        drop(g);
        if should_run {
            let _ = sse(
                out,
                "note",
                json!({"t": format!("▶ รัน startup script (เทิร์นแรกของวัน): {cmd}")}),
            );
            let output = tools::execute(
                "run_command",
                &json!({ "command": cmd, "timeout_seconds": 120 }),
                &mut None,
            );
            let ok = output.starts_with("[run_command] รหัสจบ: 0");
            append_startup_log(&today, &cmd, &output, ok); // เก็บประวัติย้อนหลัง
            let mut g = shared.lock().unwrap();
            g.startup_note = Some(format!("[startup script: {cmd}]\nผลลัพธ์:\n{output}"));
            let _ = write_settings(&g.startup_script, &g.skills, &g.startup_last_run);
            g.rebuild_system();
        }
    }

    // สร้าง content ของผู้ใช้: ข้อความธรรมดา หรือ array multimodal (ข้อความ + รูปภาพ)
    let user_content = if images.is_empty() {
        json!(message)
    } else {
        let mut parts = vec![json!({"type": "text", "text": message})];
        for (mime, data) in &images {
            parts.push(json!({
                "type": "image_url",
                "image_url": { "url": format!("data:{mime};base64,{data}") }
            }));
        }
        json!(parts)
    };

    let mut g = shared.lock().unwrap();
    let cur_eff = g.current_eff();
    {
        let Shared { history, plan, .. } = &mut *g;
        history.push(json!({"role": "user", "content": user_content}));
        let mut sink = SseSink { out };
        crate::run_turn(agent, &cur_eff, history, plan, &mut sink);
    }
    crate::save_session(&g.history);
    drop(g);
    let _ = sse(out, "done", json!({"ok": true}));
    let _ = out.flush();
}

/// จัดการคำสั่งที่ขึ้นต้นด้วย / คืนข้อความที่จะตอบ (None = ไม่ใช่คำสั่ง)
fn try_command(message: &str, _eff: &config::Effective, shared: &Mutex<Shared>) -> Option<String> {
    if !message.starts_with('/') {
        return None;
    }
    let cmd = message.split_whitespace().next().unwrap_or("");
    let mut g = shared.lock().unwrap();
    let cur = g.current_eff();
    let text = match cmd {
        "/help" => commands_text(),
        "/model" => format!(
            "แบ็กเอนด์: {} | base_url: {} | model: {}",
            cur.backend.label(),
            if cur.base_url.is_empty() {
                "-"
            } else {
                &cur.base_url
            },
            if cur.model.is_empty() {
                "-"
            } else {
                &cur.model
            }
        ),
        "/plan" => match &g.plan {
            Some(p) => format!("แผนงานปัจจุบัน:\n{p}"),
            None => "ยังไม่มีแผน — บอกให้ CommandBlock วางแผนก่อนเริ่มงานได้".to_string(),
        },
        "/skills" => tools::list_skills_public(),
        "/reset" | "/clear" => {
            g.history.truncate(1);
            g.plan = None;
            "เริ่มบทสนทนาใหม่ (ล้างประวัติแล้ว)".to_string()
        }
        "/forget" => {
            g.history.truncate(1);
            g.plan = None;
            let _ = std::fs::remove_file(crate::SESSION_FILE);
            "ล้างความจำแล้ว (ลบไฟล์ buff_session.json และเริ่มใหม่)".to_string()
        }
        "/preview" => tools::reopen_preview(),
        _ => return None,
    };
    Some(text)
}

fn commands_text() -> String {
    [
        "คำสั่งในแชท:",
        "  /help     ดูคำสั่งนี้",
        "  /model    ดูแบ็กเอนด์/โมเดลที่ใช้อยู่",
        "  /plan     ดูแผนงานล่าสุดของ CommandBlock",
        "  /skills   ดูรายการทักษะเฉพาะทางที่โหลดได้",
        "  /preview  เปิดพรีวิวเว็บครั้งล่าสุดอีกครั้ง",
        "  /reset    ล้างประวัติการสนทนา เริ่มใหม่",
        "  /forget   ล้างความจำทั้งหมด (ลบ buff_session.json)",
        "",
        "ตัวอย่าง:",
        "  • \"โปรเจกต์นี้คืออะไร สรุปโครงสร้างให้หน่อย\"",
        "  • \"สร้างหน้าเว็บโปรไฟล์แล้วเปิดพรีวิวให้ดู\"",
        "  • \"ค้นหาว่า Rust เวอร์ชันล่าสุดคืออะไร\"",
        "  • \"แก้บั๊กในไฟล์ src/main.rs ที่ทำให้ crash\"",
    ]
    .join("\n")
}

/// รายการโมเดลทั้งหมดที่เลือกสลับได้: จาก config.json + Ollama ในเครื่อง (ถ้าเปิด)
fn build_models_list(agent: &ureq::Agent, shared: &Mutex<Shared>) -> Vec<Value> {
    let g = shared.lock().unwrap();
    build_models_list_inner(agent, &g)
}

fn build_models_list_inner(agent: &ureq::Agent, g: &Shared) -> Vec<Value> {
    let cur_model = g.model.clone();
    let cur_base = g.base_url.clone();
    let mut out: Vec<Value> = Vec::new();
    // 1) โมเดลจาก config.json
    for m in &g.cfg_models {
        let base = if m.base_url.is_empty() {
            g.eff.base_url.clone()
        } else {
            m.base_url.clone()
        };
        let active = m.name == cur_model && base == cur_base;
        if !out
            .iter()
            .any(|x| x["name"] == m.name && x["base_url"] == base)
        {
            out.push(json!({
                "name": m.name,
                "base_url": base,
                "source": "config",
                "active": active,
            }));
        }
    }
    // 2) โมเดล Ollama ในเครื่อง (ตรวจอัตโนมัติ)
    if let Some(list) = ollama_models(agent) {
        for (name, base) in list {
            if out
                .iter()
                .any(|x| x["name"] == name && x["base_url"] == base)
            {
                continue;
            }
            out.push(json!({
                "name": name,
                "base_url": base,
                "source": "ollama",
                "active": name == cur_model && base == cur_base,
            }));
        }
    }
    // 3) ถ้าไม่มีเลย ใส่ตัวปัจจุบัน
    if out.is_empty() && !cur_model.is_empty() {
        out.push(json!({
            "name": cur_model,
            "base_url": cur_base,
            "source": "current",
            "active": true,
        }));
    }
    out
}

/// ดึงรายชื่อโมเดลที่ติดตั้งใน Ollama (localhost)
fn ollama_models(agent: &ureq::Agent) -> Option<Vec<(String, String)>> {
    let url = format!(
        "{}/models",
        config::DEFAULT_OLLAMA_URL.trim_end_matches('/')
    );
    let resp = agent
        .get(&url)
        .timeout(std::time::Duration::from_secs(6))
        .call()
        .ok()?;
    let text = resp.into_string().ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    let mut out = Vec::new();
    if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
        for m in arr {
            if let Some(id) = m["id"].as_str() {
                if !id.is_empty() {
                    out.push((id.to_string(), config::DEFAULT_OLLAMA_URL.to_string()));
                }
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn skill_count() -> usize {
    tools::list_skills_public()
        .lines()
        .filter(|l| l.starts_with("  • "))
        .count()
}

// ---------- SSE ----------

struct SseSink<'a> {
    out: &'a mut BufWriter<TcpStream>,
}

impl TurnSink for SseSink<'_> {
    fn content(&mut self, delta: &str) {
        let _ = sse(self.out, "content", json!({"t": delta}));
    }
    fn think(&mut self, text: &str) {
        let _ = sse(self.out, "think", json!({"t": text}));
    }
    fn tools_begin(&mut self) {}
    fn tool(&mut self, name: &str, args: &Value) {
        let _ = sse(
            self.out,
            "tool",
            json!({"name": name, "args": args.to_string()}),
        );
    }
    fn note(&mut self, msg: &str) {
        let _ = sse(self.out, "note", json!({"t": msg}));
    }
    fn result(&mut self, _text: &str) {
        // หลังรันเครื่องมือไฟล์สำเร็จ → ส่งเหตุการณ์ "change" ให้ UI อัปเดตกล่องสรุป
        if let Some(e) = tools::take_last_change() {
            let _ = sse(
                self.out,
                "change",
                json!({"path": e.path, "status": e.status, "added": e.added, "deleted": e.deleted}),
            );
        }
    }
    fn end_line(&mut self) {}
}

fn sse(out: &mut BufWriter<TcpStream>, event: &str, data: Value) -> std::io::Result<()> {
    writeln!(out, "event: {event}")?;
    writeln!(out, "data: {data}")?;
    writeln!(out)?;
    out.flush()
}

// ---------- util ----------

fn respond(out: &mut BufWriter<TcpStream>, status: u16, ctype: &str, body: &[u8]) {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Error",
    };
    write!(
        out,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .ok();
    let _ = out.write_all(body);
    let _ = out.flush();
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

const FILE_SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "vendor",
    ".venv",
    "venv",
    "__pycache__",
    ".next",
    ".nuxt",
    ".freebuff",
    ".idea",
    ".vscode",
    ".cache",
    "preview",
    "out",
    "bin",
    "obj",
    ".pytest_cache",
    "coverage",
];

/// ไล่รายการไฟล์ในโปรเจกต์ (ข้ามโฟลเดอร์ใหญ่) — สำหรับแท็บ Files
fn list_project_files() -> Vec<String> {
    fn walk(dir: &std::path::Path, prefix: &str, out: &mut Vec<String>, depth: usize) {
        if depth > 6 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if FILE_SKIP_DIRS.contains(&name.as_str())
                || name.starts_with('.')
                || name.ends_with(".WebView2")
            {
                continue;
            }
            let Ok(ft) = e.file_type() else {
                continue;
            };
            let rel = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            if ft.is_dir() {
                walk(&e.path(), &rel, out, depth + 1);
            } else if ft.is_file() {
                out.push(rel);
            }
        }
    }
    let mut out = Vec::new();
    walk(std::path::Path::new("."), "", &mut out, 0);
    out.sort();
    out
}

/// percent-decode ง่ายๆ สำหรับ query param
fn percent_decode_gui(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let h = |c: u8| -> Option<u8> {
                match c {
                    b'0'..=b'9' => Some(c - b'0'),
                    b'a'..=b'f' => Some(c - b'a' + 10),
                    b'A'..=b'F' => Some(c - b'A' + 10),
                    _ => None,
                }
            };
            if let (Some(hh), Some(ll)) = (h(b[i + 1]), h(b[i + 2])) {
                out.push(hh * 16 + ll);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}
