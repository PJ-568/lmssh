use futures_util::stream;
use lmssh::session::{AiChunkOutcome, CommandExecutor, OutputGuard};

#[tokio::test]
async fn executor_collects_stream_and_applies_guard() {
  let executor = CommandExecutor::new();
  let mut guard = OutputGuard::new(10_000, 2, 15);

  let chunks = stream::iter(vec![
    Ok::<_, lmssh::error::LmsshError>("line-1\n".to_string()),
    Ok::<_, lmssh::error::LmsshError>("line-2\n".to_string()),
    Ok::<_, lmssh::error::LmsshError>("line-3\n".to_string()),
  ]);

  let outcome = executor.collect_ai_stream(&mut guard, chunks).await.unwrap();
  assert_eq!(
    outcome,
    AiChunkOutcome {
      output: "line-1\nline-2\n".to_string(),
      truncated: true,
    }
  );
}
