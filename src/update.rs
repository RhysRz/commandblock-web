use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

const RELEASE_URL: &str = "https://api.github.com/repos/RhysRz/commandblock-web/releases/latest";
const PACKAGE: &str = "CommandBlock-Windows-x64.zip";
const CHECKSUM: &str = "CommandBlock-Windows-x64.zip.sha256";

#[derive(Clone, Debug)]
struct Release {
    tag: String,
    notes: String,
    published_at: String,
    release_url: String,
    package_url: String,
    checksum_url: String,
    package_size: Option<u64>,
}

#[derive(Clone, Debug)]
enum UpdateStatus {
    Checking,
    UpToDate,
    Available(Release),
    Downloading {
        tag: String,
        downloaded: u64,
        total: Option<u64>,
        retry: u8,
    },
    Ready {
        tag: String,
    },
    Error(String),
}

static STATUS: OnceLock<Mutex<UpdateStatus>> = OnceLock::new();

fn status_store() -> &'static Mutex<UpdateStatus> {
    STATUS.get_or_init(|| Mutex::new(UpdateStatus::Checking))
}

fn set_status(next: UpdateStatus) {
    if let Ok(mut status) = status_store().lock() {
        *status = next;
    }
}

pub fn build_matches_tag(tag: &str, build: &str) -> bool {
    tag.strip_prefix("build-") == Some(build)
}

pub fn release_is_newer(
    tag: &str,
    published_at: &str,
    current_build: &str,
    current_timestamp: i64,
) -> bool {
    if build_matches_tag(tag, current_build) {
        return false;
    }
    time::OffsetDateTime::parse(
        published_at,
        &time::format_description::well_known::Rfc3339,
    )
    .map(|published| published.unix_timestamp() > current_timestamp)
    .unwrap_or(false)
}

pub fn check_for_update_async() {
    set_status(UpdateStatus::Checking);
    std::thread::spawn(|| match latest_release() {
        Ok(None) => set_status(UpdateStatus::UpToDate),
        Ok(Some(release)) => set_status(UpdateStatus::Available(release)),
        Err(error) => set_status(UpdateStatus::Error(error)),
    });
}

pub fn download_available_release_async() -> Result<(), String> {
    let release = {
        let status = status_store()
            .lock()
            .map_err(|_| "สถานะอัปเดตขัดข้อง".to_string())?;
        match &*status {
            UpdateStatus::Available(release) => release.clone(),
            UpdateStatus::Downloading { .. } => return Ok(()),
            UpdateStatus::Ready { .. } => return Err("ดาวน์โหลดอัปเดตเสร็จแล้ว".to_string()),
            UpdateStatus::Checking => return Err("กำลังตรวจหารุ่นใหม่ กรุณารอสักครู่".to_string()),
            UpdateStatus::UpToDate => return Err("คุณใช้ CommandBlock รุ่นล่าสุดแล้ว".to_string()),
            UpdateStatus::Error(error) => return Err(error.clone()),
        }
    };

    set_status(UpdateStatus::Downloading {
        tag: release.tag.clone(),
        downloaded: 0,
        total: release.package_size,
        retry: 0,
    });
    std::thread::spawn(move || match stage_release(&release) {
        Ok(()) => set_status(UpdateStatus::Ready { tag: release.tag }),
        Err(error) => set_status(UpdateStatus::Error(error)),
    });
    Ok(())
}

pub fn status_json() -> Value {
    let status = status_store().lock();
    let Ok(status) = status else {
        return json!({"state": "error", "message": "สถานะอัปเดตขัดข้อง"});
    };
    match &*status {
        UpdateStatus::Checking => json!({"state": "checking"}),
        UpdateStatus::UpToDate => json!({"state": "up_to_date"}),
        UpdateStatus::Available(release) => json!({
            "state": "available",
            "tag": release.tag,
            "total": release.package_size,
            "release_notes": release.notes,
            "published_at": release.published_at,
            "release_url": release.release_url,
        }),
        UpdateStatus::Downloading {
            tag,
            downloaded,
            total,
            retry,
        } => json!({
            "state": "downloading",
            "tag": tag,
            "downloaded": downloaded,
            "total": total,
            "percent": progress_percent(*downloaded, *total),
            "retry": retry,
        }),
        UpdateStatus::Ready { tag } => json!({"state": "ready", "tag": tag}),
        UpdateStatus::Error(message) => json!({"state": "error", "message": message}),
    }
}

pub fn apply_staged_update() -> bool {
    launch_staged_update().is_ok()
}

