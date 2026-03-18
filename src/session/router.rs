use crate::session::{Action, SessionState};

#[derive(Debug, Default)]
pub struct Router;

impl Router {
  pub fn handle_command(&self, session: &mut SessionState, cmd: &str) -> Action {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
      return Action::NoOutput;
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
      _ => Action::NoOutput,
    }
  }
}
