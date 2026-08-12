fn main() {
    println!("cargo:rerun-if-changed=assets/buff-command-block.ico");
    let build_id = std::process::Command::new("git").args(["rev-parse", "HEAD"]).output()
        .ok().filter(|out| out.status.success()).and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_else(|| "local".to_string());
    println!("cargo:rustc-env=COMMAND_BLOCK_BUILD_ID={}", build_id.trim());

    if std::env::var_os("CARGO_CFG_TARGET_OS").as_deref()
        == Some(std::ffi::OsStr::new("windows"))
    {
        let mut resource = winres::WindowsResource::new();
        resource.set_icon("assets/buff-command-block.ico");
        resource
            .compile()
            .expect("failed to embed the Commandblock executable icon");
    }
}
