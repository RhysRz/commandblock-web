use commandblock::browser::{requires_confirmation, validate_public_https};

#[test]
fn public_https_url_rejects_private_hosts_and_credentials() {
    assert!(validate_public_https("https://www.google.com/").is_ok());
    assert!(validate_public_https("http://www.google.com/").is_err());
    assert!(validate_public_https("https://user:secret@example.com/").is_err());
    assert!(validate_public_https("https://localhost/").is_err());
    assert!(validate_public_https("https://192.168.1.5/").is_err());
}

#[test]
fn submit_like_controls_require_confirmation() {
    assert!(requires_confirmation("button", "Send message", "submit"));
    assert!(requires_confirmation("a", "Delete account", ""));
    assert!(!requires_confirmation("a", "Next page", ""));
}
