use lmssh::session::{Action, Router, SessionState};

#[test]
fn other_command_produces_ai_request_and_applies_vfs_side_effects() {
  let mut s = SessionState::new_for_test("root", "debian", "/root");
  let r = Router::default();
  let act = r.handle_command(&mut s, "echo hi > /tmp/out.txt");

  // side effect
  assert_eq!(s.vfs().read_file("/tmp/out.txt"), Some("hi\n"));

  // action
  let Action::AiRequest {
    system_prompt,
    user_command,
  } = act
  else {
    panic!("expected AiRequest");
  };
  assert!(system_prompt.contains("禁止markdown"));
  assert_eq!(user_command, "echo hi > /tmp/out.txt");
}
