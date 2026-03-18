use crate::vfs::VirtualFileSystem;

#[derive(Debug)]
pub struct SessionState {
  pub username: String,
  pub hostname: String,
  pub cwd: String,
  pub client_ip: String,
  pub terminal: String,
  pub term_width: u32,
  pub term_height: u32,

  pub history: Vec<String>,
  pub recent_commands: Vec<(String, String)>,

  pub vfs: VirtualFileSystem,
}

impl SessionState {
  pub fn new_for_test(username: &str, hostname: &str, cwd: &str) -> Self {
    Self {
      username: username.to_string(),
      hostname: hostname.to_string(),
      cwd: cwd.to_string(),
      client_ip: "127.0.0.1".to_string(),
      terminal: "xterm".to_string(),
      term_width: 80,
      term_height: 24,
      history: vec![],
      recent_commands: vec![],
      vfs: VirtualFileSystem::new(username),
    }
  }

  pub fn push_history(&mut self, cmd: &str) {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
      return;
    }
    self.history.push(trimmed.to_string());
  }
}
