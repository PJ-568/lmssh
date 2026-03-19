use std::pin::Pin;

use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;

use crate::ai::types::{ChatCompletionsChunk, ChatCompletionsRequest};
use crate::error::Result;

pub struct OpenAiClient {
  base_url: String,
  api_key: String,
  http: reqwest::Client,
}

impl OpenAiClient {
  pub fn new(base_url: String, api_key: String) -> Result<Self> {
    Ok(Self {
      base_url,
      api_key,
      http: reqwest::Client::new(),
    })
  }

  pub async fn stream_chat(
    &self,
    req: ChatCompletionsRequest,
  ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
    type DynStream = Pin<Box<dyn Stream<Item = Result<String>> + Send>>;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
      AUTHORIZATION,
      HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
    );

    let url = chat_completions_url(&self.base_url);
    let resp = self
      .http
      .post(url)
      .headers(headers)
      .json(&ChatCompletionsRequest {
        stream: true,
        ..req
      })
      .send()
      .await?;

    if !resp.status().is_success() {
      // 不泄露 body（可能包含敏感信息），对齐 reference 的通用错误文案。
      let s: DynStream = Box::pin(tokio_stream::once(Err(
        std::io::Error::other("AI service unavailable").into(),
      )));
      return Ok(s);
    }

    // NOTE: 这里用最小 SSE 行解析（避免 eventsource-stream 的 trait 兼容问题）。
    // OpenAI SSE: 每行形如 `data: {...}` 或 `data: [DONE]`，事件之间以空行分隔。
    let mut byte_stream = resp.bytes_stream();

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<String>>(32);
    tokio::spawn(async move {
      let mut buf: Vec<u8> = Vec::new();
      while let Some(item) = byte_stream.next().await {
        match item {
          Ok(bytes) => {
            buf.extend_from_slice(&bytes);
            // 逐行处理（\n 分隔；SSE 的 \r\n 也兼容处理）。
            while let Some(pos) = buf.iter().position(|b| *b == b'\n') {
              let mut line = buf.drain(..=pos).collect::<Vec<u8>>();
              if line.ends_with(b"\n") {
                line.pop();
              }
              if line.ends_with(b"\r") {
                line.pop();
              }

              if line.is_empty() {
                continue;
              }

              let Some(rest) = line.strip_prefix(b"data: ") else {
                continue;
              };

              if rest == b"[DONE]" {
                return;
              }

              let Ok(s) = std::str::from_utf8(rest) else {
                continue;
              };

              if let Ok(chunk) = serde_json::from_str::<ChatCompletionsChunk>(s) {
                for c in chunk.choices {
                  if let Some(content) = c.delta.content
                    && tx.send(Ok(content)).await.is_err()
                  {
                    return;
                  }
                }
              }
            }
          }
          Err(_) => {
            let _ = tx
              .send(Err(
                std::io::Error::other("AI service unavailable").into(),
              ))
              .await;
            return;
          }
        }
      }
    });

    let s: DynStream = Box::pin(ReceiverStream::new(rx));
    Ok(s)
  }
}

fn chat_completions_url(base_url: &str) -> String {
  format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_sse_data_line() {
    let mut buf = b"data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n".to_vec();
    let pos = buf.iter().position(|b| *b == b'\n').unwrap();
    let mut line = buf.drain(..=pos).collect::<Vec<u8>>();
    if line.ends_with(b"\n") {
      line.pop();
    }
    let rest = line.strip_prefix(b"data: ").unwrap();
    let s = std::str::from_utf8(rest).unwrap();
    let chunk: ChatCompletionsChunk = serde_json::from_str(s).unwrap();
    assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("hi"));
  }

  #[test]
  fn chat_completion_url_uses_provider_base_path() {
    assert_eq!(
      chat_completions_url("https://api.pj568.eu.org/proxy/wasteest-source"),
      "https://api.pj568.eu.org/proxy/wasteest-source/chat/completions"
    );
  }
}
