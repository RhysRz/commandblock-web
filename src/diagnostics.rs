use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const BACKUP_LIMIT: usize = 5;

#[derive(Clone, Debug, Serialize)]
pub struct BackupInfo {
    pub name: String,
    pub created_at: u64,
}

pub fn install_panic_reporter() {
    std::panic::set_hook(Box::new(|panic| {
        let location = panic
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
            .unwrap_or_else(|| "ไม่ทราบตำแหน่ง".to_string());
        let report = format!(
            "CommandBlock crash report\ncreated_at={}\nversion={}\nbuild_id={}\nos={}\nlocation={}\nmessage=Unexpected application failure\n",
            now(),
            env!("CARGO_PKG_VERSION"),
            env!("COMMAND_BLOCK_BUILD_ID"),
            std::env::consts::OS,
            location,
        );
        let _ = std::fs::create_dir_all(reports_dir());
        let _ = std::fs::write(reports_dir().join(format!("crash-{}.txt", now())), report);
    }));
}

pub fn latest_report() -> String {
    let Some(path) = report_files().into_iter().next() else {
        return "ยังไม่มี crash report ในเครื่องนี้".to_string();
    };
    std::fs::read_to_string(path).unwrap_or_else(|_| "อ่าน crash report ไม่สำเร็จ".to_string())
}

pub fn create_backup() -> Result<BackupInfo, String> {
    let created_at = now();
    let backup = json!({
        "version": 1,
        "created_at": created_at,
        "config": read_optional(&config_path())?,
        "project_settings": read_optional(&project_settings_path())?,
    });
    std::fs::create_dir_all(backups_dir()).map_err(|error| error.to_string())?;
    let name = format!("settings-{created_at}.json");
    std::fs::write(
        backups_dir().join(&name),
        serde_json::to_vec_pretty(&backup).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    prune_backups()?;
    Ok(BackupInfo { name, created_at })
}

pub fn list_backups() -> Vec<BackupInfo> {
    backup_files()
        .into_iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().to_string();
            let created_at = name
                .strip_prefix("settings-")?
                .strip_suffix(".json")?
                .parse()
                .ok()?;
            Some(BackupInfo { name, created_at })
        })
        .collect()
}

pub fn restore_backup(name: &str) -> Result<(), String> {
    let safe_name = Path::new(name)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| value == &name && value.starts_with("settings-") && value.ends_with(".json"))
        .ok_or("ชื่อ backup ไม่ถูกต้อง")?;
    let text = std::fs::read_to_string(backups_dir().join(safe_name))
        .map_err(|error| format!("อ่าน backup ไม่สำเร็จ: {error}"))?;
    let backup: Value = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    write_optional(&config_path(), backup.get("config"))?;
    write_optional(&project_settings_path(), backup.get("project_settings"))?;
    Ok(())
}

fn write_optional(path: &Path, value: Option<&Value>) -> Result<(), String> {
    match value.and_then(Value::as_str) {
        Some(text) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            std::fs::write(path, text).map_err(|error| error.to_string())
        }
        None => {
            if path.exists() {
                std::fs::remove_file(path).map_err(|error| error.to_string())?;
            }
            Ok(())
        }
    }
}

fn read_optional(path: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn prune_backups() -> Result<(), String> {
    for path in backup_files().into_iter().skip(BACKUP_LIMIT) {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn backup_files() -> Vec<PathBuf> {
    let mut files = std::fs::read_dir(backups_dir())
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("settings-") && name.ends_with(".json"))
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| right.file_name().cmp(&left.file_name()));
    files
}

fn report_files() -> Vec<PathBuf> {
    let mut files = std::fs::read_dir(reports_dir())
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("txt"))
        .collect::<Vec<_>>();
    files.sort_by(|left, right| right.file_name().cmp(&left.file_name()));
    files
}

fn data_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("CommandBlock")
}

fn reports_dir() -> PathBuf {
    data_dir().join("reports")
}

fn backups_dir() -> PathBuf {
    data_dir().join("backups")
}

fn project_settings_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".freebuff")
        .join("settings.json")
}

fn config_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut path = cwd.join("config.json");
    if !path.exists() {
        if let Some(parent) = std::env::current_exe()
            .ok()
            .and_then(|executable| executable.parent().map(Path::to_path_buf))
        {
            let alongside_executable = parent.join("config.json");
            if alongside_executable.exists() {
                path = alongside_executable;
            }
        }
    }
    path
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::BACKUP_LIMIT;

    #[test]
    fn keeps_five_or_fewer_backup_files() {
        assert_eq!(BACKUP_LIMIT, 5);
    }
}
