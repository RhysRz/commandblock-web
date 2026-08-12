//! Commandblock — ผู้ช่วยพัฒนาโค้ด AI แบบตัวแทน (agentic coding assistant CLI)
//!
//! รัน: Commandblock.exe  แล้วพิมพ์งานเป็นภาษาไทย/อังกฤษ
//! หรือ: Commandblock.exe "คำถาม/งาน"  สำหรับรันครั้งเดียว (one-shot)

// ไม่เปิดหน้าต่าง console ตอนดับเบิลคลิก (เป็นแอปเดสก์ท็อป) —
// แต่โหมด CLI ยังพิมพ์/อ่านจาก cmd ได้ปกติ (สืบทอด console ต้นทาง)
#![cfg_attr(windows, windows_subsystem = "windows")]

mod config;
mod gui;
mod llm;

use commandblock::connector;
pub use commandblock::tools;

use serde_json::{json, Value};
use std::fs;
use std::io::{self, Write};

/// ตัวรับผลลัพธ์ระหว่างเทิร์นของ Commandblock — CLI พิมพ์จอ, GUI ส่ง SSE ไปเบราว์เซอร์
pub trait TurnSink {
    /// เนื้อหาคำตอบทีละคำ (streaming)
    fn content(&mut self, delta: &str);
    /// ความคิดของโมเดล (thinking/reasoning) — แสดงแบบเรียลไทม์
    fn think(&mut self, text: &str);
    /// เริ่มเรียกเครื่องมือชุดหนึ่ง
    fn tools_begin(&mut self);
    /// เรียกเครื่องมือหนึ่งตัว
    fn tool(&mut self, name: &str, args: &Value);
    /// ข้อความแจ้งเตือน/สถานะ (เช่น ตรวจจับลูป, ถึงขีดจำกัด)
    fn note(&mut self, msg: &str);
    /// ผลลัพธ์เครื่องมือ (CLI แสดงขนาด, GUI ข้าม)
    fn result(&mut self, text: &str);
    /// จบเทิร์นของ LLM (CLI: ขึ้นบรรทัดใหม่ถ้ามีเนื้อหา)
    fn end_line(&mut self);
}

/// sink สำหรับ CLI (พิมพ์จอเหมือนเดิม)
struct CliSink {
    header: bool,
}

impl TurnSink for CliSink {
    fn content(&mut self, delta: &str) {
        if !self.header {
            print!("CommandBlock: ");
            self.header = true;
        }
        print!("{delta}");
        let _ = io::stdout().flush();
    }
    fn think(&mut self, _text: &str) {
        // CLI: ไม่แสดงความคิดของโมเดล (เก็บจอสะอาด) — GUI แสดงเป็นรายละเอียด
    }
    fn tools_begin(&mut self) {
        println!("[CommandBlock กำลังใช้เครื่องมือ…]");
    }
    fn tool(&mut self, name: &str, args: &Value) {
        println!("  • {name} {}", summarize_args(args));
    }
    fn note(&mut self, msg: &str) {
        println!("\n{msg}");
    }
    fn result(&mut self, text: &str) {
        if text.len() > 2000 {
            println!("    (ผลลัพธ์ยาว {} ตัวอักษร)", text.len());
        }
    }
    fn end_line(&mut self) {
        if self.header {
            println!();
            self.header = false;
        }
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_ROUNDS: usize = 15;
const MAX_TOOL_RESULT_IN_HISTORY: usize = 6000;
const MAX_HISTORY: usize = 60; // จำนวนข้อความสูงสุดที่ส่งให้ AI ต่อรอบ (ตัดเก่าเมื่อเกิน)
const SESSION_FILE: &str = "buff_session.json"; // ความจำข้ามเซสชัน
const SESSION_KEEP: usize = 20; // จำนวนข้อความล่าสุดที่บันทึกลง session

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 {
        match args[1].as_str() {
            "-h" | "--help" | "help" => {
                print_main_help();
                return;
            }
            "-v" | "--version" => {
                println!("CommandBlock v{VERSION} — ผู้ช่วยพัฒนาโค้ด AI");
                return;
            }
            _ => {}
        }
    }

    if args.iter().any(|arg| arg == "--connector") {
        if let Err(error) = connector::launch_sidecar() {
            eprintln!("Desktop Connector: {error}");
            std::process::exit(1);
        }
        return;
    }

    let cli_mode = args.iter().any(|a| a == "--cli");
    let one_shot = args.len() >= 2 && !cli_mode && !args[1].starts_with('-') && args[1] != "--gui";

    let cfg = config::load();

    // ตรวจ Ollama (ถ้า backend เป็น auto และไม่มี key จะลองใช้ Ollama)
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(600))
        .build();
    // ตรวจหา Ollama ที่ localhost:11434 เสมอ (ไม่เกี่ยวกับ base_url ของ API)
    let ollama_url = config::DEFAULT_OLLAMA_URL.to_string();
    let ollama_up = cfg.backend != config::Backend::Offline && llm::ollama_reachable(&agent, &ollama_url);
    let picked_model = if ollama_up {
        llm::pick_model(&agent, &ollama_url)
    } else {
        None
    };

