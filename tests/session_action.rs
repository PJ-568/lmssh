use lmssh::session::Action;

#[test]
fn action_is_constructible() {
    let a = Action::Disconnect;
    match a {
        Action::Disconnect => {}
        _ => panic!("unexpected"),
    }
}
