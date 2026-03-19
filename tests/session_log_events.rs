use lmssh::logging::event::{Event, EventKind};
use lmssh::logging::jsonl::JsonlLogger;

#[test]
fn logger_writes_connected_and_auth_events() {
  let dir = tempfile::tempdir().unwrap();
  let logger = JsonlLogger::new(dir.path(), "sess").unwrap();

  logger
    .log(&Event::new("sess", 1, EventKind::Connected))
    .unwrap();
  logger
    .log(&Event::new(
      "sess",
      2,
      EventKind::AuthSuccess {
        username: "root".to_string(),
      },
    ))
    .unwrap();
  logger
    .log(&Event::new(
      "sess",
      3,
      EventKind::AuthFailed {
        username: "nobody".to_string(),
      },
    ))
    .unwrap();

  let content = std::fs::read_to_string(logger.path()).unwrap();
  assert!(content.contains("\"type\":\"Connected\""));
  assert!(content.contains("\"type\":\"AuthSuccess\""));
  assert!(content.contains("\"type\":\"AuthFailed\""));
}
