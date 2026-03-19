use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum EventKind {
    Connected,
    AuthSuccess { username: String },
    AuthFailed { username: String },
    Command { input: String },
    AiResponse { bytes: usize, truncated: bool },
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub session_id: String,
    pub timestamp_ms: u64,

    #[serde(flatten)]
    pub kind: EventKind,
}

impl Event {
    pub fn new(session_id: impl Into<String>, timestamp_ms: u64, kind: EventKind) -> Self {
        Self {
            session_id: session_id.into(),
            timestamp_ms,
            kind,
        }
    }
}
