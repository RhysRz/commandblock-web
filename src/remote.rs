//! จุดเชื่อมต่อ Remote PC แบบ peer-to-peer ของ CommandBlock Web
//!
//! Supabase ใช้เก็บเฉพาะ SDP สำหรับจับคู่ WebRTC ชั่วคราวเท่านั้น ภาพหน้าจอและ
//! คำสั่งควบคุมเดินทางใน DataChannel ที่เข้ารหัสของ WebRTC โดยตรง
use base64::Engine;
use bytes::BytesMut;
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use futures::FutureExt;
use image::{DynamicImage, ImageBuffer, Rgba};
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use scrap::{Capturer, Display};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{self, ErrorKind, Write};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::Duration;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use webrtc::data_channel::{DataChannel, DataChannelEvent};
use webrtc::peer_connection::{
    register_default_interceptors, MediaEngine, PeerConnection, PeerConnectionBuilder,
    PeerConnectionEventHandler, RTCConfigurationBuilder, RTCIceGatheringState, RTCIceServer,
    RTCPeerConnectionState, RTCSessionDescription, Registry,
};
use webrtc::runtime::{channel, default_runtime, Runtime, Sender};

const SUPABASE_URL: &str = "https://qympivgklmstrnhfaywn.supabase.co";
const SUPABASE_PUBLISHABLE_KEY: &str = "sb_publishable_UJMuyL3QY8lMEWJKZi3zAQ_NFKZY8TH";
const REMOTE_SESSION_TTL: u64 = 600;
const FRAME_CHUNK_BYTES: usize = 12_000;
const REMOTE_CREDENTIAL_SERVICE: &str = "CommandBlock Remote PC";
const REMOTE_CREDENTIAL_ACCOUNT: &str = "approval-secret";

struct Session {
    token: String,
    user_id: String,
}

/// เปิด process console แยก เพื่อไม่ปะปนกับหน้าต่าง GUI หลัก
pub fn launch_sidecar() -> Result<(), String> {
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    // ใช้ sidecar ตัวเดียวกับ Desktop Connector เพื่อให้ผู้ใช้เดิมได้รับ Remote PC
    // ในรอบอัปเดตแรก โดยไม่ต้องพึ่งไฟล์ executable เพิ่มอีกตัว
    let sidecar = current
        .parent()
        .ok_or_else(|| "หาโฟลเดอร์โปรแกรมไม่พบ".to_string())?
        .join("commandblock-connector.exe");
    if !sidecar.is_file() {
        return Err("ไม่พบ commandblock-connector.exe — กรุณาอัปเดต CommandBlock ใหม่".to_string());
    }
    Command::new(sidecar)
        .arg("--remote")
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("เปิด Remote Desktop ไม่สำเร็จ: {e}"))
}

