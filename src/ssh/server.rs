use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use russh::keys::key::KeyPair;
use russh::server::{Auth, Msg, Session};
use russh::server::Server as _;
use russh::{Channel, ChannelId, CryptoVec, MethodSet};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::Result;
use crate::ai::client::OpenAiClient;
use crate::ai::types::{ChatCompletionsRequest, ChatMessage};
use crate::logging::event::{Event, EventKind};
use crate::logging::jsonl::JsonlLogger;
use crate::session::{Action, CommandExecutor, OutputGuard, Router, SessionState};
use crate::terminal::LineEditor;

#[derive(Debug, Clone)]
pub struct ServerConfig {
  pub listen_addr: String,
}

pub fn check_password(users: &[(String, String)], username: &str, password: &str) -> bool {
  users
    .iter()
    .any(|(u, p)| u == username && p == password)
}

pub fn make_russh_config(cfg: &Config, keys: Vec<KeyPair>) -> Result<russh::server::Config> {
  Ok(russh::server::Config {
    server_id: russh::SshId::Standard(cfg.ssh.banner.clone()),
    methods: MethodSet::PASSWORD,
    auth_rejection_time: std::time::Duration::from_secs(1),
    inactivity_timeout: Some(std::time::Duration::from_secs(cfg.ssh.session_timeout_seconds)),
    keys,
    ..Default::default()
  })
}

#[derive(Clone)]
pub struct HoneypotServer {
  shared: Arc<Shared>,
}

impl HoneypotServer {
  pub fn new(cfg: Arc<Config>) -> Self {
    Self {
      shared: Arc::new(Shared { cfg }),
    }
  }
}

struct Shared {
  cfg: Arc<Config>,
}

struct SessionIo {
  channel: Option<ChannelId>,
  session_id: String,
  username: String,
  state: Option<SessionState>,
  editor: LineEditor,
  logger: Option<JsonlLogger>,
}

impl SessionIo {
  fn new(max_input_rate: u32) -> Self {
    let _ = max_input_rate;
    Self {
      channel: None,
      session_id: format!(
        "sess-{}",
        std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .map(|d| d.as_nanos())
          .unwrap_or_default()
      ),
      username: String::new(),
      state: None,
      editor: LineEditor::new(),
      logger: None,
    }
  }
}

impl russh::server::Server for HoneypotServer {
  type Handler = HoneypotHandler;

  fn new_client(&mut self, peer_addr: Option<SocketAddr>) -> Self::Handler {
    HoneypotHandler::new(self.shared.clone(), peer_addr)
  }
}

pub struct HoneypotHandler {
  shared: Arc<Shared>,
  peer_addr: Option<SocketAddr>,
  io: Arc<Mutex<SessionIo>>,
}

impl HoneypotHandler {
  fn new(shared: Arc<Shared>, peer_addr: Option<SocketAddr>) -> Self {
    let max_rate = shared.cfg.ssh.max_input_rate_per_second;
    Self {
      shared,
      peer_addr,
      io: Arc::new(Mutex::new(SessionIo::new(max_rate))),
    }
  }
}

#[async_trait]
impl russh::server::Handler for HoneypotHandler {
  type Error = russh::Error;

  async fn auth_password(&mut self, user: &str, password: &str) -> std::result::Result<Auth, Self::Error> {
    let users: Vec<(String, String)> = self
      .shared
      .cfg
      .users
      .iter()
      .map(|u| (u.username.clone(), u.password.clone()))
      .collect();
    if check_password(&users, user, password) {
      let mut io = self.io.lock().await;
      io.username = user.to_string();
      Ok(Auth::Accept)
    } else {
      Ok(Auth::Reject {
        proceed_with_methods: Some(MethodSet::PASSWORD),
      })
    }
  }

  async fn auth_publickey(
    &mut self,
    _user: &str,
    _public_key: &russh::keys::key::PublicKey,
  ) -> std::result::Result<Auth, Self::Error> {
    Ok(Auth::Reject {
      proceed_with_methods: Some(MethodSet::PASSWORD),
    })
  }

  async fn channel_open_session(
    &mut self,
    channel: Channel<Msg>,
    _session: &mut Session,
  ) -> std::result::Result<bool, Self::Error> {
    let mut io = self.io.lock().await;
    io.channel = Some(channel.id());
    Ok(true)
  }

  async fn pty_request(
    &mut self,
    _channel: ChannelId,
    term: &str,
    col_width: u32,
    row_height: u32,
    _pix_width: u32,
    _pix_height: u32,
    _modes: &[(russh::Pty, u32)],
    _session: &mut Session,
  ) -> std::result::Result<(), Self::Error> {
    let mut io = self.io.lock().await;
    if let Some(state) = io.state.as_mut() {
      state.terminal = term.to_string();
      state.term_width = if col_width == 0 { 80 } else { col_width };
      state.term_height = if row_height == 0 { 24 } else { row_height };
    }
    Ok(())
  }

