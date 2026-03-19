use lmssh::session::{Action, Router, SessionState};

#[test]
fn blacklisted_command_returns_command_not_found() {
    let mut s = SessionState::new_for_test("root", "debian", "/root");
    let r = Router;
    let act = r.handle_command(&mut s, "vim");
    assert_eq!(
        act,
        Action::SendText("bash: vim: command not found\n".to_string())
    );
}
