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

#[derive(Clone, Debug)]
pub struct CloudMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    pub is_pinned: bool,
}

#[derive(Clone, Debug)]
pub struct CloudConversation {
    pub id: String,
    pub title: String,
    pub model_id: String,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

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

fn conversations(a: &ureq::Agent, pair: &auth::TokenPair) -> Result<Vec<CloudConversation>, String> {
    let url = format!(
        "{SUPABASE_URL}/rest/v1/conversations?select=id,title,model_id,is_pinned,created_at,updated_at&user_id=eq.{}&order=is_pinned.desc,updated_at.desc,id.desc",
        pair.user_id
    );
    let rows: Vec<Value> = authed(a.get(&url), &pair.access_token)
        .call()
        .map_err(|_| "โหลดรายการสนทนาคลาวด์ไม่สำเร็จ".to_string())?
        .into_json()
        .map_err(|_| "อ่านรายการสนทนาคลาวด์ไม่สำเร็จ".to_string())?;
    Ok(rows
        .into_iter()
        .filter_map(|row| Some(CloudConversation {
            id: row.get("id")?.as_str()?.to_string(),
            title: row.get("title").and_then(Value::as_str).unwrap_or("แชทใหม่").to_string(),
            model_id: row.get("model_id").and_then(Value::as_str).unwrap_or_default().to_string(),
            is_pinned: row.get("is_pinned").and_then(Value::as_bool).unwrap_or(false),
            created_at: row.get("created_at").and_then(Value::as_str).unwrap_or_default().to_string(),
            updated_at: row.get("updated_at").and_then(Value::as_str).unwrap_or_default().to_string(),
        }))
        .collect())
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// ใช้ conversation ล่าสุดของบัญชีเป็นค่าเริ่มต้นเท่านั้น; หลังเลือก Session แล้ว GUI จะส่ง id ชัดเจน
fn latest_conversation(a: &ureq::Agent, pair: &auth::TokenPair) -> Result<Option<String>, String> {
    Ok(conversations(a, pair)?.first().map(|row| row.id.clone()))
}

pub fn list_conversations(agent: &ureq::Agent) -> Result<Vec<CloudConversation>, String> {
    let pair = auth::refresh_token(agent)?;
    conversations(&sync_agent(), &pair)
}

pub fn create_conversation(agent: &ureq::Agent, model: &str) -> Result<CloudConversation, String> {
    let pair = auth::refresh_token(agent)?;
    let a = sync_agent();
    let url = format!("{SUPABASE_URL}/rest/v1/conversations");
    let rows: Vec<Value> = authed(
        a.post(&url).set("Prefer", "return=representation"),
        &pair.access_token,
    )
    .send_json(json!({
        "user_id": pair.user_id,
        "title": "แชทใหม่",
        "model_id": model,
    }))
    .map_err(|_| "สร้าง SESSION ไม่สำเร็จ".to_string())?
    .into_json()
    .map_err(|_| "อ่าน SESSION ใหม่ไม่สำเร็จ".to_string())?;
    let row = rows.first().ok_or_else(|| "อ่าน SESSION ใหม่ไม่สำเร็จ".to_string())?;
    let conversation = CloudConversation {
        id: row.get("id").and_then(Value::as_str).ok_or_else(|| "อ่าน SESSION ใหม่ไม่สำเร็จ".to_string())?.to_string(),
        title: row.get("title").and_then(Value::as_str).unwrap_or("แชทใหม่").to_string(),
        model_id: row.get("model_id").and_then(Value::as_str).unwrap_or(model).to_string(),
        is_pinned: row.get("is_pinned").and_then(Value::as_bool).unwrap_or(false),
        created_at: row.get("created_at").and_then(Value::as_str).unwrap_or_default().to_string(),
        updated_at: row.get("updated_at").and_then(Value::as_str).unwrap_or_default().to_string(),
    };
    save_conv_id(&pair.user_id, &conversation.id);
    Ok(conversation)
}

pub fn delete_conversation(agent: &ureq::Agent, conversation_id: &str) -> Result<(), String> {
    if conversation_id.is_empty()
        || !conversation_id
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() || ch == '-')
    {
        return Err("รหัส SESSION ไม่ถูกต้อง".to_string());
    }
    let pair = auth::refresh_token(agent)?;
    let a = sync_agent();
    let url = format!(
        "{SUPABASE_URL}/rest/v1/conversations?id=eq.{conversation_id}&user_id=eq.{}",
        pair.user_id
    );
    authed(a.delete(&url), &pair.access_token)
        .call()
        .map_err(|_| "ลบ SESSION ไม่สำเร็จ".to_string())?;
    Ok(())
}

pub fn set_conversation_pin(
    agent: &ureq::Agent,
    conversation_id: &str,
    is_pinned: bool,
) -> Result<bool, String> {
    if conversation_id.is_empty()
        || !conversation_id
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() || ch == '-')
    {
        return Err("รหัส SESSION ไม่ถูกต้อง".to_string());
    }
    let pair = auth::refresh_token(agent)?;
    let a = sync_agent();
    let url = format!(
        "{SUPABASE_URL}/rest/v1/conversations?id=eq.{conversation_id}&user_id=eq.{}",
        pair.user_id
    );
    let rows: Vec<Value> = authed(
        a.patch(&url).set("Prefer", "return=representation"),
        &pair.access_token,
    )
    .send_json(json!({"is_pinned": is_pinned}))
    .map_err(|_| "บันทึกการปักหมุด SESSION ไม่สำเร็จ".to_string())?
    .into_json()
    .map_err(|_| "อ่านผลการปักหมุด SESSION ไม่สำเร็จ".to_string())?;
    if rows.is_empty() {
        return Err("ไม่พบ SESSION นี้".to_string());
    }
    Ok(is_pinned)
}

