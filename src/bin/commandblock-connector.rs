fn main() {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    let result = if std::env::args().any(|arg| arg == "--remote") {
        commandblock::remote::run(agent).map_err(|error| format!("Remote PC: {error}"))
    } else {
        commandblock::connector::run(agent).map_err(|error| format!("Desktop Connector: {error}"))
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