pub fn launch_staged_update() -> Result<(), String> {
    crate::diagnostics::create_backup()
        .map_err(|error| format!("สำรองการตั้งค่าก่อนอัปเดตไม่ได้: {error}"))?;
    let Ok(current) = std::env::current_exe() else {
        return Err("ไม่พบตำแหน่ง CommandBlock".to_string());
    };
    let Some(base) = current.parent() else {
        return Err("ไม่พบโฟลเดอร์ CommandBlock".to_string());
    };
    let stage = updates_dir().join("pending");
    if !stage.join("Commandblock.exe").is_file()
        || !stage.join("commandblock-connector.exe").is_file()
        || !stage.join("commandblock-updater.exe").is_file()
    {
        return Err("ยังไม่มีไฟล์อัปเดตที่ตรวจสอบแล้ว".to_string());
    }
    let helper = base.join("commandblock-updater.exe");
    if !helper.is_file() {
        return Err("ไม่พบ commandblock-updater.exe".to_string());
    }
    std::process::Command::new(helper)
        .arg("--apply")
        .arg(&stage)
        .arg(base)
        .arg(std::process::id().to_string())
        .spawn()
        .map_err(|error| format!("เริ่มติดตั้งอัปเดตไม่ได้: {error}"))?;
    Ok(())
}

fn latest_release() -> Result<Option<Release>, String> {
    let current = env!("COMMAND_BLOCK_BUILD_ID");
    let current_timestamp = env!("COMMAND_BLOCK_BUILD_TIMESTAMP")
        .parse::<i64>()
        .unwrap_or(i64::MAX);
    let response: Value = ureq::get(RELEASE_URL)
        .set("User-Agent", "CommandBlock-Updater")
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;
    let tag = response
        .get("tag_name")
        .and_then(Value::as_str)
        .ok_or("ไม่มี release tag")?;
    let published_at = response
        .get("published_at")
        .and_then(Value::as_str)
        .ok_or("release ไม่มีเวลาเผยแพร่")?;
    if !release_is_newer(tag, published_at, current, current_timestamp) {
        return Ok(None);
    }
    let assets = response
        .get("assets")
        .and_then(Value::as_array)
        .ok_or("ไม่มี release assets")?;
    let package = assets
        .iter()
        .find(|asset| asset.get("name").and_then(Value::as_str) == Some(PACKAGE))
        .ok_or("ไม่พบแพ็กเกจ Windows")?;
    let package_url = package
        .get("browser_download_url")
        .and_then(Value::as_str)
        .ok_or("แพ็กเกจไม่มีลิงก์ดาวน์โหลด")?;
    let checksum_url = assets
        .iter()
        .find(|asset| asset.get("name").and_then(Value::as_str) == Some(CHECKSUM))
        .and_then(|asset| asset.get("browser_download_url"))
        .and_then(Value::as_str)
        .ok_or("ไม่พบ checksum แพ็กเกจ")?;
    Ok(Some(Release {
        tag: tag.to_string(),
        notes: response
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or("ไม่มีรายละเอียดเพิ่มเติม")
            .to_string(),
        published_at: published_at.to_string(),
        release_url: response
            .get("html_url")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        package_url: package_url.to_string(),
        checksum_url: checksum_url.to_string(),
        package_size: package.get("size").and_then(Value::as_u64),
    }))
}

fn stage_release(release: &Release) -> Result<(), String> {
    let bytes = read_package_with_progress(release)?;
    let checksum = with_fetch_retries(
        || read_bytes(&release.checksum_url),
        |attempt| std::thread::sleep(retry_delay(attempt)),
    )?;
    let expected = String::from_utf8(checksum)
        .map_err(|e| e.to_string())?
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if expected.len() != 64 || expected != actual {
        return Err("checksum ของแพ็กเกจอัปเดตไม่ถูกต้อง".to_string());
    }
    let stage = updates_dir().join("pending");
    if stage.exists() {
        std::fs::remove_dir_all(&stage).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    for name in ["Commandblock.exe", "commandblock-connector.exe", "commandblock-updater.exe"] {
        let mut entry = zip.by_name(name).map_err(|_| format!("แพ็กเกจไม่มี {name}"))?;
        let mut out = std::fs::File::create(stage.join(name)).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
    }
    std::fs::write(stage.join("build-id.txt"), &release.tag).map_err(|e| e.to_string())?;
    Ok(())
}

fn read_package_with_progress(release: &Release) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut total = release.package_size;
    let mut last_error = String::new();
    for attempt in 0..3 {
        match read_package_attempt(release, &mut out, &mut total, attempt) {
            Ok(()) => return Ok(out),
            Err(error) => {
                last_error = error;
                if attempt == 2 {
                    break;
                }
                set_status(UpdateStatus::Downloading {
                    tag: release.tag.clone(),
                    downloaded: out.len() as u64,
                    total,
                    retry: attempt + 1,
                });
                std::thread::sleep(retry_delay(attempt));
            }
        }
    }
    Err(format!(
        "ดาวน์โหลดอัปเดตไม่สำเร็จหลังลองใหม่ 3 ครั้ง: {last_error}"
    ))
}

