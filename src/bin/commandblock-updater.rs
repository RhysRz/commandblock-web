fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 || args[1] != "--apply" {
        return;
    }
    let source = std::path::PathBuf::from(&args[2]);
    let target = std::path::PathBuf::from(&args[3]);
    let pid: u32 = args[4].parse().unwrap_or_default();
    for _ in 0..30 {
        if !process_exists(pid) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    for file in ["Commandblock.exe", "commandblock-connector.exe"] {
        let _ = std::fs::copy(source.join(file), target.join(file));
    }
    let _ = std::process::Command::new(target.join("Commandblock.exe")).spawn();
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}
#[cfg(not(windows))]
fn process_exists(_: u32) -> bool {
    false
}