    let eff = config::effective(&cfg, ollama_up, picked_model.clone());

    // โหมด GUI (ค่าเริ่มต้น): Commandblock.exe → เปิดแชทหน้าตาเหมือนแชท AI ในเบราว์เซอร์
    if !cli_mode && !one_shot {
        gui::serve(agent, &eff, cfg.models.clone());
    }

    let key_note = if eff.backend == config::Backend::OpenAI {
        format!("key: {}", mask_key(&eff.api_key))
    } else {
        String::new()
    };

    println!("CommandBlock v{VERSION} — ผู้ช่วยพัฒนาโค้ด AI");
    println!(
        "แบ็กเอนด์: {} | model: {} {}",
        eff.backend.label(),
        if eff.model.is_empty() { "-" } else { &eff.model },
        key_note
    );
    if eff.backend == config::Backend::Offline {
        if ollama_up && picked_model.is_none() {
            println!(
                "⚠️ Ollama เปิดอยู่แต่ยังไม่มีโมเดล — รัน: ollama pull qwen2.5-coder:7b แล้วเปิด CommandBlock ใหม่"
            );
        } else {
            println!(
                "⚠️ ยังไม่มี AI ต่อ — ตั้งค่าโดย: (1) ใส่ API key ใน config.json หรือตั้งตัวแปร BUFF_API_KEY (2) หรือเปิด Ollama ที่ http://localhost:11434 (3) ดูวิธีใน README.md"
            );
        }
    }
    println!("พิมพ์ /help เพื่อดูคำสั่ง หรือพิมพ์งานของคุณ แล้วกด Enter\n");

    let system = system_prompt();
    let mut history: Vec<Value> = vec![json!({"role": "system", "content": system})];

    // ความจำข้ามเซสชัน: โหลดบทสนทนาก่อนหน้า (ถ้ามี)
    let loaded = load_session();
    let mut session_note = String::new();
    if !loaded.is_empty() {
        session_note = format!(
            "🧠 จำบทสนทนาก่อนหน้าได้ {} ข้อความ (พิมพ์ /forget เพื่อล้างความจำ)\n",
            loaded.len()
        );
        history.extend(loaded);
    }
    if !session_note.is_empty() {
        println!("{session_note}");
    }
    let mut plan: Option<String> = None;

    // one-shot mode: Commandblock.exe "คำถาม"
    if one_shot {
        let q = args[1..].join(" ");
        println!("คุณ: {q}\n");
        history.push(json!({"role": "user", "content": q}));
        let mut sink = CliSink { header: false };
        run_turn(&agent, &eff, &mut history, &mut plan, &mut sink);
        save_session(&history);
        // ถ้า Commandblock เปิดพรีวิวเว็บไว้ ให้ค้างโปรแกรมเพื่อให้เซิร์ฟเวอร์รันต่อ (กด Enter/Ctrl+C เพื่อปิด)
        if let Some(u) = tools::last_preview_url() {
            println!("\n🖥️ พรีวิวเปิดอยู่ที่ {u}\n   (กด Enter หรือ Ctrl+C เพื่อปิดพรีวิว)");
            let mut _line = String::new();
            let _ = io::stdin().read_line(&mut _line);
        }
        return;
    }

