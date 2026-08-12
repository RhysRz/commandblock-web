//! บัญชีผู้ใช้ของ CommandBlock Desktop — เข้าสู่ระบบด้วยบัญชีเดียวกับ CommandBlock Web
//!
//! ใช้ Supabase Auth (โปรเจกต์เดียวกับ connector/remote) เพื่อสมัคร/เข้าสู่ระบบ
//! และเก็บ refresh token ไว้ใน Windows Credential Manager (keyring) —
//! บัญชีใช้ระบุตัวตนสำหรับเก็บข้อมูลต่อบัญชี (ประวัติแชท, โน้ต) ในเครื่อง
//! ตัวแอปไม่ต้องเรียก Supabase อีกหลังล็อกอิน (แชทใช้โมเดลของ config.json โดยตรง)

use serde_json::{json, Value};

const SUPABASE_URL: &str = "https://qympivgklmstrnhfaywn.supabase.co";
const SUPABASE_PUBLISHABLE_KEY: &str = "sb_publishable_UJMuyL3QY8lMEWJKZi3zAQ_NFKZY8TH";
/// เก็บ session ในไฟล์ .freebuff/auth.json (local) — เสถียรกว่า Windows Credential Manager
/// และตรวจสอบ/ดีบักได้ง่าย (ไฟล์ถูก gitignore แล้ว)
const AUTH_FILE: &str = ".freebuff/auth.json";

/// ข้อมูลบัญชีที่ล็อกอินอยู่ (เก็บในเครื่อง + keyring)
#[derive(Clone, Debug)]
pub struct Account {
    pub email: String,
    pub user_id: String,
    pub refresh_token: String,
}

/// ผลลัพธ์การสมัครสมาชิก
#[derive(Debug)]
pub enum SignUpResult {
    /// สมัครสำเร็จและล็อกอินเข้าไปเลย (โปรเจกต์ไม่บังคับยืนยันอีเมล)
    LoggedIn(Account),
    /// สมัครสำเร็จ แต่ต้องยืนยันอีเมลก่อนเข้าสู่ระบบ
    NeedsConfirmation { email: String },
}

/// อ่านบัญชีที่เคยล็อกอิน (จาก .freebuff/auth.json — ไม่ต้องเชื่อมเน็ต)
pub fn restore() -> Result<Option<Account>, String> {
    let Ok(text) = std::fs::read_to_string(AUTH_FILE) else {
        return Ok(None);
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Ok(None);
    };
    let email = v.get("email").and_then(Value::as_str).unwrap_or("");
    let user_id = v.get("user_id").and_then(Value::as_str).unwrap_or("");
    let refresh_token = v
        .get("refresh_token")
        .and_then(Value::as_str)
        .unwrap_or("");
    if email.is_empty() || user_id.is_empty() || refresh_token.is_empty() {
        return Ok(None);
    }
    Ok(Some(Account {
        email: email.to_string(),
        user_id: user_id.to_string(),
        refresh_token: refresh_token.to_string(),
    }))
}

fn save_account(account: &Account) -> Result<(), String> {
    let _ = std::fs::create_dir_all(".freebuff");
    let payload = json!({
        "email": account.email,
        "user_id": account.user_id,
        "refresh_token": account.refresh_token,
    })
    .to_string();
    std::fs::write(AUTH_FILE, payload).map_err(|e| format!("บันทึก session ไม่สำเร็จ: {e}"))
}

/// ออกจากระบบ — ลบไฟล์ session
pub fn sign_out() -> Result<(), String> {
    match std::fs::remove_file(AUTH_FILE) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("ลบ session ไม่สำเร็จ: {e}")),
    }
}

/// ดึงข้อความ error จาก response ของ Supabase (4xx/5xx)
fn error_from_response(body: &str) -> String {
    let Ok(v) = serde_json::from_str::<Value>(body) else {
        return "การเชื่อมต่อกับเซิร์ฟเวอร์ล้มเหลว".to_string();
    };
    v.get("error_description")
        .and_then(Value::as_str)
        .or_else(|| v.get("error").and_then(Value::as_str))
        .or_else(|| v.get("msg").and_then(Value::as_str))
        .unwrap_or("การเชื่อมต่อกับเซิร์ฟเวอร์ล้มเหลว")
        .to_string()
}

