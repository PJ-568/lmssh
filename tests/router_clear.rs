use lmssh::session::{Action, Router, SessionState};

#[test]
fn clear_sends_ansi_clear_screen() {
  let mut s = SessionState::new_for_test("root", "debian", "/root");
  let r = Router::default();
  assert_eq!(
    r.handle_command(&mut s, "clear"),
    Action::SendText("\x1b[2J\x1b[H".to_string())
  );
}
