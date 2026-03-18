use std::path::Path;

use rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};

use crate::error::{LmsshError, Result};

pub fn ensure_host_key(path: impl AsRef<Path>) -> Result<russh::keys::key::KeyPair> {
  let path = path.as_ref();

  if path.exists() {
    let key = PrivateKey::read_openssh_file(path)?;
    if key.algorithm() != Algorithm::Ed25519 {
      return Err(LmsshError::UnexpectedHostKeyAlgorithm {
        got: key.algorithm(),
      });
    }
    return Ok(russh::keys::load_secret_key(path, None)?);
  }

  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }

  let key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)?;
  key.write_openssh_file(path, LineEnding::LF)?;

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)?;
  }

  Ok(russh::keys::load_secret_key(path, None)?)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ensure_host_key_creates_and_reuses() {
    let dir = tempfile::tempdir().unwrap();
    let key_path = dir.path().join("host_ed25519");

    assert!(!key_path.exists());
    ensure_host_key(&key_path).unwrap();
    assert!(key_path.exists());

    let first = std::fs::read(&key_path).unwrap();
    ensure_host_key(&key_path).unwrap();
    let second = std::fs::read(&key_path).unwrap();
    assert_eq!(first, second);
  }
}
