use lmssh::session::{AiChunkOutcome, CommandExecutor, OutputGuard};

#[test]
fn executor_truncates_stream_when_guard_hits_limit() {
    let executor = CommandExecutor::new();
    let mut guard = OutputGuard::new(10_000, 2, 15);

    let outcome = executor.collect_ai_chunks(
        &mut guard,
        vec![
            "line-1\n".to_string(),
            "line-2\n".to_string(),
            "line-3\n".to_string(),
        ],
    );

    assert!(outcome.truncated);
    assert_eq!(outcome.output, "line-1\nline-2\n");
}

#[test]
fn executor_suppresses_no_output_marker() {
    let executor = CommandExecutor::new();
    let mut guard = OutputGuard::new(10_000, 200, 15);

    let outcome = executor.collect_ai_chunks(&mut guard, vec!["<NO_OUTPUT>".to_string()]);

    assert_eq!(
        outcome,
        AiChunkOutcome {
            output: String::new(),
            truncated: false,
        }
    );
}
