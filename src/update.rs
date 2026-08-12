use sha2::{Digest, Sha256};
use std::path::PathBuf;

const RELEASE_URL: &str = "https://api.github.com/repos/RhysRz/commandblock-web/releases/latest";
const PACKAGE: &str = "CommandBlock-Windows-x64.zip";

pub fn build_matches_tag(tag: &str, build: &str) -> bool {
    tag.strip_prefix("build-") == Some(build)
}

pub fn stage_newer_release_async() {
    std::thread::spawn(|| { let _ = stage_newer_release(); });
}

pub fn apply_staged_update() -> bool {
    let Ok(current) = std::env::current_exe() else { return false; };
    let Some(base) = current.parent() else { return false; };
    let stage = updates_dir().join("pending");
    if !stage.join("Commandblock.exe").is_file() || !stage.join("commandblock-connector.exe").is_file() { return false; }
    let helper = base.join("commandblock-updater.exe");
    if !helper.is_file() { return false; }
    std::process::Command::new(helper).arg("--apply").arg(&stage).arg(base).arg(std::process::id().to_string()).spawn().is_ok()
}

fn stage_newer_release() -> Result<(), String> {
    let current = env!("COMMAND_BLOCK_BUILD_ID");
    let response: serde_json::Value = ureq::get(RELEASE_URL).set("User-Agent", "CommandBlock-Updater").call()
        .map_err(|e| e.to_string())?.into_json().map_err(|e| e.to_string())?;
    let tag = response.get("tag_name").and_then(serde_json::Value::as_str).ok_or("ไม่มี release tag")?;
    if build_matches_tag(tag, current) { return Ok(()); }
    let assets = response.get("assets").and_then(serde_json::Value::as_array).ok_or("ไม่มี release assets")?;
    let url = assets.iter().find(|a| a.get("name").and_then(serde_json::Value::as_str) == Some(PACKAGE))
        .and_then(|a| a.get("browser_download_url")).and_then(serde_json::Value::as_str).ok_or("ไม่พบแพ็กเกจ Windows")?;
    let checksum_url = assets.iter().find(|a| a.get("name").and_then(serde_json::Value::as_str) == Some("CommandBlock-Windows-x64.zip.sha256"))
        .and_then(|a| a.get("browser_download_url")).and_then(serde_json::Value::as_str).ok_or("ไม่พบ checksum แพ็กเกจ")?;
    let bytes = read_bytes(url)?;
    let expected = String::from_utf8(read_bytes(checksum_url)?).map_err(|e| e.to_string())?.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if expected.len() != 64 || expected != actual { return Err("checksum ของแพ็กเกจอัปเดตไม่ถูกต้อง".to_string()); }
    let stage = updates_dir().join("pending");
    if stage.exists() { std::fs::remove_dir_all(&stage).map_err(|e| e.to_string())?; }
    std::fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    for name in ["Commandblock.exe", "commandblock-connector.exe"] {
        let mut entry = zip.by_name(name).map_err(|_| format!("แพ็กเกจไม่มี {name}"))?;
        let mut out = std::fs::File::create(stage.join(name)).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
    }
    std::fs::write(stage.join("build-id.txt"), tag).map_err(|e| e.to_string())?;
    Ok(())
}

fn read_bytes(url: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut reader = ureq::get(url).set("User-Agent", "CommandBlock-Updater").call().map_err(|e| e.to_string())?.into_reader();
    std::io::Read::read_to_end(&mut reader, &mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

fn updates_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("CommandBlock").join("updates")
}

#[cfg(test)]
mod tests {
    use super::build_matches_tag;
    #[test]
    fn only_the_same_release_build_is_considered_current() {
        assert!(build_matches_tag("build-abc123", "abc123"));
        assert!(!build_matches_tag("build-def456", "abc123"));
        assert!(!build_matches_tag("v1.0.0", "abc123"));
    }
}
