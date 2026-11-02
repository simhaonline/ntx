//! Nityaksha Shared LLM Adapter Protocols
//! Standardizes task inputs, outputs, and streaming tool calls across OpenAI, Anthropic, Gemini, and Ollama.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Gemini,
    Ollama,
    DeepSeek,
    Nous,
    Groq,
    OpenRouter,
    Together,
    Mistral,
    Cohere,
    Perplexity,
    XAI,
    Fireworks,
    Cerebras,
    SambaNova,
    LMStudio,
    VLLM,
    Bedrock,
    Vertex,
    Azure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardLlmRequest {
    pub provider: ProviderType,
    pub model: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub tools: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardToolCall {
    pub id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardLlmResponse {
    pub content: String,
    pub tool_calls: Vec<StandardToolCall>,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub finish_reason: String,
}

pub struct SharedAdapter;

impl SharedAdapter {
    /// Format request payload for target LLM provider handshaking
    pub fn build_provider_payload(req: &StandardLlmRequest) -> serde_json::Value {
        match req.provider {
            ProviderType::OpenAI
            | ProviderType::DeepSeek
            | ProviderType::Ollama
            | ProviderType::Nous
            | ProviderType::Groq
            | ProviderType::OpenRouter
            | ProviderType::Together
            | ProviderType::Mistral
            | ProviderType::Cohere
            | ProviderType::Perplexity
            | ProviderType::XAI
            | ProviderType::Fireworks
            | ProviderType::Cerebras
            | ProviderType::SambaNova
            | ProviderType::LMStudio
            | ProviderType::VLLM
            | ProviderType::Azure => {
                serde_json::json!({
                    "model": req.model,
                    "messages": [
                        { "role": "system", "content": req.system_prompt },
                        { "role": "user", "content": req.user_prompt }
                    ],
                    "temperature": req.temperature,
                    "max_tokens": req.max_tokens,
                    "tools": req.tools
                })
            }
            ProviderType::Anthropic | ProviderType::Bedrock => {
                serde_json::json!({
                    "model": req.model,
                    "system": req.system_prompt,
                    "messages": [
                        { "role": "user", "content": req.user_prompt }
                    ],
                    "max_tokens": req.max_tokens,
                    "temperature": req.temperature,
                    "tools": req.tools
                })
            }
            ProviderType::Gemini | ProviderType::Vertex => {
                serde_json::json!({
                    "contents": [
                        { "role": "user", "parts": [{ "text": req.user_prompt }] }
                    ],
                    "systemInstruction": {
                        "parts": [{ "text": req.system_prompt }]
                    },
                    "generationConfig": {
                        "temperature": req.temperature,
                        "maxOutputTokens": req.max_tokens
                    }
                })
            }
        }
    }
}
