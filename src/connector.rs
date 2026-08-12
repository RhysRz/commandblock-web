use serde_json::{json, Value};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

const SUPABASE_URL: &str = "https://qympivgklmstrnhfaywn.supabase.co";
const SUPABASE_PUBLISHABLE_KEY: &str = "sb_publishable_UJMuyL3QY8lMEWJKZi3zAQ_NFKZY8TH";

/// เปิด Connector แบบ console แยกจากแอป GUI เพื่อให้ Windows ส่ง stdin/stdout ได้ถูกต้อง
pub fn launch_sidecar() -> Result<(), String> {
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    let sidecar = current.parent().ok_or_else(|| "หาโฟลเดอร์โปรแกรมไม่พบ".to_string())?.join("commandblock-connector.exe");
    if !sidecar.is_file() {
        return Err("ไม่พบ commandblock-connector.exe — กรุณาอัปเดต CommandBlock ใหม่".to_string());
    }
    Command::new(sidecar).spawn().map(|_| ()).map_err(|e| format!("เปิด Desktop Connector ไม่สำเร็จ: {e}"))
}

pub fn safe_child(root: &Path, requested: &str) -> Result<PathBuf, String> {
    let relative = Path::new(requested);
    if requested.trim().is_empty() || relative.is_absolute() {
        return Err("พาธต้องอยู่ภายในโฟลเดอร์ Connector".to_string());
    }
    if relative.components().any(|part| matches!(part, Component::ParentDir | Component::Prefix(_) | Component::RootDir)) {
        return Err("ไม่อนุญาตให้เข้าถึงนอกโฟลเดอร์ Connector".to_string());
    }
    Ok(root.join(relative))
}

