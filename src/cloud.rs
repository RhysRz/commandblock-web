//! ซิงก์ประวัติแชทกับ Supabase (ตาราง conversations + messages เดียวกับ CommandBlock Web)
//!
//! - `push`: หลังจบแต่ละเทิร์น — อัปเดต conversation ของบัญชี (ข้อความ user/assistant)
//! - `pull`: ตอนล็อกอิน/เปิดแอป — ดึง conversation ล่าสุดของบัญชีกลับมา (ใช้ข้ามเครื่องได้)
//!
//! ข้อความ tool/system ไม่ซิงก์ (เก็บเฉพาะประวัติแชทที่อ่านรู้เรื่อง) — ฝั่งเครื่องมี
//! `.freebuff/sessions/<user_id>.json` เป็นแคชเต็มสำหรับ resume แบบละเอียด

use crate::auth;
use serde_json::{json, Value};

const SUPABASE_URL: &str = "https://qympivgklmstrnhfaywn.supabase.co";
const SUPABASE_PUBLISHABLE_KEY: &str = "sb_publishable_UJMuyL3QY8lMEWJKZi3zAQ_NFKZY8TH";
/// meta file เก็บ conversation_id ของบัญชี (ต่อ user)
const META_DIR: &str = ".freebuff/sessions";

fn meta_path(user_id: &str) -> String {
    format!("{META_DIR}/{user_id}.meta.json")
}

fn save_conv_id(user_id: &str, conv_id: &str) {
    let _ = std::fs::create_dir_all(META_DIR);
    let _ = std::fs::write(
        meta_path(user_id),
        json!({ "conversation_id": conv_id }).to_string(),
    );
}

/// agent ที่มี timeout สั้นสำหรับซิงก์ (ไม่ให้ค้างตอนเน็ตช้า)
fn sync_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(12))
        .build()
}

fn authed(mut req: ureq::Request, token: &str) -> ureq::Request {
    req = req
        .set("apikey", SUPABASE_PUBLISHABLE_KEY)
        .set("Authorization", &format!("Bearer {token}"));
    req
}

/// ดึงเฉพาะข้อความ user/assistant (text) จาก history ไปซิงก์
fn text_messages(history: &[Value]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for m in history {
        let role = m.get("role").and_then(Value::as_str).unwrap_or("");
        if role != "user" && role != "assistant" {
            continue;
        }
        let content = match m.get("content") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Array(parts)) => {
                let mut text = String::new();
                for p in parts {
                    if let Some(t) = p.get("text").and_then(Value::as_str) {
                        text.push_str(t);
                    }
                }
                text
            }
            _ => continue,
        };
        if content.trim().is_empty() {
            continue;
        }
        out.push((role.to_string(), content));
    }
    out
}

/// ใช้ conversation ล่าสุดของบัญชีเป็นแหล่งจริง ไม่ยึด meta ของเครื่องหนึ่งเครื่องใด
fn latest_conversation(a: &ureq::Agent, pair: &auth::TokenPair) -> Result<Option<String>, String> {
    let url = format!(
        "{SUPABASE_URL}/rest/v1/conversations?select=id&user_id=eq.{}&order=updated_at.desc&limit=1",
        pair.user_id
    );
    let rows: Vec<Value> = authed(a.get(&url), &pair.access_token)
        .call()
        .map_err(|_| "โหลดรายการสนทนาคลาวด์ไม่สำเร็จ".to_string())?
        .into_json()
        .map_err(|_| "อ่านรายการสนทนาคลาวด์ไม่สำเร็จ".to_string())?;
    Ok(rows
        .first()
        .and_then(|row| row.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string))
}