fn read_package_attempt(
    release: &Release,
    out: &mut Vec<u8>,
    total: &mut Option<u64>,
    retry: u8,
) -> Result<(), String> {
    let mut request = ureq::get(&release.package_url).set("User-Agent", "CommandBlock-Updater");
    if !out.is_empty() {
        request = request.set("Range", &format!("bytes={}-", out.len()));
    }
    let response = request.call().map_err(|e| e.to_string())?;
    if !out.is_empty() && response.status() == 200 {
        // CDN ไม่รองรับ Range: เริ่มใหม่แทนการต่อไฟล์ที่ซ้ำกัน
        out.clear();
    }
    *total = release.package_size.or_else(|| {
        response
            .header("Content-Length")
            .and_then(|value| value.parse::<u64>().ok())
    });
    let mut reader = response.into_reader();
    let mut buffer = [0u8; 32 * 1024];
    loop {
        let count = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        out.extend_from_slice(&buffer[..count]);
        set_status(UpdateStatus::Downloading {
            tag: release.tag.clone(),
            downloaded: out.len() as u64,
            total: *total,
            retry,
        });
    }
    Ok(())
}

fn retry_delay(attempt: u8) -> Duration {
    Duration::from_secs(u64::from(attempt) + 1)
}

fn read_bytes(url: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut reader = ureq::get(url)
        .set("User-Agent", "CommandBlock-Updater")
        .call()
        .map_err(|e| e.to_string())?
        .into_reader();
    reader.read_to_end(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

fn with_fetch_retries<T, F, P>(mut fetch: F, mut pause: P) -> Result<T, String>
where
    F: FnMut() -> Result<T, String>,
    P: FnMut(u8),
{
    let mut last_error = String::new();
    for attempt in 0..3 {
        match fetch() {
            Ok(value) => return Ok(value),
            Err(error) => {
                last_error = error;
                if attempt < 2 {
                    pause(attempt);
                }
            }
        }
    }
    Err(format!(
        "ดาวน์โหลดไฟล์ตรวจสอบไม่สำเร็จหลังลองใหม่ 3 ครั้ง: {last_error}"
    ))
}

fn progress_percent(downloaded: u64, total: Option<u64>) -> Option<u8> {
    total
        .filter(|total| *total > 0)
        .map(|total| ((downloaded.saturating_mul(100) / total).min(100)) as u8)
}

fn updates_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("CommandBlock")
        .join("updates")
}

#[cfg(test)]
mod tests {
    use super::{build_matches_tag, progress_percent, release_is_newer};

    #[test]
    fn only_the_same_release_build_is_considered_current() {
        assert!(build_matches_tag("build-abc123", "abc123"));
        assert!(!build_matches_tag("build-def456", "abc123"));
        assert!(!build_matches_tag("v1.0.0", "abc123"));
    }

    #[test]
    fn only_newer_runtime_releases_are_offered() {
        assert!(release_is_newer(
            "build-next",
            "2026-08-12T16:00:00Z",
            "current",
            1_786_500_000
        ));
        assert!(!release_is_newer(
            "build-current",
            "2026-08-12T16:00:00Z",
            "current",
            1
        ));
        assert!(!release_is_newer(
            "build-next",
            "2026-08-12T15:00:00Z",
            "current",
            1_786_700_000
        ));
        assert!(!release_is_newer(
            "build-next",
            "not-a-timestamp",
            "current",
            1
        ));
    }

    #[test]
    fn download_progress_is_capped_and_handles_unknown_sizes() {
        assert_eq!(progress_percent(25, Some(100)), Some(25));
        assert_eq!(progress_percent(125, Some(100)), Some(100));
        assert_eq!(progress_percent(25, None), None);
    }

    #[test]
    fn checksum_fetch_retries_until_the_third_attempt() {
        let mut attempts = 0;
        let result = super::with_fetch_retries(
            || {
                attempts += 1;
                if attempts < 3 {
                    Err("Unexpected EOF".to_string())
                } else {
                    Ok("checksum")
                }
            },
            |_| {},
        );
        assert_eq!(result.unwrap(), "checksum");
        assert_eq!(attempts, 3);
    }
}
