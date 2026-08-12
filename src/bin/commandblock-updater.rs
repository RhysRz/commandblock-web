fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 || args[1] != "--apply" {
        return;
    }
    let source = std::path::PathBuf::from(&args[2]);
    let target = std::path::PathBuf::from(&args[3]);
    let pid: u32 = args[4].parse().unwrap_or_default();
    let script = std::env::temp_dir().join(format!("commandblock-update-{pid}.cmd"));
    let quote = |path: &std::path::Path| format!("\"{}\"", path.display());
    let body = format!(
        "@echo off\r\n:wait\r\ntasklist /FI \"PID eq {pid}\" /NH | findstr /C:\" {pid} \" >nul\r\nif not errorlevel 1 (timeout /t 1 /nobreak >nul & goto wait)\r\ntimeout /t 1 /nobreak >nul\r\ncopy /Y {} {} >nul\r\ncopy /Y {} {} >nul\r\ncopy /Y {} {} >nul\r\nstart \"\" {}\r\ndel \"%~f0\"\r\n",
        quote(&source.join("Commandblock.exe")), quote(&target.join("Commandblock.exe")),
        quote(&source.join("commandblock-connector.exe")), quote(&target.join("commandblock-connector.exe")),
        quote(&source.join("commandblock-updater.exe")), quote(&target.join("commandblock-updater.exe")),
        quote(&target.join("Commandblock.exe")),
    );
    if std::fs::write(&script, body).is_ok() {
        let _ = std::process::Command::new("cmd.exe").args(["/C", script.to_string_lossy().as_ref()]).spawn();
    }
}
