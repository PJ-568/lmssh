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
      "exit" | "logout" => Action::Disconnect,
      _ => Action::NoOutput,
    }
  }
}
