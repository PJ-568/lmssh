use lmssh::session::OutputGuard;

#[test]
fn guard_truncates_when_too_many_lines() {
  let mut g = OutputGuard::new(10_000, 3, 15);
  assert!(!g.push("a\n"));
  assert!(!g.push("b\n"));
  assert!(g.push("c\n")); // reaching limit should stop
}

#[test]
fn guard_truncates_on_numbered_lines() {
  let mut g = OutputGuard::new(10_000, 200, 2);
  assert!(!g.push("1. a\n"));
  assert!(g.push("2. b\n"));
}
