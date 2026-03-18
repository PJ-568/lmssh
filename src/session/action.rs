#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
  SendText(String),
  NoOutput,
  Disconnect,
  AiRequest {
    system_prompt: String,
    user_command: String,
  },
}