pub fn run(agent: ureq::Agent) -> Result<(), String> {
    println!("CommandBlock Remote PC");
    println!("ลงชื่อเข้าใช้บัญชีเดียวกับ CommandBlock Web (จะไม่บันทึกรหัสผ่าน)");
    let email = prompt("อีเมล: ")?;
    let password = prompt_remote_password()?;
    let session = sign_in(&agent, &email, &password)?;
    let name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows PC".to_string());
    let device = create_device(&agent, &session, &name)?;
    println!("✓ Remote PC ออนไลน์ในชื่อ {name}");
    println!("เปิดหน้าต่างนี้ค้างไว้ แล้วเลือก Remote PC จาก CommandBlock Web (Ctrl+C เพื่อหยุด)");
    loop {
        heartbeat(&agent, &session.token, &device)?;
        if let Some(request) = next_request(&agent, &session.token, &device)? {
            serve_requested_session(&agent, &session, &request)?;
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn prompt(label: &str) -> Result<String, String> {
    let value = prompt_optional(label)?;
    if value.is_empty() {
        Err("ห้ามเว้นว่าง".to_string())
    } else {
        Ok(value)
    }
}

fn prompt_optional(label: &str) -> Result<String, String> {
    print!("{label}");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut value = String::new();
    io::stdin()
        .read_line(&mut value)
        .map_err(|e| e.to_string())?;
    Ok(value.trim().to_string())
}

fn show_password_requested(input: &str) -> bool {
    matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn prompt_remote_password() -> Result<String, String> {
    let choice = prompt_optional("แสดงรหัสผ่านขณะพิมพ์หรือไม่? [y/N]: ")?;
    if show_password_requested(&choice) {
        prompt_optional("รหัสผ่าน (แสดง): ")
    } else {
        rpassword::prompt_password("รหัสผ่าน (ซ่อน): ").map_err(|error| error.to_string())
    }
}

fn auth<'a>(request: ureq::Request, token: &'a str) -> ureq::Request {
    request
        .set("apikey", SUPABASE_PUBLISHABLE_KEY)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json")
}

fn sign_in(agent: &ureq::Agent, email: &str, password: &str) -> Result<Session, String> {
    let response: Value = agent
        .post(&format!("{SUPABASE_URL}/auth/v1/token?grant_type=password"))
        .set("apikey", SUPABASE_PUBLISHABLE_KEY)
        .set("Content-Type", "application/json")
        .send_json(json!({"email":email,"password":password}))
        .map_err(|_| "เข้าสู่ระบบ Remote PC ไม่สำเร็จ".to_string())?
        .into_json()
        .map_err(|_| "อ่าน session Remote PC ไม่สำเร็จ".to_string())?;
    Ok(Session {
        token: response
            .get("access_token")
            .and_then(Value::as_str)
            .ok_or_else(|| "บัญชีหรือรหัสผ่านไม่ถูกต้อง".to_string())?
            .to_owned(),
        user_id: response
            .get("user")
            .and_then(|x| x.get("id"))
            .and_then(Value::as_str)
            .ok_or_else(|| "อ่านผู้ใช้ Remote PC ไม่สำเร็จ".to_string())?
            .to_owned(),
    })
}

fn create_device(agent: &ureq::Agent, session: &Session, name: &str) -> Result<String, String> {
    let rows: Vec<Value> = auth(
        agent.post(&format!("{SUPABASE_URL}/rest/v1/remote_devices")),
        &session.token,
    )
    .set("Prefer", "return=representation")
    .send_json(json!({"user_id":session.user_id,"name":name}))
    .map_err(|_| "ลงทะเบียน Remote PC ไม่สำเร็จ".to_string())?
    .into_json()
    .map_err(|_| "อ่านข้อมูล Remote PC ไม่สำเร็จ".to_string())?;
    rows.first()
        .and_then(|x| x.get("id"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "ไม่ได้ device id จาก Supabase".to_string())
}
fn heartbeat(agent: &ureq::Agent, token: &str, device: &str) -> Result<(), String> {
    auth(
        agent.patch(&format!(
            "{SUPABASE_URL}/rest/v1/remote_devices?id=eq.{device}"
        )),
        token,
    )
    .send_json(json!({"last_seen_at": now_rfc3339()?}))
    .map_err(|_| "ส่ง heartbeat Remote PC ไม่สำเร็จ".to_string())?;
    Ok(())
}
fn next_request(agent: &ureq::Agent, token: &str, device: &str) -> Result<Option<Value>, String> {
    let now = now_rfc3339()?;
    let rows: Vec<Value> = auth(agent.get(&format!("{SUPABASE_URL}/rest/v1/remote_sessions?device_id=eq.{device}&status=eq.requested&expires_at=gt.{now}&order=created_at.asc&limit=1")), token)
        .call().map_err(|_| "ตรวจคำขอ Remote PC ไม่สำเร็จ".to_string())?.into_json().map_err(|_| "อ่านคำขอ Remote PC ไม่สำเร็จ".to_string())?;
    Ok(rows.into_iter().next())
}
fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| e.to_string())
}
fn patch_session(agent: &ureq::Agent, token: &str, id: &str, value: Value) -> Result<(), String> {
    auth(
        agent.patch(&format!(
            "{SUPABASE_URL}/rest/v1/remote_sessions?id=eq.{id}"
        )),
        token,
    )
    .send_json(value)
    .map_err(|_| "อัปเดตสถานะ Remote PC ไม่สำเร็จ".to_string())?;
    Ok(())
}

/// รหัสลับนี้อยู่เฉพาะ Windows Credential Manager ของเครื่องปลายทางเท่านั้น
/// เซิร์ฟเวอร์ได้รับเพียง hash ของรหัส 6 หลักในแต่ละ session
fn device_approval_secret() -> Result<String, String> {
    let entry = keyring::Entry::new(REMOTE_CREDENTIAL_SERVICE, REMOTE_CREDENTIAL_ACCOUNT)
        .map_err(|error| format!("เปิด Windows Credential Manager ไม่สำเร็จ: {error}"))?;
    match entry.get_password() {
        Ok(secret) if !secret.trim().is_empty() => Ok(secret),
        Ok(_) | Err(keyring::Error::NoEntry) => {
            let seed = format!(
                "{}:{}:{:?}",
                std::env::var("COMPUTERNAME").unwrap_or_default(),
                OffsetDateTime::now_utc().unix_timestamp_nanos(),
                std::process::id()
            );
            let secret = sha256_hex(&seed);
            entry
                .set_password(&secret)
                .map_err(|error| format!("บันทึกรหัส Remote PC ไม่สำเร็จ: {error}"))?;
            Ok(secret)
        }
        Err(error) => Err(format!("อ่านรหัส Remote PC ไม่สำเร็จ: {error}")),
    }
}

fn sha256_hex(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn approval_code(secret: &str, session_id: &str) -> String {
    let stamp = OffsetDateTime::now_utc().unix_timestamp_nanos();
    let hash = sha256_hex(&format!("{secret}:{session_id}:{stamp}"));
    let number = u32::from_str_radix(&hash[..8], 16).unwrap_or(0) % 1_000_000;
    format!("{number:06}")
}

fn approval_hash(secret: &str, code: &str) -> String {
    sha256_hex(&format!("{secret}:{code}"))
}

fn audit_event(
    agent: &ureq::Agent,
    session: &Session,
    device_id: &str,
    remote_session_id: &str,
    action: &str,
    mode: &str,
) {
    let _ = auth(
        agent.post(&format!("{SUPABASE_URL}/rest/v1/device_audit_events")),
        &session.token,
    )
    .send_json(json!({
        "user_id": session.user_id,
        "device_kind": "remote",
        "device_id": device_id,
        "remote_session_id": remote_session_id,
        "action": action,
        "mode": mode,
    }));
}

fn wait_for_approval(
    agent: &ureq::Agent,
    token: &str,
    id: &str,
    secret: &str,
    expected_hash: &str,
) -> Result<bool, String> {
    let mut attempts = 0_u8;
    let mut previous_input = String::new();
    for _ in 0..120 {
        thread::sleep(Duration::from_secs(1));
        let rows: Vec<Value> = auth(
            agent.get(&format!(
                "{SUPABASE_URL}/rest/v1/remote_sessions?id=eq.{id}&select=status,approval_code_input"
            )),
            token,
        )
        .call()
        .map_err(|_| "ตรวจรหัสยืนยัน Remote PC ไม่สำเร็จ".to_string())?
        .into_json()
        .map_err(|_| "อ่านรหัสยืนยัน Remote PC ไม่สำเร็จ".to_string())?;
        let row = match rows.first() {
            Some(row) => row,
            None => return Ok(false),
        };
        if row.get("status").and_then(Value::as_str) != Some("requested") {
            return Ok(false);
        }
        let input = row
            .get("approval_code_input")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if input.len() == 6 && input != previous_input {
            previous_input = input.to_string();
            if approval_hash(secret, input) == expected_hash {
                return Ok(true);
            }
            attempts += 1;
            let _ = patch_session(
                agent,
                token,
                id,
                json!({"approval_attempts":attempts,"approval_code_input":null}),
            );
            if attempts >= 5 {
                return Ok(false);
            }
        }
    }
    Ok(false)
}

fn serve_requested_session(
    agent: &ureq::Agent,
    session: &Session,
    request: &Value,
) -> Result<(), String> {
    let id = request
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "คำขอ Remote ไม่มี id".to_string())?;
    let mode = request
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("view");
    let device_id = request
        .get("device_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "คำขอ Remote ไม่มี device id".to_string())?;
    let permission = if mode == "control" {
        "ดูหน้าจอและควบคุมเมาส์/คีย์บอร์ด"
    } else {
        "ดูหน้าจอเท่านั้น"
    };
    let allowed = MessageDialog::new()
        .set_level(MessageLevel::Warning)
        .set_title("CommandBlock Remote PC")
        .set_description(format!(
            "มีคำขอ Remote PC\nสิทธิ์: {permission}\n\nอนุญาตเพียงครั้งนี้หรือไม่?"
        ))
        .set_buttons(MessageButtons::YesNo)
        .show()
        == MessageDialogResult::Yes;
    if !allowed {
        audit_event(agent, session, device_id, id, "denied", mode);
        return patch_session(
            agent,
            &session.token,
            id,
            json!({"status":"denied","closed_reason":"host_denied"}),
        );
    }
    let offer = request
        .get("offer")
        .cloned()
        .ok_or_else(|| "คำขอ Remote ยังเตรียมการเชื่อมต่อไม่ครบ".to_string())?;
    let secret = device_approval_secret()?;
    let code = approval_code(&secret, id);
    let code_hash = approval_hash(&secret, &code);
    let expires_at = (OffsetDateTime::now_utc() + time::Duration::minutes(2))
        .format(&Rfc3339)
        .map_err(|error| error.to_string())?;
    patch_session(
        agent,
        &session.token,
        id,
        json!({
            "approval_code_hash":code_hash,
            "approval_expires_at":expires_at,
            "approval_attempts":0,
            "approval_code_input":null
        }),
    )?;
    audit_event(agent, session, device_id, id, "approval_requested", mode);
    MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title("CommandBlock Remote PC")
        .set_description(format!(
            "ยืนยันเครื่องปลายทางแล้ว\n\nกรอกรหัสนี้บน CommandBlock Web ภายใน 2 นาที:\n\n{code}\n\nห้ามบอกรหัสนี้ให้ผู้อื่น"
        ))
        .set_buttons(MessageButtons::Ok)
        .show();
    if !wait_for_approval(agent, &session.token, id, &secret, &code_hash)? {
        audit_event(agent, session, device_id, id, "approval_failed", mode);
        return patch_session(
            agent,
            &session.token,
            id,
            json!({"status":"denied","closed_reason":"approval_expired_or_invalid"}),
        );
    }
    patch_session(
        agent,
        &session.token,
        id,
        json!({"status":"accepted","host_verified_at":now_rfc3339()?}),
    )?;
    audit_event(agent, session, device_id, id, "accepted", mode);
    match block_on_webrtc(run_peer(
        &offer,
        mode == "control",
        agent.clone(),
        session.token.clone(),
        id.to_owned(),
    )) {
        Ok(()) => {
            audit_event(agent, session, device_id, id, "closed", mode);
            patch_session(
                agent,
                &session.token,
                id,
                json!({"status":"closed","closed_reason":"completed"}),
            )
        }
        Err(error) => {
            audit_event(agent, session, device_id, id, "closed", mode);
            let _ = patch_session(
                agent,
                &session.token,
                id,
                json!({"status":"closed","closed_reason":"connection_error"}),
            );
            Err(error)
        }
    }
}

/// WebRTC ใช้ Tokio ภายใน จึงต้องขับ future ด้วย runtime ของไลบรารีเอง
/// แทน `futures::executor::block_on` ที่ไม่มี async I/O reactor.
fn block_on_webrtc<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("สร้าง Tokio runtime สำหรับ Remote PC ไม่สำเร็จ")
        .block_on(future)
}

#[derive(Clone)]
struct RemoteHandler {
    runtime: Arc<dyn Runtime>,
    control: bool,
    done: Sender<()>,
    ice_done: Sender<()>,
}

fn remote_udp_bind_addrs() -> Vec<std::net::SocketAddr> {
    vec![std::net::SocketAddr::from(([0, 0, 0, 0], 0))]
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for RemoteHandler {
    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        if state == RTCIceGatheringState::Complete {
            let _ = self.ice_done.try_send(());
        }
    }
    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        if state == RTCPeerConnectionState::Failed || state == RTCPeerConnectionState::Closed {
            let _ = self.done.try_send(());
        }
    }
    async fn on_data_channel(&self, dc: Arc<dyn DataChannel>) {
        let runtime = self.runtime.clone();
        let task_runtime = runtime.clone();
        let control = self.control;
        let done = self.done.clone();
        runtime.spawn(Box::pin(async move {
            let _ = serve_data_channel(dc, control, done, task_runtime).await;
        }));
    }
}

async fn run_peer(
    offer: &Value,
    control: bool,
    agent: ureq::Agent,
    token: String,
    id: String,
) -> Result<(), String> {
    let runtime = default_runtime().ok_or_else(|| "ไม่พบ runtime สำหรับ WebRTC".to_string())?;
    let (done_tx, mut done_rx) = channel::<()>(1);
    let (ice_tx, mut ice_rx) = channel::<()>(1);
    let mut media = MediaEngine::default();
    media.register_default_codecs().map_err(|e| e.to_string())?;
    let registry =
        register_default_interceptors(Registry::new(), &mut media).map_err(|e| e.to_string())?;
    let pc = PeerConnectionBuilder::<std::net::SocketAddr>::new()
        .with_configuration(
            RTCConfigurationBuilder::new()
                .with_ice_servers(vec![RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                    ..Default::default()
                }])
                .build(),
        )
        .with_media_engine(media)
        .with_interceptor_registry(registry)
        .with_handler(Arc::new(RemoteHandler {
            runtime: runtime.clone(),
            control,
            done: done_tx.clone(),
            ice_done: ice_tx,
        }))
        .with_udp_addrs(remote_udp_bind_addrs())
        .with_runtime(runtime.clone())
        .build()
        .await
        .map_err(|e| e.to_string())?;
    let offer: RTCSessionDescription = serde_json::from_value(offer.clone())
        .map_err(|_| "รูปแบบ WebRTC offer ไม่ถูกต้อง".to_string())?;
    pc.set_remote_description(offer)
        .await
        .map_err(|e| e.to_string())?;
    let answer = pc.create_answer(None).await.map_err(|e| e.to_string())?;
    pc.set_local_description(answer)
        .await
        .map_err(|e| e.to_string())?;
    ice_rx
        .recv()
        .await
        .ok_or_else(|| "ICE gathering ถูกยกเลิก".to_string())?;
    let answer = pc
        .local_description()
        .await
        .ok_or_else(|| "สร้าง WebRTC answer ไม่สำเร็จ".to_string())?;
    patch_session(
        &agent,
        &token,
        &id,
        json!({"status":"connected", "answer":answer}),
    )?;
    println!("✓ เชื่อมต่อ Remote PC แล้ว (หมดอายุภายใน {REMOTE_SESSION_TTL} วินาทีหากไม่มีการเชื่อมต่อ)");
    done_rx.recv().await;
    pc.close().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn serve_data_channel(
    dc: Arc<dyn DataChannel>,
    control: bool,
    done: Sender<()>,
    runtime: Arc<dyn Runtime>,
) -> Result<(), String> {
    let mut opened = false;
    let mut ticker = runtime.sleep(Duration::from_millis(100));
    let (frames, frame_rx) = mpsc::sync_channel::<(Vec<u8>, usize, usize)>(1);
    let keep_capturing = Arc::new(AtomicBool::new(true));
    let capture_switch = keep_capturing.clone();
    thread::spawn(move || capture_worker(frames, capture_switch));
    loop {
        if opened {
            futures::select! {
                event = dc.poll().fuse() => match event {
                    Some(DataChannelEvent::OnMessage(message)) => { if control { apply_input(&message.data)?; } }
                    Some(DataChannelEvent::OnClose) | None => { keep_capturing.store(false, Ordering::Relaxed); let _ = done.try_send(()); return Ok(()); }
                    _ => {}
                },
                _ = ticker.as_mut().fuse() => {
                    if let Ok((frame, width, height)) = frame_rx.try_recv() { send_frame(&dc, frame, width, height).await?; }
                    ticker = runtime.sleep(Duration::from_millis(100));
                }
            }
        } else {
            match dc.poll().await {
                Some(DataChannelEvent::OnOpen) => opened = true,
                Some(DataChannelEvent::OnClose) | None => {
                    keep_capturing.store(false, Ordering::Relaxed);
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
fn capture_worker(tx: mpsc::SyncSender<(Vec<u8>, usize, usize)>, keep_capturing: Arc<AtomicBool>) {
    let Ok((mut capturer, width, height)) = make_capture() else {
        return;
    };
    while keep_capturing.load(Ordering::Relaxed) {
        match capture_frame(&mut capturer, width, height) {
            Ok(Some(frame)) => {
                let _ = tx.try_send((frame, width, height));
                thread::sleep(Duration::from_millis(500));
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(_) => break,
        }
    }
}

fn make_capture() -> Result<(Capturer, usize, usize), String> {
    let display = Display::primary().map_err(|e| e.to_string())?;
    let capturer = Capturer::new(display).map_err(|e| e.to_string())?;
    let width = capturer.width();
    let height = capturer.height();
    Ok((capturer, width, height))
}
fn capture_frame(
    capturer: &mut Capturer,
    width: usize,
    height: usize,
) -> Result<Option<Vec<u8>>, String> {
    let frame = match capturer.frame() {
        Ok(frame) => frame,
        Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut rgba = Vec::with_capacity(width * height * 4);
    for pixel in frame.chunks_exact(4).take(width * height) {
        rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
    }
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, rgba)
        .ok_or_else(|| "แปลงภาพหน้าจอไม่สำเร็จ".to_string())?;
    let resized = DynamicImage::ImageRgba8(image)
        .resize(1280, 720, image::imageops::FilterType::Triangle)
        .to_rgb8();
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 55)
        .encode_image(&DynamicImage::ImageRgb8(resized))
        .map_err(|e| e.to_string())?;
    Ok(Some(jpeg))
}
async fn send_frame(
    dc: &Arc<dyn DataChannel>,
    frame: Vec<u8>,
    width: usize,
    height: usize,
) -> Result<(), String> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(frame);
    let chunks: Vec<&str> = encoded
        .as_bytes()
        .chunks(FRAME_CHUNK_BYTES)
        .map(|x| std::str::from_utf8(x).unwrap_or(""))
        .collect();
    let id = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis()
    );
    dc.send_text(&json!({"type":"frame","id":id,"width":width,"height":height,"chunks":chunks.len(),"max_frame_bytes":FRAME_CHUNK_BYTES}).to_string()).await.map_err(|e| e.to_string())?;
    for (index, data) in chunks.into_iter().enumerate() {
        dc.send(BytesMut::from(
            json!({"type":"frame_chunk","id":id,"index":index,"data":data})
                .to_string()
                .as_bytes(),
        ))
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn apply_input(bytes: &[u8]) -> Result<(), String> {
    let event: Value = serde_json::from_slice(bytes).map_err(|_| "คำสั่งควบคุมไม่ถูกต้อง".to_string())?;
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    match event.get("type").and_then(Value::as_str) {
        Some("pointer") => {
            let x = event
                .get("x")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let y = event
                .get("y")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let display = Display::primary().map_err(|e| e.to_string())?;
            enigo
                .move_mouse(
                    (x * display.width() as f64) as i32,
                    (y * display.height() as f64) as i32,
                    Coordinate::Abs,
                )
                .map_err(|e| e.to_string())?;
            match event.get("action").and_then(Value::as_str) {
                Some("down") => enigo
                    .button(Button::Left, Direction::Press)
                    .map_err(|e| e.to_string())?,
                Some("up") => enigo
                    .button(Button::Left, Direction::Release)
                    .map_err(|e| e.to_string())?,
                _ => {}
            }
        }
        Some("wheel") => {
            enigo
                .scroll(
                    event.get("delta").and_then(Value::as_i64).unwrap_or(0) as i32,
                    Axis::Vertical,
                )
                .map_err(|e| e.to_string())?;
        }
        Some("text") => {
            enigo
                .text(event.get("value").and_then(Value::as_str).unwrap_or(""))
                .map_err(|e| e.to_string())?;
        }
        Some("key") => {
            let key = match event.get("key").and_then(Value::as_str).unwrap_or("") {
                "Enter" => Key::Return,
                "Backspace" => Key::Backspace,
                "Tab" => Key::Tab,
                "Escape" => Key::Escape,
                other => Key::Unicode(other.chars().next().unwrap_or(' ')),
            };
            enigo
                .key(key, Direction::Click)
                .map_err(|e| e.to_string())?;
        }
        _ => {}
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn chunk_size_stays_inside_webrtc_message_limit() {
        assert!(super::FRAME_CHUNK_BYTES < 16_384);
    }

    #[test]
    fn visible_password_requires_an_explicit_yes() {
        assert!(super::show_password_requested("y"));
        assert!(super::show_password_requested(" YES "));
        assert!(!super::show_password_requested(""));
        assert!(!super::show_password_requested("n"));
        assert!(!super::show_password_requested("show"));
    }

    #[test]
    fn remote_peer_binds_an_ephemeral_udp_socket_for_ice() {
        let expected = "0.0.0.0:0".parse::<std::net::SocketAddr>().unwrap();
        assert_eq!(super::remote_udp_bind_addrs(), vec![expected]);
    }

    #[test]
    fn remote_peer_runs_inside_the_webrtc_runtime() {
        assert_eq!(super::block_on_webrtc(async { 6 * 7 }), 42);
    }
}