    // REPL
    loop {
        print!("คุณ> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {}
            Err(_) => break,
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if input.starts_with('/') {
            match input {
                "/help" => print_help(),
                "/exit" | "/quit" => break,
                "/reset" => {
                    history.truncate(1);
                    plan = None;
                    println!("เริ่มบทสนทนาใหม่ (ล้างประวัติแล้ว)");
                }
                "/forget" => {
                    history.truncate(1);
                    plan = None;
                    let _ = fs::remove_file(SESSION_FILE);
                    println!("ล้างความจำแล้ว (ลบไฟล์ {SESSION_FILE} และเริ่มใหม่)");
                }
                "/plan" => match &plan {
                    Some(p) => println!("แผนงานปัจจุบัน:\n{p}"),
                    None => println!("ยังไม่มีแผน — บอกให้ CommandBlock วางแผนก่อนเริ่มงานได้"),
                },
                "/skills" => println!("{}", tools::list_skills_public()),
                "/preview" => println!("{}", tools::reopen_preview()),
                "/model" => println!(
                    "แบ็กเอนด์: {} | base_url: {} | model: {}",
                    eff.backend.label(),
                    if eff.base_url.is_empty() { "-" } else { &eff.base_url },
                    if eff.model.is_empty() { "-" } else { &eff.model }
                ),
                _ => println!("ไม่รู้จักคำสั่ง '{input}' — พิมพ์ /help"),
            }
            continue;
        }

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") || input == "ออก" {
            break;
        }

        history.push(json!({"role": "user", "content": input}));
        let mut sink = CliSink { header: false };
        run_turn(&agent, &eff, &mut history, &mut plan, &mut sink);
        save_session(&history);
    }

    save_session(&history);
    println!("ลาก่อนครับ 👋");
}

/// โหลดความจำจากไฟล์ session (เฉพาะข้อความที่เคยบันทึกไว้)
pub fn load_session() -> Vec<Value> {
    let Ok(text) = fs::read_to_string(SESSION_FILE) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    match v {
        Value::Array(arr) => arr.into_iter().filter(|m| m.get("role").is_some()).collect(),
        _ => Vec::new(),
    }
}

/// บันทึกความจำลงไฟล์ session (เก็บเฉพาะ 20 ข้อความล่าสุด)
pub fn save_session(history: &[Value]) {
    let msgs: Vec<Value> = history.iter().skip(1).cloned().collect();
    let mut start = msgs.len().saturating_sub(SESSION_KEEP);
    // ถ้าข้อความแรกเป็น tool message ต้องเอา assistant (ที่เรียกเครื่องมือนั้น) ก่อนหน้าด้วย
    if start > 0 && msgs[start]["role"] == "tool" {
        start -= 1;
    }
    let kept: Vec<Value> = msgs[start..].to_vec();
    if kept.is_empty() {
        let _ = fs::remove_file(SESSION_FILE);
        return;
    }
    if let Ok(text) = serde_json::to_string_pretty(&kept) {
        let _ = fs::write(SESSION_FILE, text);
    }
}

/// ตัดประวัติเก่าเมื่อยาวเกิน (กัน context บวม) — เก็บ system + ข้อความล่าสุดเสมอ
fn trim_history(h: &mut Vec<Value>) {
    if h.len() <= MAX_HISTORY {
        return;
    }
    let mut start = h.len() - (MAX_HISTORY - 1);
    if start < 1 {
        start = 1;
    }
    // ถ้าตำแหน่งเริ่มเป็น tool message ให้ถอยไปเริ่มที่ assistant ที่เรียกมัน
    if h[start]["role"] == "tool" && start > 1 {
        start -= 1;
    }
    let mut kept = vec![h[0].clone()];
    kept.extend_from_slice(&h[start..]);
    *h = kept;
}

