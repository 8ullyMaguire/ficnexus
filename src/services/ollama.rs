//! Ollama client — local embeddings + small-model text generation.
//!
//! Ollama runs on localhost:11434 (already used by the QA triage worker).
//! The Roadmap Consensus Engine uses `/api/embeddings` (nomic-embed-text,
//! 768-d); comment moderation triage uses `/api/generate` (llama3.1:8b) for
//! cheap one-shot classification. Keeping this as a thin wrapper avoids
//! adding heavy Rust ML dependencies; the model is called over HTTP with the
//! shared reqwest client.

use serde::Deserialize;
use serde_json::json;

/// Error returned by the Ollama embeddings call. Callers treat this as
/// non-fatal (a failed embedding should not fail the suggestion submission —
/// the raw text is still stored, just unclustered).
#[derive(Debug, Clone)]
pub struct OllamaError(pub String);

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ollama: {}", self.0)
    }
}

impl std::error::Error for OllamaError {}

#[derive(Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    model: String,
    http: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String, http: reqwest::Client) -> Self {
        Self { base_url, model, http }
    }

    /// Embed a piece of text via Ollama's `/api/embeddings`.
    ///
    /// Returns the raw vector (dimension depends on the model — nomic-embed-text
    /// is 768-d). Errors are returned as `OllamaError` so callers can degrade
    /// gracefully.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, OllamaError> {
        let url = format!("{}/api/embeddings", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .json(&json!({ "model": self.model, "prompt": text }))
            .send()
            .await
            .map_err(|e| OllamaError(format!("request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(OllamaError(format!("HTTP {}", resp.status())));
        }

        let body: EmbeddingResponse = resp
            .json()
            .await
            .map_err(|e| OllamaError(format!("bad response: {e}")))?;

        if body.embedding.is_empty() {
            return Err(OllamaError("empty embedding".into()));
        }
        Ok(body.embedding)
    }

    /// Generate a completion for `prompt` via Ollama's `/api/generate`.
    ///
    /// Non-streaming one-shot call (used for tiny LLM applications like
    /// comment moderation triage). The `model` field set at construction is
    /// the EMBEDDING model — so the chat model is passed explicitly here
    /// via the request body. Errors are returned as `OllamaError` so
    /// callers can degrade gracefully.
    pub async fn generate(&self, prompt: &str, chat_model: &str) -> Result<String, OllamaError> {
        let url = format!("{}/api/generate", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .json(&json!({
                "model": chat_model,
                "prompt": prompt,
                "stream": false,
                // Keep the model resident between calls (avoids repeated cold
                // loads on low-memory hosts) and disable thinking-mode so
                // JSON tasks get a clean response (thinking models like
                // lfm2.5 / qwen3 emit <think> blocks otherwise).
                "keep_alive": "30m",
                "think": false,
            }))
            .send()
            .await
            .map_err(|e| OllamaError(format!("request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(OllamaError(format!("HTTP {}", resp.status())));
        }

        let body: GenerateResponse = resp
            .json()
            .await
            .map_err(|e| OllamaError(format!("bad response: {e}")))?;

        let mut out = body.response.trim().to_string();
        // Strip any thinking blocks that slipped through (some models emit
        // <think>…</think> regardless of the option). Keep the text AFTER the
        // block — that's the actual answer.
        if let Some(start) = out.find("<think>") {
            let after_think = out[start + "<think>".len()..].to_string();
            if let Some(end_rel) = after_think.find("</think>") {
                let end = start + "<think>".len() + end_rel + "</think>".len();
                let post = out[end..].trim();
                if !post.is_empty() {
                    out = post.to_string();
                }
            }
        }
        if out.trim().is_empty() {
            return Err(OllamaError("empty response".into()));
        }
        Ok(out)
    }

    /// Generate a completion for `prompt` via Ollama's `/api/generate` with
    /// `"format": "json"` so the model is constrained to emit a JSON object.
    ///
    /// Used by Ask the Archive (natural-language → search filters). The
    /// caller still validates the reply defensively — `format: json` is a
    /// strong nudge, not a guarantee. `chat_model` is passed in the body
    /// (the client's `model` field is the embedding model). Errors surface
    /// as `OllamaError` like [`Self::generate`].
    pub async fn generate_json(&self, prompt: &str, chat_model: &str) -> Result<String, OllamaError> {
        let url = format!("{}/api/generate", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .json(&json!({
                "model": chat_model,
                "prompt": prompt,
                "stream": false,
                "format": "json",
                "keep_alive": "30m",
                "think": false,
            }))
            .send()
            .await
            .map_err(|e| OllamaError(format!("request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(OllamaError(format!("HTTP {}", resp.status())));
        }

        let body: GenerateResponse = resp
            .json()
            .await
            .map_err(|e| OllamaError(format!("bad response: {e}")))?;

        let mut out = body.response.trim().to_string();
        // Strip any thinking blocks that slipped through (some models emit
        // <think>…</think> regardless of the option). Keep the text AFTER the
        // block — that's the actual answer.
        if let Some(start) = out.find("<think>") {
            let after_think = out[start + "<think>".len()..].to_string();
            if let Some(end_rel) = after_think.find("</think>") {
                let end = start + "<think>".len() + end_rel + "</think>".len();
                let post = out[end..].trim();
                if !post.is_empty() {
                    out = post.to_string();
                }
            }
        }
        if out.trim().is_empty() {
            return Err(OllamaError("empty response".into()));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn embed_error_when_ollama_down() {
        // Port 1 is not listening — connection refused surfaces as OllamaError.
        let client = OllamaClient::new(
            "http://127.0.0.1:1".into(),
            "nomic-embed-text".into(),
            reqwest::Client::new(),
        );
        let err = client.embed("test").await.expect_err("should fail");
        assert!(err.to_string().starts_with("ollama:"));
    }

    #[tokio::test]
    async fn embed_returns_ollama_error_on_http_error() {
        // A closed mock server: we bind a listener, grab its port, drop it,
        // then connect — the request fails fast with a transport error.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().unwrap().port();
        drop(listener); // now nothing is listening

        let client = OllamaClient::new(
            format!("http://127.0.0.1:{port}"),
            "nomic-embed-text".into(),
            reqwest::Client::new(),
        );
        let err = client.embed("test").await.expect_err("should fail");
        assert!(err.to_string().starts_with("ollama:"));
    }

    #[tokio::test]
    async fn generate_error_when_ollama_down() {
        // Port 1 is not listening — connection refused surfaces as OllamaError.
        let client = OllamaClient::new(
            "http://127.0.0.1:1".into(),
            "nomic-embed-text".into(),
            reqwest::Client::new(),
        );
        let err = client.generate("hi", "llama3.1:8b").await.expect_err("should fail");
        assert!(err.to_string().starts_with("ollama:"));
    }

    #[tokio::test]
    async fn generate_returns_ollama_error_on_http_error() {
        // A listener that always answers 500 — the HTTP error path.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(mut s) = stream {
                    let _ = read_request(&mut s);
                    let _ = std::io::Write::write_all(
                        &mut s,
                        b"HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
                    );
                }
            }
        });

        let client = OllamaClient::new(
            format!("http://127.0.0.1:{port}"),
            "nomic-embed-text".into(),
            reqwest::Client::new(),
        );
        let err = client.generate("hi", "llama3.1:8b").await.expect_err("should fail");
        assert!(err.to_string().starts_with("ollama:"), "err: {err}");
        assert!(err.to_string().contains("500"), "err: {err}");
    }

    #[tokio::test]
    async fn generate_parses_response_field() {
        // A listener that answers a valid /api/generate payload.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().unwrap().port();
        let body = br#"{"model":"llama3.1:8b","response":"toxic | slur | 0.9","done":true}"#;
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(mut s) = stream {
                    let _ = read_request(&mut s);
                    let _ = std::io::Write::write_all(
                        &mut s,
                        format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                            body.len()
                        )
                        .as_bytes(),
                    );
                    let _ = std::io::Write::write_all(&mut s, body);
                }
            }
        });

        let client = OllamaClient::new(
            format!("http://127.0.0.1:{port}"),
            "nomic-embed-text".into(),
            reqwest::Client::new(),
        );
        let out = client.generate("classify", "llama3.1:8b").await.expect("should succeed");
        assert_eq!(out, "toxic | slur | 0.9");
    }

    /// Drain one HTTP request (headers + body) from the socket so the mock
    /// server can respond without the client seeing a mid-send reset.
    fn read_request(s: &mut std::net::TcpStream) -> std::io::Result<()> {
        use std::io::{BufRead, Read};
        let mut reader = std::io::BufReader::new(s.try_clone()?);
        let mut head = Vec::new();
        loop {
            let n = reader.read_until(b'\n', &mut head)?;
            if n == 0 {
                return Ok(());
            }
            if head.ends_with(b"\r\n\r\n") || head.ends_with(b"\n\n") {
                break;
            }
        }
        // Read the body if Content-Length says there is one.
        let head_str = String::from_utf8_lossy(&head);
        let mut len = 0usize;
        for line in head_str.lines() {
            let lower = line.to_ascii_lowercase();
            if let Some(v) = lower.strip_prefix("content-length:") {
                len = v.trim().parse().unwrap_or(0);
            }
        }
        if len > 0 {
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf)?;
        }
        Ok(())
    }
}
