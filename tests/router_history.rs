use lmssh::session::{Action, Router, SessionState};

#[test]
fn history_formats_like_bash() {
  let mut s = SessionState::new_for_test("root", "debian", "/root");
  s.push_history("pwd");
  s.push_history("ls -la");
  let r = Router::default();

  let act = r.handle_command(&mut s, "history");
  let Action::SendText(text) = act else {
    panic!("expected SendText");
  };

  assert_eq!(text, "    1  pwd\n    2  ls -la\n");
}
