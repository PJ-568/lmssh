use thiserror::Error;

use ssh_key::Algorithm;

#[derive(Debug, Error)]
pub enum LmsshError {
  #[error("failed to deserialize config TOML: {0}")]
  ConfigToml(#[from] toml::de::Error),

  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("russh-keys error: {0}")]
  RusshKeys(#[from] russh::keys::Error),

  #[error("ssh key error: {0}")]
  SshKey(#[from] ssh_key::Error),

  #[error("unexpected host key algorithm: {got:?}")]
  UnexpectedHostKeyAlgorithm { got: Algorithm },
}

pub type Result<T> = std::result::Result<T, LmsshError>;
