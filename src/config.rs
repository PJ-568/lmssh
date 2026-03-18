use serde::Deserialize;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

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

  pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
    let toml_str = std::fs::read_to_string(path)?;
    Self::load_from_str(&toml_str)
  }
}

pub fn default_config_path() -> PathBuf {
  default_config_path_from_env(|k| std::env::var_os(k))
}

fn default_config_path_from_env<F>(mut get_env: F) -> PathBuf
where
  F: FnMut(&str) -> Option<OsString>,
{
  if let Some(xdg) = get_env("XDG_CONFIG_HOME") {
    return PathBuf::from(xdg).join("lmssh").join("config.toml");
  }

  if let Some(home) = get_env("HOME") {
    return PathBuf::from(home)
      .join(".config")
      .join("lmssh")
      .join("config.toml");
  }

  // 极端情况下环境变量缺失，退化为当前目录。
  PathBuf::from("config.toml")
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

#[cfg(test)]
mod default_path_tests {
  use super::*;

  #[test]
  fn default_config_path_prefers_xdg() {
    let p = default_config_path_from_env(|k| match k {
      "XDG_CONFIG_HOME" => Some("/tmp/xdg".into()),
      "HOME" => Some("/home/test".into()),
      _ => None,
    });
    assert_eq!(p, PathBuf::from("/tmp/xdg/lmssh/config.toml"));
  }

  #[test]
  fn default_config_path_falls_back_to_home() {
    let p = default_config_path_from_env(|k| match k {
      "XDG_CONFIG_HOME" => None,
      "HOME" => Some("/home/test".into()),
      _ => None,
    });
    assert_eq!(p, PathBuf::from("/home/test/.config/lmssh/config.toml"));
  }

  #[test]
  fn default_config_path_falls_back_to_cwd() {
    let p = default_config_path_from_env(|_| None);
    assert_eq!(p, PathBuf::from("config.toml"));
  }
}
