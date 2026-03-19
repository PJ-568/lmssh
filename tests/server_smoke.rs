#[test]
fn server_module_compiles() {
    // compile-time smoke
    let _ = lmssh::ssh::server::ServerConfig {
        listen_addr: "127.0.0.1:2222".into(),
    };
}
