#[test]
fn load_config_from_toml_str() {
    let toml = r#"
    [ssh]
    listen_addr = "127.0.0.1:2222"
    hostname = "debian"
    banner = "SSH-2.0-OpenSSH_9.2p1 Debian-2+deb12u3"
    host_key_path = "data/host_ed25519"

    [openai]
    api_key = "sk-test"
    base_url = "https://api.openai.com/v1"
    model = "gpt-4o-mini"
  "#;
    let cfg = lmssh::config::Config::load_from_str(toml).unwrap();
    assert_eq!(cfg.ssh.hostname, "debian");
}