/// วน agentic loop: ให้ LLM คิด → ใช้เครื่องมือ → ส่งผลกลับ → จนกว่าไม่มี tool call
/// ผลลัพธ์ทั้งหมดส่งผ่าน `sink` (CLI พิมพ์จอ / GUI ส่ง SSE)
pub fn run_turn(
    agent: &ureq::Agent,
    eff: &config::Effective,
    history: &mut Vec<Value>,
    plan: &mut Option<String>,
    sink: &mut dyn TurnSink,
) {
    if matches!(eff.backend, config::Backend::Offline) {
        sink.note(
            "CommandBlock: ขอโทษครับ ตอนนี้ยังไม่มี AI ต่อ (โหมด offline)\n   วิธีเปิดใช้งาน:\n     1. ใส่ API key: แก้ไฟล์ config.json (api_key) หรือตั้งตัวแปร BUFF_API_KEY\n     2. หรือเปิด Ollama (https://ollama.com) ที่เครื่อง แล้วรัน: ollama pull qwen2.5-coder:7b\n   แล้วรัน CommandBlock ใหม่ (ดูรายละเอียดใน README.md)",
        );
        sink.end_line();
        return;
    }

    let mut rounds = 0;
    let mut tools_used = true;
    let mut seen_calls: Vec<(String, Value)> = Vec::new();
    let mut name_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut last_tool_result: Option<String> = None;

    while tools_used {
        rounds += 1;
        if rounds > MAX_ROUNDS {
            sink.note(&format!("(ถึงขีดจำกัด {MAX_ROUNDS} รอบ — ขอสรุปผลล่าสุด)"));
            final_summary(agent, eff, history, last_tool_result.as_deref(), sink);
            break;
        }

        let use_tools = !matches!(eff.backend, config::Backend::Offline);
        let schemas = if use_tools { tools::tool_schemas() } else { Vec::new() };

        let sink_rc = std::rc::Rc::new(std::cell::RefCell::new(&mut *sink));
        let mut content_out = { let s = std::rc::Rc::clone(&sink_rc); move |c: &str| s.borrow_mut().content(c) };
        let mut think_out = { let s = std::rc::Rc::clone(&sink_rc); move |t: &str| s.borrow_mut().think(t) };
        let resp = match llm::chat_stream(agent, eff, history, &schemas, &mut content_out, &mut think_out) {
            Ok(r) => r,
            Err(e) => {
                // ถ้า error เกี่ยวกับ tools (บาง endpoint ไม่รองรับ) ลองใหม่แบบไม่มี tools
                if e.to_lowercase().contains("tools") {
                    let sink_rc2 = std::rc::Rc::new(std::cell::RefCell::new(&mut *sink));
                    let mut content_out2 = { let s = std::rc::Rc::clone(&sink_rc2); move |c: &str| s.borrow_mut().content(c) };
                    let mut think_out2 = { let s = std::rc::Rc::clone(&sink_rc2); move |t: &str| s.borrow_mut().think(t) };
                    match llm::chat_stream(agent, eff, history, &[], &mut content_out2, &mut think_out2) {
                        Ok(r) => r,
                        Err(e2) => {
                            sink.note(&e2);
                            sink.end_line();
                            return;
                        }
                    }
                } else {
                    sink.note(&e);
                    sink.end_line();
                    return;
                }
            }
        };
        sink.end_line();

        // เก็บข้อความ assistant (ต้องมี tool_calls เดิมถ้ามี)
        let mut msg = json!({"role": "assistant", "content": resp.content.clone()});
        if !resp.tool_calls.is_empty() {
            msg["tool_calls"] = json!(
                resp.tool_calls
                    .iter()
                    .map(|t| {
                        let mut tc = json!({
                            "id": t.id,
                            "type": "function",
                            "function": {
                                "name": t.name,
                                "arguments": serde_json::to_string(&t.arguments).unwrap_or_else(|_| "{}".into())
                            }
                        });
                        // Gemini ต้องการ thought_signature ส่งกลับไปด้วย ไม่งั้น HTTP 400
                        if let Some(extra) = &t.extra {
                            tc["extra_content"] = extra.clone();
                        }
                        tc
                    })
                    .collect::<Vec<_>>()
            );
        }
        history.push(msg);

        // บางโมเดลเขียน tool call เป็น JSON ในข้อความ — จับมาใช้เป็นทางเลือก
        let mut calls = resp.tool_calls;
        if calls.is_empty() && !resp.content.trim().is_empty() {
            if let Some(parsed) = llm::extract_content_tool_calls(&resp.content) {
                calls = parsed;
            }
        }

        if calls.is_empty() {
            tools_used = false;
            // เนื้อหาถูกพิมพ์ระหว่าง streaming แล้ว — ถ้าไม่มีเนื้อหาเลย ใช้ผลลัพธ์เครื่องมือล่าสุดแทน
            if resp.content.trim().is_empty() {
                if let Some(f) = last_tool_result.as_deref() {
                    sink.note(&format!("[CommandBlock] AI ไม่ได้ตอบสรุป แต่ผลลัพธ์ล่าสุดจากเครื่องมือ:\n{f}"));
                    sink.end_line();
                }
            }
            if resp.finish == "length" {
                sink.note("(ข้อความถูกตัดเพราะยาวเกิน — พิมพ์ 'ต่อ' เพื่อให้ตอบต่อ)");
            }
        } else {
            // ตรวจจับลูป: เรียกเครื่องมือเดิมซ้ำๆ หรือเรียกตัวเดิมบ่อยเกินไป → หยุดและขอสรุป
            let mut looped = false;
            for tc in &calls {
                let key = (tc.name.clone(), tc.arguments.clone());
                if seen_calls.iter().any(|(n, a)| *n == key.0 && *a == key.1) {
                    looped = true;
                    break;
                }
                let count = name_counts.entry(tc.name.clone()).or_insert(0);
                *count += 1;
                if *count > 4 {
                    looped = true;
                    break;
                }
            }
            if looped {
                sink.note("[CommandBlock] AI วนลูปเรียกเครื่องมือเดิมบ่อยเกินไป — ขอสรุปผลล่าสุดแทน");
                final_summary(agent, eff, history, last_tool_result.as_deref(), sink);
                sink.end_line();
                return;
            }

            sink.tools_begin();
            for tc in &calls {
                sink.tool(&tc.name, &tc.arguments);
                seen_calls.push((tc.name.clone(), tc.arguments.clone()));
                let result = tools::execute(&tc.name, &tc.arguments, plan);
                sink.result(&result);
                // ตัดผลลัพธ์ก่อนเก็บเข้า history เพื่อกัน context บวม (CPU inference ช้า)
                let clipped = clip_result(&result);
                // เก็บผลลัพธ์ที่มีข้อมูลจริงไว้ใช้เป็น fallback (ไม่ใช่ update_plan)
                if matches!(
                    tc.name.as_str(),
                    "read_file" | "list_directory" | "code_search" | "run_command" | "web_search" | "read_url"
                        | "open_preview" | "list_skills" | "load_skill"
                ) {
                    last_tool_result = Some(clipped.clone());
                }
                history.push(json!({
                    "role": "tool",
                    "tool_call_id": tc.id,
                    "content": clipped
                }));
            }
            trim_history(history);
        }
    }
}

