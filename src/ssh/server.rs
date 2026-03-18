use crate::config::Config;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ServerConfig {
  pub listen_addr: String,
}

pub fn check_password(users: &[(String, String)], username: &str, password: &str) -> bool {
  users
    .iter()
    .any(|(u, p)| u == username && p == password)
}

pub async fn run_server(_cfg: &Config) -> Result<()> {
  // Task 3 会接入 russh Server/Handler。
  Ok(())
}