fn account_from_response(response: &str, fallback_email: &str) -> Result<Account, String> {
    let v: Value = serde_json::from_str(response).map_err(|_| "อ่านคำตอบจากเซิร์ฟเวอร์ไม่สำเร็จ")?;
    let refresh_token = v
        .get("refresh_token")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let user = v.get("user").cloned().unwrap_or(Value::Null);
    let user_id = user
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let email = user
        .get("email")
        .and_then(Value::as_str)
        .unwrap_or(fallback_email)
        .to_string();
    if user_id.is_empty() || refresh_token.is_empty() {
        return Err("บัญชีหรือรหัสผ่านไม่ถูกต้อง".to_string());
    }
    Ok(Account {
        email,
        user_id,
        refresh_token,
    })
}

fn post_json(agent: &ureq::Agent, url: &str, payload: Value) -> Result<String, String> {
    let req = agent
        .post(url)
        .set("apikey", SUPABASE_PUBLISHABLE_KEY)
        .set("Content-Type", "application/json");
    match req.send_json(payload) {
        Ok(resp) => resp
            .into_string()
            .map_err(|_| "อ่านคำตอบจากเซิร์ฟเวอร์ไม่สำเร็จ".to_string()),
        Err(ureq::Error::Status(_code, resp)) => {
            let body = resp
                .into_string()
                .unwrap_or_default();
            Err(error_from_response(&body))
        }
        Err(_) => Err("ไม่สามารถเชื่อมต่อเซิร์ฟเวอร์ได้ — ตรวจอินเทอร์เน็ต".to_string()),
    }
}

/// สมัครสมาชิกใหม่ (อีเมล + รหัสผ่าน)
pub fn sign_up(agent: &ureq::Agent, email: &str, password: &str) -> Result<SignUpResult, String> {
    let body = post_json(
        agent,
        &format!("{SUPABASE_URL}/auth/v1/signup"),
        json!({ "email": email, "password": password }),
    )?;
    let v: Value = serde_json::from_str(&body).map_err(|_| "อ่านคำตอบจากเซิร์ฟเวอร์ไม่สำเร็จ")?;
    let has_token = v
        .get("access_token")
        .and_then(Value::as_str)
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    if has_token {
        let account = account_from_response(&body, email)?;
        save_account(&account)?;
        Ok(SignUpResult::LoggedIn(account))
    } else {
        // ต้องยืนยันอีเมลก่อน
        Ok(SignUpResult::NeedsConfirmation {
            email: email.to_string(),
        })
    }
}

/// เข้าสู่ระบบด้วยอีเมล + รหัสผ่าน
pub fn sign_in(agent: &ureq::Agent, email: &str, password: &str) -> Result<Account, String> {
    let body = post_json(
        agent,
        &format!("{SUPABASE_URL}/auth/v1/token?grant_type=password"),
        json!({ "email": email, "password": password }),
    )?;
    let account = account_from_response(&body, email)?;
    save_account(&account)?;
    Ok(account)
}

/// คู่ token ที่เพิ่ง refresh — ใช้เรียก REST API ของ Supabase ได้ทันที
#[derive(Clone, Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub user_id: String,
}

/// ขอ access_token ใหม่จาก refresh token (refresh token หมุนใหม่ทุกครั้ง — เก็บตัวใหม่ลง keyring)
pub fn refresh_token(agent: &ureq::Agent) -> Result<TokenPair, String> {
    let account = restore()?.ok_or_else(|| "ยังไม่ได้เข้าสู่ระบบ".to_string())?;
    let body = post_json(
        agent,
        &format!("{SUPABASE_URL}/auth/v1/token?grant_type=refresh_token"),
        json!({ "refresh_token": account.refresh_token }),
    )?;
    let v: Value =
        serde_json::from_str(&body).map_err(|_| "อ่าน session ใหม่ไม่สำเร็จ".to_string())?;
    let access_token = v
        .get("access_token")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let user_id = v
        .get("user")
        .and_then(|u| u.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if access_token.is_empty() || user_id.is_empty() {
        return Err("session หมดอายุ — กรุณาเข้าสู่ระบบใหม่".to_string());
    }
    // refresh token หมุนรอบทุกครั้ง — บันทึกตัวใหม่ (ถ้า server ให้มา)
    if let Some(rt) = v.get("refresh_token").and_then(Value::as_str) {
        if !rt.is_empty() {
            let _ = save_account(&Account {
                email: account.email.clone(),
                user_id: account.user_id.clone(),
                refresh_token: rt.to_string(),
            });
        }
    }
    Ok(TokenPair {
        access_token,
        user_id,
    })
}

/// ตรวจสอบรูปแบบอีเมลอย่างง่าย
pub fn valid_email(email: &str) -> bool {
    let email = email.trim();
    let Some(at) = email.find('@') else {
        return false;
    };
    at > 0 && email[at + 1..].contains('.') && email.len() <= 254
}
