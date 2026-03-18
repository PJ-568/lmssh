use std::sync::Arc;

use lmssh::config::Config;
use lmssh::ssh::server::{make_russh_config, HoneypotServer};

#[test]
fn russh_server_config_builds() {
  let cfg = Config::default();
  let server_cfg = make_russh_config(&cfg, vec![]).unwrap();
  let _server = HoneypotServer::new(Arc::new(cfg));
  assert!(server_cfg.methods.contains(russh::MethodSet::PASSWORD));
}
