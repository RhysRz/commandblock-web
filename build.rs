fn main() {
    println!("cargo:rerun-if-changed=assets/buff-command-block.ico");

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
