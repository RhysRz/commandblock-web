fn main() {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    if let Err(error) = commandblock::connector::run(agent) {
        eprintln!("Desktop Connector: {error}");
        std::process::exit(1);
    }
}
