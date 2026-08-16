//! สัญญากลางสำหรับ Native Browser — โค้ด WebView จริงอยู่ใน GUI thread เท่านั้น

use serde_json::{json, Value};
use std::sync::{Arc, Mutex, OnceLock};
use url::{Host, Url};

/// ตำแหน่ง Preview native browser ในหน้าต่างหลัก หน่วยเป็น physical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrowserBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl BrowserBounds {
    pub const EMPTY: Self = Self {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
    };

    pub fn is_visible(self) -> bool {
        self.width > 1 && self.height > 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserCommand {
    Show { url: String, bounds: BrowserBounds },
    Hide,
    SetBounds { bounds: BrowserBounds },
    Navigate { url: String },
    Back,
    Forward,
    Reload,
    Inspect,
    Click { selector: String },
    ConfirmPending,
    CancelPending,
    Fill { selector: String, value: String },
    Press { key: String },
    Scroll { direction: ScrollDirection },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserReply {
    Ok {
        message: String,
        url: String,
        details: Value,
    },
    ConfirmationRequired {
        site: String,
        action: String,
        selector: String,
    },
    Error(String),
}

impl BrowserReply {
    pub fn ok(message: impl Into<String>, url: impl Into<String>, details: Value) -> Self {
        Self::Ok {
            message: message.into(),
            url: url.into(),
            details,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }

    pub fn to_tool_text(&self) -> String {
        match self {
            Self::Ok {
                message,
                url,
                details,
            } => {
                let tail = if details.is_null() || details == &json!({}) {
                    String::new()
                } else {
                    format!("\nรายละเอียด: {}", details)
                };
                format!("[Browser] {message}\nURL: {url}{tail}")
            }
            Self::ConfirmationRequired {
                site,
                action,
                selector,
            } => format!(
                "[Browser: ต้องยืนยัน] เว็บไซต์ {site} กำลังจะ {action} ({selector}) — รอผู้ใช้ยืนยันในหน้าต่าง CommandBlock"
            ),
            Self::Error(message) => format!("[Browser] {message}"),
        }
    }

    pub fn as_json(&self) -> Value {
        match self {
            Self::Ok {
                message,
                url,
                details,
            } => json!({"ok": true, "message": message, "url": url, "details": details}),
            Self::ConfirmationRequired {
                site,
                action,
                selector,
            } => json!({
                "ok": false,
                "confirmation_required": true,
                "site": site,
                "action": action,
                "selector": selector,
            }),
            Self::Error(message) => json!({"ok": false, "error": message}),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserConfirmation {
    pub site: String,
    pub action: String,
    pub selector: String,
}

fn confirmation_slot() -> &'static Mutex<Option<BrowserConfirmation>> {
    static SLOT: OnceLock<Mutex<Option<BrowserConfirmation>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// ส่งคำขอยืนยันจาก worker ของ AI ไปยัง UI โดยไม่เปิดทางให้ AI ยืนยันเอง.
pub fn record_confirmation(reply: &BrowserReply) {
    let BrowserReply::ConfirmationRequired { site, action, selector } = reply else {
        return;
    };
    if let Ok(mut slot) = confirmation_slot().lock() {
        *slot = Some(BrowserConfirmation {
            site: site.clone(),
            action: action.clone(),
            selector: selector.clone(),
        });
    }
}

pub fn take_confirmation() -> Option<BrowserConfirmation> {
    confirmation_slot().lock().ok().and_then(|mut slot| slot.take())
}

/// สะพานแบบ thread-safe จาก worker ของ AI ไปยัง native UI thread.
pub trait BrowserBridge: Send + Sync {
    fn dispatch(&self, command: BrowserCommand) -> BrowserReply;
}

fn bridge_slot() -> &'static Mutex<Option<Arc<dyn BrowserBridge>>> {
    static SLOT: OnceLock<Mutex<Option<Arc<dyn BrowserBridge>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

pub fn register_bridge(bridge: Arc<dyn BrowserBridge>) {
    if let Ok(mut slot) = bridge_slot().lock() {
        *slot = Some(bridge);
    }
}

pub fn clear_bridge() {
    if let Ok(mut slot) = bridge_slot().lock() {
        *slot = None;
    }
}

pub fn dispatch(command: BrowserCommand) -> BrowserReply {
    let bridge = bridge_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().cloned());
    match bridge {
        Some(bridge) => bridge.dispatch(command),
        None => BrowserReply::error("Native Browser ยังไม่พร้อม — เปิด CommandBlock Desktop ก่อน"),
    }
}

fn host_is_private(host: &Host<&str>) -> bool {
    match host {
        Host::Domain(domain) => {
            let domain = domain.to_ascii_lowercase();
            domain == "localhost"
                || domain.ends_with(".localhost")
                || domain.ends_with(".local")
                || domain.ends_with(".internal")
        }
        Host::Ipv4(ip) => {
            ip.is_loopback() || ip.is_unspecified() || ip.is_private() || ip.is_link_local()
        }
        Host::Ipv6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
        }
    }
}

/// อนุญาตเฉพาะเว็บไซต์ HTTPS สาธารณะ และตัด URL ที่อาจพา credentials ออกนอกเครื่อง.
pub fn validate_public_https(raw_url: &str) -> Result<String, String> {
    let parsed = Url::parse(raw_url.trim()).map_err(|_| "ลิงก์ Browser ไม่ถูกต้อง".to_string())?;
    if parsed.scheme() != "https" {
        return Err("Browser URL ต้องเริ่มด้วย https:// เท่านั้น".to_string());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("Browser URL ต้องไม่มีข้อมูลเข้าสู่ระบบในลิงก์".to_string());
    }
    let host = parsed
        .host()
        .ok_or_else(|| "Browser URL ต้องมีชื่อเว็บไซต์".to_string())?;
    if host_is_private(&host) {
        return Err("Browser URL ต้องเป็นเว็บไซต์สาธารณะ ไม่ใช่ localhost หรือเครือข่ายภายใน".to_string());
    }
    Ok(parsed.to_string())
}

/// ป้องกันการให้ AI ส่ง/ลบ/ซื้อผ่านเว็บไซต์โดยไม่มีการยืนยันจากผู้ใช้.
pub fn requires_confirmation(tag: &str, label: &str, control_type: &str) -> bool {
    let combined = format!(
        "{} {} {}",
        tag.to_ascii_lowercase(),
        label.to_ascii_lowercase(),
        control_type.to_ascii_lowercase()
    );
    [
        "submit",
        "send",
        "post",
        "publish",
        "delete",
        "remove",
        "buy",
        "purchase",
        "pay",
        "confirm",
        "sign out",
        "logout",
        "ส่ง",
        "โพสต์",
        "ลบ",
        "ซื้อ",
        "ชำระ",
        "ยืนยัน",
        "ออกจากระบบ",
    ]
    .iter()
    .any(|keyword| combined.contains(keyword))
}

pub fn validate_selector(selector: &str) -> Result<String, String> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Err("ต้องระบุ selector จากผล browser_inspect".to_string());
    }
    if selector.len() > 500 || selector.contains('\0') {
        return Err("selector ไม่ถูกต้อง".to_string());
    }
    Ok(selector.to_string())
}