fn cloud_messages(
    a: &ureq::Agent,
    pair: &auth::TokenPair,
    conv_id: &str,
) -> Result<Vec<CloudMessage>, String> {
    let url = format!(
        "{SUPABASE_URL}/rest/v1/messages?select=id,role,content,created_at,is_pinned&conversation_id=eq.{conv_id}&user_id=eq.{}&order=created_at.asc,id.asc",
        pair.user_id
    );
    let arr: Vec<Value> = authed(a.get(&url), &pair.access_token)
        .call()
        .map_err(|_| "โหลดประวัติคลาวด์ไม่สำเร็จ".to_string())?
        .into_json()
        .map_err(|_| "อ่านประวัติคลาวด์ไม่สำเร็จ".to_string())?;
    Ok(arr
        .into_iter()
        .filter_map(|m| {
            let id = m.get("id").and_then(Value::as_str)?;
            let role = m.get("role").and_then(Value::as_str)?;
            let content = m.get("content").and_then(Value::as_str)?;
            let created_at = m.get("created_at").and_then(Value::as_str).unwrap_or("");
            let is_pinned = m.get("is_pinned").and_then(Value::as_bool).unwrap_or(false);
            ((role == "user" || role == "assistant") && !content.trim().is_empty())
                .then(|| CloudMessage {
                    id: id.to_string(),
                    role: role.to_string(),
                    content: content.to_string(),
                    created_at: created_at.to_string(),
                    is_pinned,
                })
        })
        .collect())
}

/// ดึงประวัติแชทของบัญชีจากคลาวด์ พร้อม ID/เวลาเพื่อเรียง UI ข้ามเครื่อง
pub fn pull(agent: &ureq::Agent) -> Result<Option<(String, Vec<CloudMessage>)>, String> {
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

pub fn pull_conversation(agent: &ureq::Agent, conv_id: &str) -> Result<Vec<CloudMessage>, String> {
    if conv_id.is_empty() || !conv_id.chars().all(|ch| ch.is_ascii_hexdigit() || ch == '-') {
        return Err("รหัส SESSION ไม่ถูกต้อง".to_string());
    }
    let pair = auth::refresh_token(agent)?;
    let a = sync_agent();
    let url = format!(
        "{SUPABASE_URL}/rest/v1/conversations?select=id&id=eq.{conv_id}&user_id=eq.{}&limit=1",
        pair.user_id
    );
    let rows: Vec<Value> = authed(a.get(&url), &pair.access_token)
        .call()
        .map_err(|_| "ตรวจสอบ SESSION ไม่สำเร็จ".to_string())?
        .into_json()
        .map_err(|_| "อ่าน SESSION ไม่สำเร็จ".to_string())?;
    if rows.is_empty() {
        return Err("ไม่พบ SESSION นี้".to_string());
    }
    save_conv_id(&pair.user_id, conv_id);
    cloud_messages(&a, &pair, conv_id)
}

pub fn set_message_pin(agent: &ureq::Agent, message_id: &str, is_pinned: bool) -> Result<bool, String> {
    if message_id.is_empty() || !message_id.chars().all(|ch| ch.is_ascii_hexdigit() || ch == '-') {
        return Err("รหัสข้อความไม่ถูกต้อง".to_string());
    }
    let pair = auth::refresh_token(agent)?;
    let a = sync_agent();
    let url = format!("{SUPABASE_URL}/rest/v1/messages?id=eq.{message_id}&user_id=eq.{}", pair.user_id);
    let rows: Vec<Value> = authed(
        a.patch(&url).set("Prefer", "return=representation"),
        &pair.access_token,
    )
    .send_json(json!({"is_pinned": is_pinned}))
    .map_err(|_| "บันทึกการปักหมุดไม่สำเร็จ".to_string())?
    .into_json()
    .map_err(|_| "อ่านผลการปักหมุดไม่สำเร็จ".to_string())?;
    if rows.is_empty() {
        return Err("ไม่พบข้อความนี้".to_string());
    }
    Ok(is_pinned)
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
pub fn push(agent: &ureq::Agent, history: &[Value], model: &str, selected_conversation: Option<&str>) -> Result<String, String> {
    let msgs = text_messages(history);
    if msgs.is_empty() {
        return Ok(selected_conversation.unwrap_or_default().to_string());
    }
    let a = sync_agent();
    let pair = auth::refresh_token(agent)?;
    let conv_id = selected_conversation
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .unwrap_or(ensure_conversation(agent, &pair, model, &msgs[0].1)?);
    let remote = cloud_messages(&a, &pair, &conv_id)?;
    let rows: Vec<Value> = msgs
        .iter()
        .filter(|message| !remote.iter().any(|row| row.role == message.0 && row.content == message.1))
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
        return Ok(conv_id);
    }
    let url = format!("{SUPABASE_URL}/rest/v1/messages");
    match authed(
        a.post(&url).set("Prefer", "return=minimal"),
        &pair.access_token,
    )
    .send_json(json!(rows))
    {
        Ok(_) => {
            let conversation_url = format!("{SUPABASE_URL}/rest/v1/conversations?id=eq.{conv_id}&user_id=eq.{}", pair.user_id);
            let _ = authed(a.patch(&conversation_url), &pair.access_token)
                .send_json(json!({"updated_at": now_rfc3339()}));
            save_conv_id(&pair.user_id, &conv_id);
            Ok(conv_id)
        },
        Err(e) => {
            eprintln!("[cloud] push messages ล้มเหลว: {e}");
            Err("บันทึกประวัติคลาวด์ไม่สำเร็จ".to_string())
        }
    }
}
