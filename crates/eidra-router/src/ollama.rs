use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tracing::{debug, warn};

use crate::error::RouterError;

/// Router for forwarding requests to a local Ollama instance.
///
/// Converts OpenAI-format chat completion requests to Ollama's API format
/// and forwards them to the configured endpoint.
pub struct OllamaRouter {
    /// Ollama API endpoint (e.g., "http://localhost:11434").
    endpoint: String,

    /// Default model to use when no explicit mapping exists.
    model: String,

    /// Optional mappings from upstream model names to local models.
    model_mapping: HashMap<String, String>,
}

impl OllamaRouter {
    /// Create a new OllamaRouter with the given endpoint and model.
    pub fn new(endpoint: &str, model: &str) -> Self {
        Self {
            endpoint: normalize_endpoint(endpoint),
            model: model.to_string(),
            model_mapping: HashMap::new(),
        }
    }

    /// Create a router with explicit model mappings.
    pub fn with_model_mapping(
        endpoint: &str,
        default_model: &str,
        model_mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            endpoint: normalize_endpoint(endpoint),
            model: default_model.to_string(),
            model_mapping,
        }
    }

    /// Get the configured endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get the configured model.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Convert an OpenAI-format request body to Ollama format and send it.
    ///
    /// Returns an OpenAI-compatible chat completion response so cloud-first
    /// clients can keep working when sensitive traffic is routed locally.
    ///
    /// OpenAI request format:
    /// ```json
    /// {"model":"gpt-4","messages":[{"role":"user","content":"..."}]}
    /// ```
    ///
    /// Ollama format:
    /// ```json
    /// {"model":"qwen2.5:latest","messages":[{"role":"user","content":"..."}],"stream":false}
    /// ```
    ///
    /// The upstream model field is mapped to a configured local model.
    /// Streaming is disabled for simplicity in v1.
    pub async fn route(&self, openai_body: &str) -> Result<String, RouterError> {
        let requested_model = self.extract_requested_model(openai_body)?;
        let response_model = requested_model.unwrap_or_else(|| self.model.clone());
        let ollama_body = self.convert_openai_to_ollama(openai_body)?;

        let uri = format!("{}/api/chat", self.endpoint);
        debug!("Routing to Ollama: {}", uri);

        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(ollama_body)))
            .map_err(|e| RouterError::Custom(format!("Failed to build request: {}", e)))?;

        let client = Client::builder(TokioExecutor::new()).build_http();

        let resp = client.request(req).await.map_err(|e| {
            warn!("Ollama request failed: {}", e);
            RouterError::OllamaUnavailable {
                endpoint: self.endpoint.clone(),
            }
        })?;

        let status = resp.status();
        let body_bytes = resp
            .into_body()
            .collect()
            .await
            .map_err(|e| RouterError::UpstreamError(format!("Failed to read response: {}", e)))?
            .to_bytes();

        let body_str = String::from_utf8_lossy(&body_bytes).to_string();

        if !status.is_success() {
            return Err(RouterError::UpstreamError(format!(
                "Ollama returned status {}: {}",
                status, body_str
            )));
        }

        self.convert_ollama_to_openai(&body_str, &response_model)
    }

    /// Check if the Ollama instance is available by hitting the root endpoint.
    pub async fn health_check(&self) -> bool {
        let uri = self.endpoint.clone();

        let req = match Request::builder()
            .method("GET")
            .uri(&uri)
            .body(Full::new(Bytes::new()))
        {
            Ok(r) => r,
            Err(_) => return false,
        };

        let client = Client::builder(TokioExecutor::new()).build_http();

        match client.request(req).await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Convert an OpenAI-format chat completion body to Ollama format.
    ///
    /// - Overrides the "model" field with the configured Ollama model.
    /// - Sets "stream" to false.
    /// - Preserves the "messages" array and other compatible fields.
    fn convert_openai_to_ollama(&self, openai_body: &str) -> Result<String, RouterError> {
        let target_model =
            self.resolve_target_model(self.extract_requested_model(openai_body)?.as_deref());
        self.convert_openai_to_ollama_with_model(openai_body, target_model)
    }

    fn convert_openai_to_ollama_with_model(
        &self,
        openai_body: &str,
        target_model: &str,
    ) -> Result<String, RouterError> {
        let mut body: serde_json::Value = serde_json::from_str(openai_body)
            .map_err(|e| RouterError::FormatConversion(format!("Invalid JSON: {}", e)))?;

        let obj = body.as_object_mut().ok_or_else(|| {
            RouterError::FormatConversion("Request body must be a JSON object".to_string())
        })?;

        if !obj
            .get("messages")
            .is_some_and(|messages| messages.is_array())
        {
            return Err(RouterError::FormatConversion(
                "Request body must include a messages array for local chat routing".to_string(),
            ));
        }

        // Override model with the resolved local model
        obj.insert(
            "model".to_string(),
            serde_json::Value::String(target_model.to_string()),
        );

        // Disable streaming for v1
        obj.insert("stream".to_string(), serde_json::Value::Bool(false));

        // Remove OpenAI-specific fields that Ollama doesn't understand
        let openai_only_fields = [
            "frequency_penalty",
            "presence_penalty",
            "logprobs",
            "top_logprobs",
            "n",
            "response_format",
            "tools",
            "tool_choice",
            "user",
            "logit_bias",
        ];
        for field in &openai_only_fields {
            obj.remove(*field);
        }

        serde_json::to_string(&body)
            .map_err(|e| RouterError::FormatConversion(format!("Failed to serialize: {}", e)))
    }

    fn extract_requested_model(&self, openai_body: &str) -> Result<Option<String>, RouterError> {
        let body: serde_json::Value = serde_json::from_str(openai_body)
            .map_err(|e| RouterError::FormatConversion(format!("Invalid JSON: {}", e)))?;

        let obj = body.as_object().ok_or_else(|| {
            RouterError::FormatConversion("Request body must be a JSON object".to_string())
        })?;

        Ok(obj
            .get("model")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned))
    }

    fn resolve_target_model<'a>(&'a self, requested_model: Option<&str>) -> &'a str {
        requested_model
            .and_then(|model| self.model_mapping.get(model))
            .or_else(|| self.model_mapping.get("default"))
            .map(String::as_str)
            .unwrap_or(&self.model)
    }

    fn convert_ollama_to_openai(
        &self,
        ollama_body: &str,
        response_model: &str,
    ) -> Result<String, RouterError> {
        let ollama: serde_json::Value = serde_json::from_str(ollama_body)?;

        let message = ollama
            .get("message")
            .cloned()
            .or_else(|| {
                ollama.get("response").and_then(|value| {
                    value.as_str().map(|content| {
                        serde_json::json!({
                            "role": "assistant",
                            "content": content,
                        })
                    })
                })
            })
            .ok_or_else(|| {
                RouterError::FormatConversion(
                    "Ollama response did not include a chat message".to_string(),
                )
            })?;

        let finish_reason = ollama
            .get("done_reason")
            .and_then(|value| value.as_str())
            .unwrap_or("stop");

        let prompt_tokens = ollama
            .get("prompt_eval_count")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let completion_tokens = ollama
            .get("eval_count")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let created = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        let openai = serde_json::json!({
            "id": format!("chatcmpl-eidra-{}", created),
            "object": "chat.completion",
            "created": created,
            "model": response_model,
            "choices": [
                {
                    "index": 0,
                    "message": message,
                    "finish_reason": finish_reason,
                }
            ],
            "usage": {
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "total_tokens": prompt_tokens + completion_tokens,
            }
        });

        serde_json::to_string(&openai)
            .map_err(|e| RouterError::FormatConversion(format!("Failed to serialize: {}", e)))
    }
}

