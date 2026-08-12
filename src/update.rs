use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const RELEASE_URL: &str = "https://api.github.com/repos/RhysRz/commandblock-web/releases/latest";
const PACKAGE: &str = "CommandBlock-Windows-x64.zip";
const CHECKSUM: &str = "CommandBlock-Windows-x64.zip.sha256";

#[derive(Clone, Debug)]
struct Release {
    tag: String,
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
        }),
        UpdateStatus::Downloading {
            tag,
            downloaded,
            total,
        } => json!({
            "state": "downloading",
            "tag": tag,
            "downloaded": downloaded,
            "total": total,
            "percent": progress_percent(*downloaded, *total),
        }),
        UpdateStatus::Ready { tag } => json!({"state": "ready", "tag": tag}),
        UpdateStatus::Error(message) => json!({"state": "error", "message": message}),
    }
}

pub fn apply_staged_update() -> bool {
    launch_staged_update().is_ok()
}

pub fn launch_staged_update() -> Result<(), String> {
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
    if build_matches_tag(tag, current) {
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
        package_url: package_url.to_string(),
        checksum_url: checksum_url.to_string(),
        package_size: package.get("size").and_then(Value::as_u64),
    }))
}

fn stage_release(release: &Release) -> Result<(), String> {
    let bytes = read_package_with_progress(release)?;
    let expected = String::from_utf8(read_bytes(&release.checksum_url)?)
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
    let response = ureq::get(&release.package_url)
        .set("User-Agent", "CommandBlock-Updater")
        .call()
        .map_err(|e| e.to_string())?;
    let total = response
        .header("Content-Length")
        .and_then(|value| value.parse::<u64>().ok())
        .or(release.package_size);
    let mut reader = response.into_reader();
    let mut out = Vec::new();
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
            total,
        });
    }
    Ok(out)
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
    use super::{build_matches_tag, progress_percent};

    #[test]
    fn only_the_same_release_build_is_considered_current() {
        assert!(build_matches_tag("build-abc123", "abc123"));
        assert!(!build_matches_tag("build-def456", "abc123"));
        assert!(!build_matches_tag("v1.0.0", "abc123"));
    }

    #[test]
    fn download_progress_is_capped_and_handles_unknown_sizes() {
        assert_eq!(progress_percent(25, Some(100)), Some(25));
        assert_eq!(progress_percent(125, Some(100)), Some(100));
        assert_eq!(progress_percent(25, None), None);
    }
}
