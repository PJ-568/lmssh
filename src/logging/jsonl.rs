use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::logging::event::Event;

#[derive(Debug, Clone)]
pub struct JsonlLogger {
    path: PathBuf,
}

impl JsonlLogger {
    pub fn new(dir: impl AsRef<Path>, session_id: &str) -> Result<Self> {
        std::fs::create_dir_all(dir.as_ref())?;
        let path = dir.as_ref().join(format!("session_{session_id}.jsonl"));
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn log(&self, event: &Event) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        serde_json::to_writer(&mut file, event)?;
        file.write_all(b"\n")?;
        file.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::logging::event::{Event, EventKind};

    #[test]
    fn append_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let logger = JsonlLogger::new(dir.path(), "test").unwrap();

        logger
            .log(&Event::new(
                "test",
                1,
                EventKind::Command {
                    input: "pwd".to_string(),
                },
            ))
            .unwrap();
        logger
            .log(&Event::new("test", 2, EventKind::Disconnected))
            .unwrap();

        let content = std::fs::read_to_string(logger.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        let e1: Event = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(e1.session_id, "test");
        assert_eq!(e1.timestamp_ms, 1);

        let e2: Event = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(e2.kind, EventKind::Disconnected);
    }
}
