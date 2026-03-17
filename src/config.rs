use serde::Deserialize;

use crate::error::Result;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
  pub ssh: SshConfig,
  pub openai: OpenAiConfig,
}

impl Config {
  pub fn load_from_str(toml_str: &str) -> Result<Self> {
    Ok(toml::from_str(toml_str)?)
  }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SshConfig {
  pub listen_addr: String,
  pub hostname: String,
  pub banner: String,
  pub host_key_path: String,
  pub session_timeout_seconds: u64,
  pub max_input_rate_per_second: u32,
}

impl Default for SshConfig {
  fn default() -> Self {
    Self {
      listen_addr: "127.0.0.1:2222".to_string(),
      hostname: "debian".to_string(),
      banner: "SSH-2.0-OpenSSH_9.2p1 Debian-2+deb12u3".to_string(),
      host_key_path: "data/host_ed25519".to_string(),
      session_timeout_seconds: 120,
      max_input_rate_per_second: 32,
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OpenAiConfig {
  pub api_key: String,
  pub base_url: String,
  pub model: String,
  pub max_tokens: u32,
  pub temperature: f32,
}

impl Default for OpenAiConfig {
  fn default() -> Self {
    Self {
      api_key: String::new(),
      base_url: "https://api.openai.com".to_string(),
      model: "gpt-4o-mini".to_string(),
      max_tokens: 512,
      temperature: 0.6,
    }
  }
}
