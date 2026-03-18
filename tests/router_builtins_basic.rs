use lmssh::session::{Action, Router, SessionState};

fn mk_session() -> SessionState {
  SessionState::new_for_test("root", "debian", "/root")
}

#[test]
fn pwd_outputs_cwd_with_newline() {
  let mut s = mk_session();
  let r = Router::default();
  let act = r.handle_command(&mut s, "pwd");
  assert_eq!(act, Action::SendText("/root\n".to_string()));
}

#[test]
fn exit_disconnects() {
  let mut s = mk_session();
  let r = Router::default();
  assert_eq!(r.handle_command(&mut s, "exit"), Action::Disconnect);
  assert_eq!(r.handle_command(&mut s, "logout"), Action::Disconnect);
}