  async fn shell_request(
    &mut self,
    channel: ChannelId,
    session: &mut Session,
  ) -> std::result::Result<(), Self::Error> {
    let mut io = self.io.lock().await;
    let username = if io.username.is_empty() {
      "root".to_string()
    } else {
      io.username.clone()
    };
    let cwd = if username == "root" {
      "/root"
    } else {
      "/home/user"
    };
    let client_ip = self
      .peer_addr
      .map(|a| a.ip().to_string())
      .unwrap_or_else(|| "unknown".to_string());
    io.state = Some(SessionState::new_for_test(&username, &self.shared.cfg.ssh.hostname, cwd));
    io.logger = JsonlLogger::new(&self.shared.cfg.logging.dir, &io.session_id).ok();
    if let Some(state) = io.state.as_mut() {
      state.client_ip = client_ip;
      state.terminal = "xterm-256color".to_string();
      state.term_width = 80;
      state.term_height = 24;
    }
    if let Some(logger) = io.logger.as_ref() {
      let _ = logger.log(&Event::new(
        io.session_id.clone(),
        now_ms(),
        EventKind::Connected,
      ));
    }
    drop(io);

    let prompt = build_prompt(&username, &self.shared.cfg.ssh.hostname, cwd);
    session.data(channel, CryptoVec::from_slice(prompt.as_bytes()));
    Ok(())
  }

  async fn data(
    &mut self,
    channel: ChannelId,
    data: &[u8],
    session: &mut Session,
  ) -> std::result::Result<(), Self::Error> {
    let (to_send, commands) = {
      let mut io = self.io.lock().await;
      let out = io.editor.process_bytes(data);
      (out.to_send, out.commands)
    };

    if !to_send.is_empty() {
      session.data(channel, CryptoVec::from_slice(&to_send));
    }

    for cmd in commands {
      let (response, disconnect, prompt, log_ai) = {
        let mut io = self.io.lock().await;
        let username = io.username.clone();
        let hostname = self.shared.cfg.ssh.hostname.clone();
        let router = Router;
        let session_id = io.session_id.clone();
        if let Some(logger) = io.logger.as_ref() {
          let _ = logger.log(&Event::new(
            session_id.clone(),
            now_ms(),
            EventKind::Command { input: cmd.clone() },
          ));
        }
        let state = io.state.as_mut().unwrap();
        state.push_history(&cmd);
        let action = router.handle_command(state, &cmd);

        match action {
          Action::SendText(text) => {
            let prompt = build_prompt(&username, &hostname, state.cwd());
            (text, false, prompt, None)
          }
          Action::NoOutput => {
            let prompt = build_prompt(&username, &hostname, state.cwd());
            (String::new(), false, prompt, None)
          }
          Action::Disconnect => (String::new(), true, String::new(), None),
          Action::AiRequest {
            system_prompt,
            user_command,
          } => {
            let text = collect_ai_for_shell(&self.shared.cfg, system_prompt, user_command)
              .await
              .unwrap_or_else(|_| "bash: internal error: AI service unavailable\n".to_string());
            let prompt = build_prompt(&username, &hostname, state.cwd());
            let bytes = text.len();
            let truncated = false;
            (text, false, prompt, Some((bytes, truncated, session_id)))
          }
        }
      };

      if !response.is_empty() {
        session.data(channel, CryptoVec::from_slice(response.as_bytes()));
      }

      if let Some((bytes, truncated, session_id)) = log_ai {
        let io = self.io.lock().await;
        if let Some(logger) = io.logger.as_ref() {
          let _ = logger.log(&Event::new(
            session_id,
            now_ms(),
            EventKind::AiResponse { bytes, truncated },
          ));
        }
      }

      if disconnect {
        session.eof(channel);
        session.close(channel);
        return Ok(());
      }

      session.data(channel, CryptoVec::from_slice(prompt.as_bytes()));
    }

    Ok(())
  }

  async fn channel_close(
    &mut self,
    _channel: ChannelId,
    _session: &mut Session,
  ) -> std::result::Result<(), Self::Error> {
    let io = self.io.lock().await;
    if let Some(logger) = io.logger.as_ref() {
      let _ = logger.log(&Event::new(
        io.session_id.clone(),
        now_ms(),
        EventKind::Disconnected,
      ));
    }
    Ok(())
  }
}

pub async fn run_server(cfg: &Config) -> Result<()> {
  let host_key = crate::ssh::hostkey::ensure_host_key(&cfg.ssh.host_key_path)?;
  let server_cfg = make_russh_config(cfg, vec![host_key])?;
  let mut server = HoneypotServer::new(Arc::new(cfg.clone()));
  server
    .run_on_address(Arc::new(server_cfg), cfg.ssh.listen_addr.as_str())
    .await?;
  Ok(())
}

fn build_prompt(username: &str, hostname: &str, cwd: &str) -> String {
  let suffix = if username == "root" { '#' } else { '$' };
  format!("{username}@{hostname}:{cwd}{suffix} ")
}

async fn collect_ai_for_shell(cfg: &Config, system_prompt: String, user_command: String) -> Result<String> {
  let client = OpenAiClient::new(cfg.openai.base_url.clone(), cfg.openai.api_key.clone())?;
  let req = ChatCompletionsRequest {
    model: cfg.openai.model.clone(),
    messages: vec![
      ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
      },
      ChatMessage {
        role: "user".to_string(),
        content: user_command,
      },
    ],
    stream: true,
    temperature: Some(cfg.openai.temperature),
    max_tokens: Some(cfg.openai.max_tokens),
  };

  let stream = client.stream_chat(req).await?;
  let executor = CommandExecutor::new();
  let mut guard = OutputGuard::new(
    cfg.limits.max_output_length,
    cfg.limits.max_output_lines,
    cfg.limits.max_numbered_lines,
  );
  let outcome = executor.collect_ai_stream(&mut guard, stream).await?;
  Ok(outcome.output)
}

fn now_ms() -> u64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_millis() as u64)
    .unwrap_or_default()
}
