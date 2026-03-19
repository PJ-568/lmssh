use lmssh::ai::types::{ChatCompletionsRequest, ChatMessage};

#[test]
fn chat_request_can_be_built_for_streaming() {
    let req = ChatCompletionsRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "sys".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "pwd".to_string(),
            },
        ],
        stream: true,
        temperature: Some(0.6),
        max_tokens: Some(512),
    };

    assert!(req.stream);
    assert_eq!(req.messages.len(), 2);
}