fn cloud_messages(
    a: &ureq::Agent,
    pair: &auth::TokenPair,
    conv_id: &str,
) -> Result<Vec<(String, String)>, String> {
    let url = format!(
        "{SUPABASE_URL}/rest/v1/messages?select=role,content,created_at&conversation_id=eq.{conv_id}&order=created_at.asc"
    );
    let arr: Vec<Value> = authed(a.get(&url), &pair.access_token)
        .call()
        .map_err(|_| "โหลดประวัติคลาวด์ไม่สำเร็จ".to_string())?
        .into_json()
        .map_err(|_| "อ่านประวัติคลาวด์ไม่สำเร็จ".to_string())?;
    Ok(arr
        .into_iter()
        .filter_map(|m| {
            let role = m.get("role").and_then(Value::as_str)?;
            let content = m.get("content").and_then(Value::as_str)?;
            ((role == "user" || role == "assistant") && !content.trim().is_empty())
                .then(|| (role.to_string(), content.to_string()))
        })
        .collect())
}

/// ดึงประวัติแชทของบัญชีจากคลาวด์ (conversation ล่าสุด) → (conversation_id, [(role, content)])
pub fn pull(agent: &ureq::Agent) -> Result<Option<(String, Vec<(String, String)>)>, String> {
    let a = sync_agent();
    let pair = auth::refresh_token(agent)?;
    let Some(conv_id) = latest_conversation(&a, &pair)? else {
        return Ok(None);
    };
    save_conv_id(&pair.user_id, &conv_id);
    Ok(Some((
        conv_id.clone(),
        cloud_messages(&a, &pair, &conv_id)?,
    )))
}

/// หา conversation ที่เคยซิงก์ หรือสร้างใหม่ → คืน conversation_id
fn ensure_conversation(
    _agent: &ureq::Agent,
    pair: &auth::TokenPair,
    model: &str,
    first_user_text: &str,
) -> Result<String, String> {
    let a = sync_agent();
    if let Some(id) = latest_conversation(&a, pair)? {
        save_conv_id(&pair.user_id, &id);
        return Ok(id);
    }
    // สร้าง conversation ใหม่ (title = ข้อความแรก)
    let title = first_user_text
        .chars()
        .take(60)
        .collect::<String>()
        .replace(['\n', '\r'], " ");
    let url = format!("{SUPABASE_URL}/rest/v1/conversations");
    let resp = authed(
        a.post(&url).set("Prefer", "return=representation"),
        &pair.access_token,
    )
    .send_json(json!({
        "user_id": pair.user_id,
        "title": title,
        "model_id": model,
    }))
    .map_err(|e| {
        eprintln!("[cloud] สร้าง conversation ล้มเหลว: {e}");
        "สร้าง conversation ไม่สำเร็จ".to_string()
    })?;
    let rows: Vec<Value> = resp
        .into_json()
        .map_err(|_| "อ่าน conversation ใหม่ไม่สำเร็จ".to_string())?;
    let id = rows
        .first()
        .and_then(|r| r.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "อ่าน conversation ใหม่ไม่สำเร็จ".to_string())?;
    save_conv_id(&pair.user_id, &id);
    Ok(id)
}

/// เติมเฉพาะข้อความที่ยังไม่มีบนคลาวด์ เพื่อไม่ลบงานที่เพิ่งส่งจากอีกเครื่อง
pub fn push(agent: &ureq::Agent, history: &[Value], model: &str) -> Result<(), String> {
    let msgs = text_messages(history);
    if msgs.is_empty() {
        return Ok(());
    }
    let a = sync_agent();
    let pair = auth::refresh_token(agent)?;
    let conv_id = ensure_conversation(agent, &pair, model, &msgs[0].1)?;
    let remote = cloud_messages(&a, &pair, &conv_id)?;
    let rows: Vec<Value> = msgs
        .iter()
        .filter(|message| !remote.contains(message))
        .map(|(role, content)| {
            json!({
                "conversation_id": conv_id,
                "user_id": pair.user_id,
                "role": role,
                "content": content,
            })
        })
        .collect();
    if rows.is_empty() {
        return Ok(());
    }
    let url = format!("{SUPABASE_URL}/rest/v1/messages");
    match authed(
        a.post(&url).set("Prefer", "return=minimal"),
        &pair.access_token,
    )
    .send_json(json!(rows))
    {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("[cloud] push messages ล้มเหลว: {e}");
            Err("บันทึกประวัติคลาวด์ไม่สำเร็จ".to_string())
        }
    }
}