fn final_summary(
    agent: &ureq::Agent,
    eff: &config::Effective,
    history: &[Value],
    fallback: Option<&str>,
    sink: &mut dyn TurnSink,
) {
    let sink_rc = std::rc::Rc::new(std::cell::RefCell::new(&mut *sink));
    let mut content_out = { let s = std::rc::Rc::clone(&sink_rc); move |c: &str| s.borrow_mut().content(c) };
    let mut think_out = { let s = std::rc::Rc::clone(&sink_rc); move |t: &str| s.borrow_mut().think(t) };
    match llm::chat_stream(agent, eff, history, &[], &mut content_out, &mut think_out) {
        Ok(r) => {
            sink.end_line();
            if r.content.trim().is_empty() {
                if let Some(f) = fallback {
                    sink.note(&format!("[CommandBlock] AI สรุปไม่ได้ แต่ผลลัพธ์ล่าสุดจากเครื่องมือ:\n{f}"));
                    sink.end_line();
                }
            }
        }
        Err(e) => sink.note(&e),
    }
}

fn clip_result(s: &str) -> String {
    if s.len() <= MAX_TOOL_RESULT_IN_HISTORY {
        return s.to_string();
    }
    let mut start = s.len() - MAX_TOOL_RESULT_IN_HISTORY;
    while !s.is_char_boundary(start) {
        start += 1;
    }
    format!("…(ผลลัพธ์ถูกตัดเหลือ {} ตัวอักษร)\n{}", MAX_TOOL_RESULT_IN_HISTORY, &s[start..])
}

