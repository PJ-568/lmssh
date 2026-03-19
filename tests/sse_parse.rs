use lmssh::ai::types::ChatCompletionsChunk;

#[test]
fn parse_minimal_sse_data_json() {
    let data = r#"{"choices":[{"delta":{"content":"hi"},"finish_reason":null}]}"#;
    let chunk: ChatCompletionsChunk = serde_json::from_str(data).unwrap();
    assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("hi"));
}
