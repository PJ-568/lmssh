use lmssh::config::Config;

#[test]
fn load_full_config() {
    let toml = r#"
    [ssh]
    listen_addr = "127.0.0.1:2222"
    hostname = "debian"
    banner = "SSH-2.0-OpenSSH_9.2p1 Debian-2+deb12u3"
    host_key_path = "data/host_ed25519"
    session_timeout_seconds = 120
    max_input_rate_per_second = 32

    [limits]
    max_output_length = 8192
    max_output_lines = 200
    max_numbered_lines = 15

    [[users]]
    username = "root"
    password = "password"

    [openai]
    api_key = "sk-test"
    base_url = "https://api.openai.com/v1"
    model = "gpt-4o-mini"
    max_tokens = 512
    temperature = 0.6

    [logging]
    dir = "logs"
  "#;

    let cfg = Config::load_from_str(toml).unwrap();
    assert_eq!(cfg.users.len(), 1);
    assert_eq!(cfg.limits.max_output_lines, 200);
    assert_eq!(cfg.logging.dir, "logs");
}