fn summarize_args(args: &Value) -> String {
    let s = serde_json::to_string(args).unwrap_or_default();
    let mut t = s.trim().to_string();
    if t.len() > 120 {
        t = format!("{}…", &t[..t.char_indices().nth(120).map(|(i, _)| i).unwrap_or(t.len())]);
    }
    t
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "***".to_string();
    }
    let head: String = key.chars().take(4).collect();
    let tail: String = key.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{head}…{tail}")
}

pub fn system_prompt() -> String {
    r#"คุณคือ "CommandBlock" ผู้ช่วยพัฒนาโค้ดอัจฉริยะ (AI coding agent) ที่ทำงานผ่านเครื่องมือในเทอร์มินัล คล้ายกับ Codebuff/Claude Code — ผู้ใช้จะขอให้คุณทำงานในโปรเจกต์โค้ดของเขา

กฎการทำงาน:
1. ตอบเป็นภาษาไทยเสมอ (โค้ด ชื่อฟังก์ชัน พาธ คำสั่ง และศัพท์เทคนิคคงเป็นภาษาอังกฤษตามธรรมเนียม)
2. ใช้เครื่องมือเมื่อจำเป็นเท่านั้น: อ่านไฟล์ก่อนแก้ไขเสมอ, ค้นหาโค้ด (code_search) ก่อนเขียนโค้ดใหม่ที่อาจซ้ำ, ใช้ run_command เพื่อตรวจสอบ/ทดสอบ
3. งานหลายขั้นตอน: เริ่มด้วยการวางแผนสั้นๆ (ใช้เครื่องมือ update_plan) แล้วลงมือทำทีละขั้น พร้อมรายงานความคืบหน้าเป็นระยะ — แต่สำหรับคำถามง่ายๆ (ถามความหมาย, ถามคำตอบสั้นๆ) ให้ตอบทันทีโดยไม่ต้องใช้เครื่องมือใดๆ
4. หลังแก้โค้ด ให้รัน typecheck/test/ build ที่เกี่ยวข้องเพื่อยืนยันว่าทำงาน (เช่น npm test, cargo build, tsc --noEmit)
5. ห้ามแก้ไขไฟล์โดยไม่รู้เนื้อหาเดิม — อ่านก่อนเสมอ
6. ระวังคำสั่งอันตราย (ลบไฟล์, git push/reset, แตะฐานข้อมูลหรือ production) — ถ้าไม่แน่ใจ ให้ถามผู้ใช้ก่อน
7. เครื่องมือเว็บ: ใช้ web_search เพื่อค้นหาข้อมูลปัจจุบัน/ข่าว/เอกสารล่าสุด (เช่น ถามเรื่องเวอร์ชันล่าสุด, วิธีติดตั้ง, ข่าว, เรื่องนอกโปรเจกต์) แล้วใช้ read_url เพื่ออ่านรายละเอียดจากลิงก์ที่เจอ — อย่าตอบจากความรู้เก่าถ้ามีข้อมูลออนไลน์ใหม่กว่า
8. ความจำ: ผู้ใช้เรียกใช้คุณหลายครั้ง — ประวัติบทสนทนาก่อนหน้า (ระหว่างเซสชัน) ถูกส่งให้ในข้อความแล้ว ใช้ต่อจากบริบทนั้นได้ เช่น ผู้ใช้ถามต่อจากงานเมื่อวาน อย่าทำเป็นลืมงานที่เคยทำไป
9. พรีวิวเว็บ: เมื่อผู้ใช้ขอ 'ดูพรีวิว/แสดงหน้าเว็บ' หรือเมื่อคุณสร้างเว็บแอป/หน้า HTML ให้ใช้ open_preview เพื่อเปิดให้ผู้ใช้เห็นภาพจริงในเบราว์เซอร์ทันที
10. ทักษะ (skills): มีทักษะเฉพาะทางให้โหลด (เหมือนผู้ช่วย AI ระดับมืออาชีพ) — ก่อนทำงานเฉพาะทางให้ใช้ list_skills ดูรายการ แล้ว load_skill อ่านคำแนะนำ เช่น ตรวจสอบความเข้าถึง (accessibility), ออกแบบ API (api-design-principles)
11. เมื่อต้องใช้เครื่องมือ: ถ้าระบบไม่ให้รูปแบบ tool_calls มา ให้ตอบเป็น JSON เดี่ยวเท่านั้นในรูปแบบ {"name": "ชื่อเครื่องมือ", "arguments": {"พารามิเตอร์": ค่า}} เช่น {"name": "list_directory", "arguments": {"path": "."}} — ต้องมี name และ arguments ครบถ้วน ห้ามมีข้อความอื่นนอกจาก JSON และอย่าใช้ key "parameters"
12. หลังเสร็จงาน สรุปสั้นๆ: ทำอะไรไป, ผลลัพธ์, ไฟล์ที่เกี่ยวข้อง และวิธีทดสอบ
13. เมื่อได้ข้อมูลจากเครื่องมือครบแล้ว ให้ตอบสรุปทันที — อย่าเรียกเครื่องมือเดิมซ้ำถ้าไม่จำเป็น (เช่น อย่า list_directory หรือ read_file ซ้ำๆ)
14. ถ้าข้อมูลไม่พอ (เช่น ยังไม่รู้ว่าโปรเจกต์นี้คืออะไร) ให้สำรวจโฟลเดอร์ด้วย list_directory/read_file ก่อน แล้วค่อยทำงาน"#.to_string()
}