pub fn run(agent: ureq::Agent) -> Result<(), String> {
    println!("CommandBlock Desktop Connector");
    println!("ลงชื่อเข้าใช้บัญชีเดียวกับ CommandBlock Web (session นี้จะไม่บันทึกรหัสผ่าน)");
    let email = prompt("อีเมล: ")?;
    let password = prompt("รหัสผ่าน: ")?;
    let token = sign_in(&agent, &email, &password)?;
    let mut root = rfd::FileDialog::new().set_title("เลือกโฟลเดอร์สำหรับ CommandBlock Web").pick_folder()
        .ok_or_else(|| "ยกเลิกการเลือกโฟลเดอร์".to_string())?;
    let name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows PC".to_string());
    let device = create_device(&agent, &token, &name, root_name(&root))?;
    println!("✓ เชื่อมต่อ {} แล้ว — อนุญาตเฉพาะ {}", name, root.display());
    println!("เปิดหน้านี้ค้างไว้ แล้วกลับไปกด Files หรือ Terminal บน CommandBlock Web (Ctrl+C เพื่อหยุด)");

    loop {
        heartbeat(&agent, &token, &device)?;
        if let Some(command) = next_command(&agent, &token, &device)? {
            let id = command.get("id").and_then(Value::as_str).ok_or_else(|| "คำสั่ง Connector ไม่มี id".to_string())?;
            mark_running(&agent, &token, id)?;
            let action = command.get("action").and_then(Value::as_str).unwrap_or("");
            let payload = command.get("payload").unwrap_or(&Value::Null);
            let result = if action == "pick_folder" {
                match rfd::FileDialog::new().set_title("เปลี่ยนโฟลเดอร์สำหรับ CommandBlock Web").pick_folder() {
                    Some(next_root) => {
                        root = next_root;
                        update_device_root(&agent, &token, &device, root_name(&root))?;
                        Ok(json!({"ok": true, "path": root_name(&root), "root": root_name(&root), "files": count_files(&root)}))
                    }
                    None => Ok(json!({"ok": false, "cancelled": true})),
                }
            } else {
                execute(action, payload, &root)
            };
            match result {
                Ok(value) => finish(&agent, &token, id, "completed", Some(value), None)?,
                Err(error) => finish(&agent, &token, id, "failed", None, Some(error))?,
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn prompt(label: &str) -> Result<String, String> {
    print!("{label}");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut value = String::new();
    io::stdin().read_line(&mut value).map_err(|e| e.to_string())?;
    let value = value.trim().to_string();
    if value.is_empty() { Err("ห้ามเว้นว่าง".to_string()) } else { Ok(value) }
}

fn sign_in(agent: &ureq::Agent, email: &str, password: &str) -> Result<String, String> {
    let response: Value = agent.post(&format!("{SUPABASE_URL}/auth/v1/token?grant_type=password"))
        .set("apikey", SUPABASE_PUBLISHABLE_KEY).set("Content-Type", "application/json")
        .send_json(json!({"email": email, "password": password}))
        .map_err(|_| "เข้าสู่ระบบ Connector ไม่สำเร็จ".to_string())?.into_json().map_err(|_| "อ่าน session Connector ไม่สำเร็จ".to_string())?;
    response.get("access_token").and_then(Value::as_str).map(str::to_string)
        .ok_or_else(|| "บัญชีหรือรหัสผ่านไม่ถูกต้อง".to_string())
}

fn auth<'a>(request: ureq::Request, token: &'a str) -> ureq::Request {
    request.set("apikey", SUPABASE_PUBLISHABLE_KEY).set("Authorization", &format!("Bearer {token}")).set("Content-Type", "application/json")
}

fn create_device(agent: &ureq::Agent, token: &str, name: &str, root_name: String) -> Result<String, String> {
    let rows: Vec<Value> = auth(agent.post(&format!("{SUPABASE_URL}/rest/v1/connector_devices")), token)
        .set("Prefer", "return=representation")
        .send_json(json!({"name": name, "root_name": root_name}))
        .map_err(|_| "ลงทะเบียน Desktop Connector ไม่สำเร็จ".to_string())?.into_json().map_err(|_| "อ่านข้อมูล Desktop Connector ไม่สำเร็จ".to_string())?;
    rows.first().and_then(|row| row.get("id")).and_then(Value::as_str).map(str::to_string)
        .ok_or_else(|| "ไม่ได้ device id จาก Supabase".to_string())
}

fn heartbeat(agent: &ureq::Agent, token: &str, device: &str) -> Result<(), String> {
    auth(agent.patch(&format!("{SUPABASE_URL}/rest/v1/connector_devices?id=eq.{device}")), token)
        .send_json(json!({"name": std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows PC".to_string())})).map_err(|_| "ส่ง heartbeat ไม่สำเร็จ".to_string())?;
    Ok(())
}

fn update_device_root(agent: &ureq::Agent, token: &str, device: &str, root_name: String) -> Result<(), String> {
    auth(agent.patch(&format!("{SUPABASE_URL}/rest/v1/connector_devices?id=eq.{device}")), token)
        .send_json(json!({"root_name": root_name})).map_err(|_| "อัปเดตโฟลเดอร์ Connector ไม่สำเร็จ".to_string())?;
    Ok(())
}

fn next_command(agent: &ureq::Agent, token: &str, device: &str) -> Result<Option<Value>, String> {
    let rows: Vec<Value> = auth(agent.get(&format!("{SUPABASE_URL}/rest/v1/connector_commands?device_id=eq.{device}&status=eq.queued&order=created_at.asc&limit=1")), token)
        .call().map_err(|_| "ตรวจคำสั่ง Connector ไม่สำเร็จ".to_string())?.into_json().map_err(|_| "อ่านคำสั่ง Connector ไม่สำเร็จ".to_string())?;
    Ok(rows.into_iter().next())
}

fn mark_running(agent: &ureq::Agent, token: &str, command: &str) -> Result<(), String> {
    auth(agent.patch(&format!("{SUPABASE_URL}/rest/v1/connector_commands?id=eq.{command}&status=eq.queued")), token)
        .send_json(json!({"status":"running"})).map_err(|_| "รับคำสั่ง Connector ไม่สำเร็จ".to_string())?;
    Ok(())
}

fn finish(agent: &ureq::Agent, token: &str, command: &str, status: &str, result: Option<Value>, error: Option<String>) -> Result<(), String> {
    auth(agent.patch(&format!("{SUPABASE_URL}/rest/v1/connector_commands?id=eq.{command}")), token)
        .send_json(json!({"status":status, "result":result, "error":error})).map_err(|_| "ส่งผล Connector ไม่สำเร็จ".to_string())?;
    Ok(())
}

fn execute(action: &str, payload: &Value, root: &Path) -> Result<Value, String> {
    match action {
        "files" => Ok(json!({"files": list_files(root)})),
        "read" => {
            let requested = payload.get("path").and_then(Value::as_str).unwrap_or("");
            let path = safe_child(root, requested)?;
            let content = std::fs::read_to_string(path).map_err(|_| "อ่านไฟล์ไม่ได้".to_string())?;
            Ok(json!({"content": content.chars().take(150_000).collect::<String>()}))
        }
        "changes" => Ok(json!({"changes": []})),
        "queue" => Ok(json!({"activity": ["Desktop Connector ออนไลน์"]})),
        "preview" => Ok(json!({"preview_url": ""})),
        "exec" => {
            let command = payload.get("command").and_then(Value::as_str).unwrap_or("").trim();
            if command.is_empty() { return Ok(json!({"output":"(ว่าง)"})); }
            println!("\n⚠️ เว็บขอรันคำสั่งใน {}:\n  {command}\nพิมพ์ yes เพื่ออนุมัติ:", root.display());
            if prompt("")? != "yes" { return Err("ผู้ใช้ไม่อนุมัติคำสั่งบน Desktop Connector".to_string()); }
            let output = crate::tools::execute("run_command", &json!({"command": command, "cwd": root, "timeout_seconds": 60}), &mut None);
            Ok(json!({"output": output}))
        }
        _ => Err("ไม่รองรับคำสั่ง Connector นี้".to_string()),
    }
}

fn root_name(root: &Path) -> String { root.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "โฟลเดอร์โปรเจกต์".to_string()) }
fn count_files(root: &Path) -> usize { list_files(root).len() }
fn list_files(root: &Path) -> Vec<String> {
    fn visit(root: &Path, current: &Path, out: &mut Vec<String>) {
        if out.len() >= 300 { return; }
        let Ok(entries) = std::fs::read_dir(current) else { return; };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() { if !matches!(name.as_str(), ".git" | "target" | "node_modules") { visit(root, &path, out); } }
            else if let Ok(relative) = path.strip_prefix(root) { out.push(relative.to_string_lossy().replace('\\', "/")); }
            if out.len() >= 300 { break; }
        }
    }
    let mut out = Vec::new(); visit(root, root, &mut out); out.sort(); out
}

#[cfg(test)]
mod tests {
    use super::safe_child;
    use std::path::Path;

    #[test]
    fn safe_child_rejects_paths_outside_the_selected_root() {
        let root = Path::new("C:\\workspace");
        assert!(safe_child(root, "src/main.rs").is_ok());
        assert!(safe_child(root, "..\\secret.txt").is_err());
        assert!(safe_child(root, "C:\\Windows\\win.ini").is_err());
    }
}
