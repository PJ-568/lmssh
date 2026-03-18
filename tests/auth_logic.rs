use lmssh::ssh::server::check_password;

#[test]
fn password_auth_accepts_config_users() {
  let users = vec![("root".to_string(), "password".to_string())];
  assert!(check_password(&users, "root", "password"));
  assert!(!check_password(&users, "root", "wrong"));
  assert!(!check_password(&users, "nobody", "password"));
}
