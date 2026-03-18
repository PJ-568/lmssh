use std::future::Future;

use futures_util::StreamExt;
use tokio::sync::Semaphore;

use crate::session::OutputGuard;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChunkOutcome {
  pub output: String,
  pub truncated: bool,
}

#[derive(Debug)]
pub struct CommandExecutor {
  gate: Semaphore,
}

impl Default for CommandExecutor {
  fn default() -> Self {
    Self::new()
  }
}

impl CommandExecutor {
  pub fn new() -> Self {
    Self {
      gate: Semaphore::new(1),
    }
  }

  pub async fn run<F, Fut>(&self, f: F)
  where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
  {
    let _permit = self.gate.acquire().await.unwrap();
    f().await;
  }

  pub fn collect_ai_chunks(
    &self,
    guard: &mut OutputGuard,
    chunks: Vec<String>,
  ) -> AiChunkOutcome {
    let mut output = String::new();
    let mut truncated = false;

    for chunk in chunks {
      if chunk == "<NO_OUTPUT>" {
        return AiChunkOutcome {
          output: String::new(),
          truncated: false,
        };
      }

      let should_stop = guard.push(&chunk);
      output.push_str(&chunk);
      if should_stop {
        truncated = true;
        break;
      }
    }

    AiChunkOutcome { output, truncated }
  }

  pub async fn collect_ai_stream<S>(&self, guard: &mut OutputGuard, mut stream: S) -> Result<AiChunkOutcome>
  where
    S: futures_util::Stream<Item = Result<String>> + Unpin,
  {
    let mut output = String::new();
    let mut truncated = false;

    while let Some(chunk) = stream.next().await {
      let chunk = chunk?;
      if chunk == "<NO_OUTPUT>" {
        return Ok(AiChunkOutcome {
          output: String::new(),
          truncated: false,
        });
      }

      let should_stop = guard.push(&chunk);
      output.push_str(&chunk);
      if should_stop {
        truncated = true;
        break;
      }
    }

    Ok(AiChunkOutcome { output, truncated })
  }
}
