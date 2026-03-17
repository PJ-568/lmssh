use thiserror::Error;

#[derive(Debug, Error)]
pub enum LmsshError {
  #[error("failed to deserialize config TOML: {0}")]
  ConfigToml(#[from] toml::de::Error),

  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, LmsshError>;