impl Default for OllamaRouter {
    fn default() -> Self {
        Self::new("http://localhost:11434", "qwen2.5:latest")
    }
}

fn normalize_endpoint(endpoint: &str) -> String {
    endpoint.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_defaults() {
        let router = OllamaRouter::default();
        assert_eq!(router.endpoint(), "http://localhost:11434");
        assert_eq!(router.model(), "qwen2.5:latest");
    }

    #[test]
    fn test_new_with_custom_values() {
        let router = OllamaRouter::new("http://192.168.1.100:11434/", "llama3:8b");
        assert_eq!(router.endpoint(), "http://192.168.1.100:11434");
        assert_eq!(router.model(), "llama3:8b");
    }

    #[test]
    fn test_convert_openai_to_ollama_basic() {
        let router = OllamaRouter::new("http://localhost:11434", "qwen2.5:latest");
        let openai = r#"{"model":"gpt-4","messages":[{"role":"user","content":"Hello"}]}"#;

        let result = router.convert_openai_to_ollama(openai).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["model"], "qwen2.5:latest");
        assert_eq!(parsed["stream"], false);
        assert_eq!(parsed["messages"][0]["role"], "user");
        assert_eq!(parsed["messages"][0]["content"], "Hello");
    }

    #[test]
    fn test_convert_strips_openai_specific_fields() {
        let router = OllamaRouter::new("http://localhost:11434", "qwen2.5:latest");
        let openai = r#"{
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hi"}],
            "frequency_penalty": 0.5,
            "presence_penalty": 0.3,
            "logprobs": true,
            "n": 2,
            "user": "test-user"
        }"#;

        let result = router.convert_openai_to_ollama(openai).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed.get("frequency_penalty").is_none());
        assert!(parsed.get("presence_penalty").is_none());
        assert!(parsed.get("logprobs").is_none());
        assert!(parsed.get("n").is_none());
        assert!(parsed.get("user").is_none());
        // These should remain
        assert_eq!(parsed["model"], "qwen2.5:latest");
        assert!(parsed.get("messages").is_some());
    }

    #[test]
    fn test_convert_preserves_temperature_and_max_tokens() {
        let router = OllamaRouter::new("http://localhost:11434", "qwen2.5:latest");
        let openai = r#"{
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hi"}],
            "temperature": 0.7,
            "max_tokens": 100,
            "top_p": 0.9
        }"#;

        let result = router.convert_openai_to_ollama(openai).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["temperature"], 0.7);
        assert_eq!(parsed["max_tokens"], 100);
        assert_eq!(parsed["top_p"], 0.9);
    }

    #[test]
    fn test_convert_invalid_json() {
        let router = OllamaRouter::default();
        let result = router.convert_openai_to_ollama("not json");
        assert!(result.is_err());
        match result.unwrap_err() {
            RouterError::FormatConversion(_) => {}
            other => panic!("Expected FormatConversion, got: {:?}", other),
        }
    }

    #[test]
    fn test_convert_non_object_json() {
        let router = OllamaRouter::default();
        let result = router.convert_openai_to_ollama("[1, 2, 3]");
        assert!(result.is_err());
        match result.unwrap_err() {
            RouterError::FormatConversion(msg) => {
                assert!(msg.contains("JSON object"));
            }
            other => panic!("Expected FormatConversion, got: {:?}", other),
        }
    }

    #[test]
    fn test_convert_system_message_preserved() {
        let router = OllamaRouter::default();
        let openai = r#"{
            "model": "gpt-4",
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Hi"}
            ]
        }"#;

        let result = router.convert_openai_to_ollama(openai).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["messages"][0]["role"], "system");
        assert_eq!(parsed["messages"][0]["content"], "You are helpful.");
        assert_eq!(parsed["messages"][1]["role"], "user");
    }

    #[test]
    fn test_convert_requires_messages_array() {
        let router = OllamaRouter::default();
        let openai = r#"{"model":"gpt-4o-mini","input":"hello"}"#;

        let result = router.convert_openai_to_ollama(openai);
        assert!(result.is_err());
        match result.unwrap_err() {
            RouterError::FormatConversion(message) => {
                assert!(message.contains("messages array"));
            }
            other => panic!("Expected FormatConversion, got: {:?}", other),
        }
    }

    #[test]
    fn test_model_mapping_overrides_requested_model() {
        let router = OllamaRouter::with_model_mapping(
            "http://localhost:11434",
            "qwen2.5:latest",
            HashMap::from([
                ("default".to_string(), "qwen2.5:latest".to_string()),
                ("gpt-4o".to_string(), "llama3.2:3b".to_string()),
            ]),
        );
        let openai = r#"{"model":"gpt-4o","messages":[{"role":"user","content":"Hello"}]}"#;

        let result = router.convert_openai_to_ollama(openai).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["model"], "llama3.2:3b");
    }

    #[test]
    fn test_convert_ollama_to_openai_response() {
        let router = OllamaRouter::default();
        let ollama = r#"{
            "model": "qwen2.5:latest",
            "message": {"role":"assistant","content":"Hello from local"},
            "done": true,
            "done_reason": "stop",
            "prompt_eval_count": 10,
            "eval_count": 4
        }"#;

        let result = router
            .convert_ollama_to_openai(ollama, "gpt-4o-mini")
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["model"], "gpt-4o-mini");
        assert_eq!(
            parsed["choices"][0]["message"]["content"],
            "Hello from local"
        );
        assert_eq!(parsed["usage"]["total_tokens"], 14);
    }
}
