use lmssh::session::{Action, Router, SessionState};

#[test]
fn cd_to_existing_dir_changes_cwd_and_no_output() {
    let mut s = SessionState::new_for_test("root", "debian", "/root");
    let r = Router;
    // seed 里有 /tmp
    let act = r.handle_command(&mut s, "cd /tmp");
    assert_eq!(act, Action::NoOutput);
    assert_eq!(s.cwd(), "/tmp");
}

#[test]
fn cd_to_missing_dir_prints_error() {
    let mut s = SessionState::new_for_test("root", "debian", "/root");
    let r = Router;
    let act = r.handle_command(&mut s, "cd /no_such_dir");
    assert_eq!(
        act,
        Action::SendText("bash: cd: /no_such_dir: No such file or directory\n".to_string())
    );
    assert_eq!(s.cwd(), "/root");
}