fn print_main_help() {
    println!(
        "CommandBlock v{VERSION} — ผู้ช่วยพัฒนาโค้ด AI\n\
         \n\
         วิธีใช้:\n\
         \x20 buff                          เปิด GUI แชท (ค่าเริ่มต้น)\n\
         \x20 buff --cli                    เปิดแชทแบบเทอร์มินัล\n\
         \x20 buff --connector              เชื่อม CommandBlock Web กับเครื่องนี้\n\
         \x20 buff \"ทำงานให้ฉัน...\"        รันงานครั้งเดียวแล้วจบ (one-shot)\n\
         \x20 buff --help                   ดูวิธีใช้นี้\n\
         \x20 buff --version                ดูเวอร์ชัน\n\
         \n\
         การตั้งค่า: ดู README.md หรือไฟล์ config.json ที่สร้างอัตโนมัติ\n"
    );
}

fn print_help() {
    println!(
        "คำสั่งในแชท:\n\
         \x20 /help     ดูคำสั่งนี้\n\
         \x20 /model    ดูแบ็กเอนด์/model ที่ใช้อยู่\n\
         \x20 /plan     ดูแผนงานล่าสุดของ CommandBlock\n\
         \x20 /skills   ดูรายการทักษะเฉพาะทางที่โหลดได้\n\
         \x20 /preview  เปิดพรีวิวเว็บครั้งล่าสุดอีกครั้ง\n\
         \x20 /reset    ล้างประวัติการสนทนา เริ่มใหม่\n\
         \x20 /forget   ล้างความจำทั้งหมด (ลบ buff_session.json)\n\
         \x20 /exit     ออกจากโปรแกรม (หรือพิมพ์ exit / ออก)\n\
         \n\
         ตัวอย่างการใช้งาน:\n\
         \x20 • \"โปรเจกต์นี้คืออะไร สรุปโครงสร้างให้หน่อย\"\n\
         \x20 • \"หาโค้ดที่จัดการ user login แล้วอธิบายว่ามันทำงานยังไง\"\n\
         \x20 • \"แก้บั๊กในไฟล์ src/main.rs ที่ทำให้ crash\"\n\
         \x20 • \"สร้างหน้าเว็บ HTML สวัสดี แล้วเปิดพรีวิวให้ดูหน่อย\"\n\
         \x20 • \"ค้นหาว่า Rust เวอร์ชันล่าสุดคืออะไร แล้วสรุปให้หน่อย\"\n"
    );
}
