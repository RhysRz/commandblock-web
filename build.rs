fn main() {
    println!("cargo:rerun-if-changed=assets/buff-command-block.ico");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=build.rs");
    let build_id = std::process::Command::new("git")
        .args([
            "log",
            "-1",
            "--format=%H",
            "--",
            "src",
            "assets",
            "Cargo.toml",
            "Cargo.lock",
            "build.rs",
        ])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "local".to_string());
    println!("cargo:rustc-env=COMMAND_BLOCK_BUILD_ID={}", build_id.trim());
    let build_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=COMMAND_BLOCK_BUILD_TIMESTAMP={build_timestamp}");

    if std::env::var_os("CARGO_CFG_TARGET_OS").as_deref() == Some(std::ffi::OsStr::new("windows")) {
        let mut resource = winres::WindowsResource::new();
        resource.set_icon("assets/buff-command-block.ico");
        resource
            .compile()
            .expect("failed to embed the Commandblock executable icon");
    }
}
