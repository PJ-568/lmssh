use crate::session::{Action, SessionState};
use crate::session::blacklist;

#[derive(Debug, Default)]
pub struct Router;

impl Router {
  pub fn handle_command(&self, session: &mut SessionState, cmd: &str) -> Action {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
      return Action::NoOutput;
    }

    let cmd0 = trimmed.split_whitespace().next().unwrap_or("");
    if blacklist::is_blacklisted(cmd0) {
      return Action::SendText(format!("bash: {cmd0}: command not found\n"));
    }

    match trimmed {
      "pwd" => Action::SendText(format!("{}\n", session.cwd)),
      "clear" => Action::SendText("\x1b[2J\x1b[H".to_string()),
      "exit" | "logout" => Action::Disconnect,
      "history" => {
        let mut out = String::new();
        for (idx, cmd) in session.history.iter().enumerate() {
          let i = idx + 1;
          out.push_str(&format!("{i:>5}  {cmd}\n"));
        }
        Action::SendText(out)
      }
      _ if trimmed == "cd" || trimmed.starts_with("cd ") => {
        let arg = trimmed.strip_prefix("cd").unwrap().trim();
        let target = if arg.is_empty() {
          // home
          if session.username == "root" {
            "/root".to_string()
          } else {
            format!("/home/{}", session.username)
          }
        } else if arg.starts_with('/') {
          normalize_path(arg)
        } else {
          let base = if session.cwd == "/" {
            format!("/{}", arg)
          } else {
            format!("{}/{}", session.cwd, arg)
          };
          normalize_path(&base)
        };

        if session.vfs.is_dir(&target) {
          session.cwd = target;
          Action::NoOutput
        } else {
          Action::SendText(format!(
            "bash: cd: {}: No such file or directory\n",
            target
          ))
        }
      }
      _ => Action::NoOutput,
    }
  }
}

fn normalize_path(path: &str) -> String {
  let raw = if path.is_empty() { "/" } else { path };
  let raw = if raw.starts_with('/') {
    raw.to_string()
  } else {
    format!("/{raw}")
  };

  let mut stack: Vec<&str> = vec![];
  for part in raw.split('/') {
    if part.is_empty() || part == "." {
      continue;
    }
    if part == ".." {
      stack.pop();
      continue;
    }
    stack.push(part);
  }

  if stack.is_empty() {
    return "/".to_string();
  }
  let mut out = String::new();
  for seg in stack {
    out.push('/');
    out.push_str(seg);
  }
  out
}
