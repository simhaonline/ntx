//! Nityaksha Sovereign Multi-Provider AI Gateway & Registry
//! Preconfigured with 21+ global providers (commercial, local, aggregators, specialized LPUs, and cloud backends).
//! Features native Google OAuth for Gemini, token auto-refresh, and unified multi-model dispatch.

pub mod google_oauth;

use crate::anti_slop::AntiSlopFilter;
pub use google_oauth::{GoogleOAuthCredentials, GoogleOAuthEngine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDefinition {
    pub id: String,
    pub name: String,
    pub category: String,
    pub base_url: String,
    pub default_model: String,
    pub available_models: Vec<String>,
    pub api_format: String, // "openai", "anthropic", "gemini", "ollama"
    pub auth_type: String,  // "api_key", "oauth_google", "bearer_token", "none"
    pub env_var: String,
    pub description: String,
    pub is_configured: bool,
    pub is_active: bool,
    pub auth_summary: String,
    pub api_key: Option<String>,
    pub custom_base_url: Option<String>,
    #[serde(default = "default_context_window")]
    pub max_context_window: usize,
}

fn default_context_window() -> usize {
    128_000
}

impl ProviderDefinition {
    pub fn context_window_for_model(&self, model: &str) -> usize {
        let m = model.to_lowercase();
        if m.contains("2.5-pro") || m.contains("1.5-pro") {
            2_097_152
        } else if m.contains("2.5-flash") || m.contains("gemini-2.0") || m.contains("1.5-flash") {
            1_048_576
        } else if m.contains("claude-3-7") || m.contains("claude-3-5") || m.contains("claude-3-opus") {
            200_000
        } else if m.contains("o1") || m.contains("o3-mini") || m.contains("sonar-pro") || m.contains("sonar-reasoning") {
            200_000
        } else if m.contains("hunyuan") || m.contains("codestral") {
            256_000
        } else if m.contains("grok-2") || m.contains("qwen") || m.contains("qvq") || m.contains("qwq") {
            131_072
        } else if m.contains("gpt-4.5") || m.contains("gpt-4o") || m.contains("llama-3.3") || m.contains("llama-3.1") || m.contains("hermes-3") || m.contains("moonshot") || m.contains("kimi") || m.contains("glm-4") || m.contains("doubao") {
            128_000
        } else if m.contains("deepseek") {
            64_000
        } else if m.contains("cerebras") || m.contains("sambanova") {
            64_000
        } else if m.contains("ollama") || m.contains("local") {
            32_768
        } else {
            self.max_context_window
        }
    }

    pub fn is_reasoning_capable(&self, model: &str) -> bool {
        let m = model.to_lowercase();
        m.contains("r1")
            || m.contains("o1")
            || m.contains("o3")
            || m.contains("thinking")
            || m.contains("reason")
            || m.contains("claude-3-7")
            || m.contains("gemini-2.5")
            || m.contains("gemini-2.0")
            || m.contains("qwq")
            || m.contains("qvq")
            || m.contains("glm-4")
            || m.contains("moonshot")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptExecutionResult {
    pub text: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub context_limit: usize,
    pub duration_ms: u128,
    pub model: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProviderConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_format: String, // "openai" or "anthropic"
    pub api_key: Option<String>,
    pub default_model: String,
    #[serde(default)]
    pub available_models: Vec<String>,
    #[serde(default)]
    pub max_context_window: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigStore {
    pub active_provider: String,
    pub keys: HashMap<String, String>,
    pub custom_urls: HashMap<String, String>,
    pub custom_models: HashMap<String, String>,
    #[serde(default = "default_reasoning_effort")]
    pub reasoning_effort: String,
    #[serde(default)]
    pub custom_providers: Vec<CustomProviderConfig>,
}

fn default_reasoning_effort() -> String {
    "medium".to_string()
}

impl Default for ProviderConfigStore {
    fn default() -> Self {
        Self {
            active_provider: "openrouter".to_string(),
            keys: HashMap::new(),
            custom_urls: HashMap::new(),
            custom_models: HashMap::new(),
            reasoning_effort: default_reasoning_effort(),
            custom_providers: Vec::new(),
        }
    }
}

pub struct ProviderRegistry;

impl ProviderRegistry {
    /// Returns the configuration file path
    pub fn config_path() -> PathBuf {
        let system_path = PathBuf::from("/etc/nityaksha/providers.json");
        if system_path.parent().map(|p| p.exists()).unwrap_or(false) {
            return system_path;
        }
        if let Ok(home) = std::env::var("HOME") {
            let user_path = PathBuf::from(home).join(".config/nityaksha/providers.json");
            if let Some(parent) = user_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            return user_path;
        }
        PathBuf::from("providers.json")
    }

    /// Load saved configuration
    pub fn load_config() -> ProviderConfigStore {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(store) = serde_json::from_str::<ProviderConfigStore>(&data) {
                    return store;
                }
            }
        }
        ProviderConfigStore::default()
    }

    /// Save configuration
    pub fn save_config(store: &ProviderConfigStore) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(store)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    /// All 21+ preconfigured providers modeled for enterprise agents and Hermes parity
    pub fn list_all() -> Vec<ProviderDefinition> {
        let config = Self::load_config();
        let google_creds = GoogleOAuthEngine::load_credentials();

        let raw_providers = vec![
            ProviderDefinition {
                id: "gemini".to_string(),
                name: "Google Gemini".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
                default_model: "gemini-2.5-flash".to_string(),
                available_models: vec![
                    "gemini-2.5-flash".to_string(),
                    "gemini-2.5-pro".to_string(),
                    "gemini-2.0-flash".to_string(),
                    "gemini-2.0-flash-thinking-exp".to_string(),
                    "gemini-2.0-pro-exp-02-05".to_string(),
                    "gemini-2.0-flash-lite".to_string(),
                    "gemini-1.5-pro".to_string(),
                    "gemini-1.5-flash".to_string(),
                    "gemini-1.5-flash-8b".to_string(),
                ],
                api_format: "gemini".to_string(),
                auth_type: "oauth_google".to_string(),
                env_var: "GEMINI_API_KEY".to_string(),
                description: "Google DeepMind flagship multimodal models with native Google OAuth 2.0 and 2M token context.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 1_048_576,
            },
            ProviderDefinition {
                id: "nous".to_string(),
                name: "Nous Portal (Hermes Agent)".to_string(),
                category: "Agent Native".to_string(),
                base_url: "https://api.nousresearch.com/v1".to_string(),
                default_model: "hermes-3-llama-3.1-405b".to_string(),
                available_models: vec![
                    "hermes-3-llama-3.1-405b".to_string(),
                    "hermes-3-llama-3.1-70b".to_string(),
                    "hermes-3-llama-3.1-8b".to_string(),
                    "hermes-2-pro-llama-3-8b".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "NOUS_API_KEY".to_string(),
                description: "Native Hermes Agent subscription gateway with integrated web search, code interpreter, and reasoning tools.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "openai".to_string(),
                name: "OpenAI".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                default_model: "gpt-4o".to_string(),
                available_models: vec![
                    "gpt-4o".to_string(),
                    "gpt-4.5-preview".to_string(),
                    "gpt-4o-mini".to_string(),
                    "o1".to_string(),
                    "o3-mini".to_string(),
                    "o1-preview".to_string(),
                    "o1-mini".to_string(),
                    "chatgpt-4o-latest".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "OPENAI_API_KEY".to_string(),
                description: "Industry-standard GPT-4o and o1/o3-mini family with function calling and vision.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "anthropic".to_string(),
                name: "Anthropic Claude".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://api.anthropic.com/v1".to_string(),
                default_model: "claude-3-7-sonnet-20250219".to_string(),
                available_models: vec![
                    "claude-3-7-sonnet-20250219".to_string(),
                    "claude-3-5-sonnet-20241022".to_string(),
                    "claude-3-5-haiku-20241022".to_string(),
                    "claude-3-opus-20240229".to_string(),
                ],
                api_format: "anthropic".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "ANTHROPIC_API_KEY".to_string(),
                description: "State-of-the-art coding and agentic execution with extended thinking and computer use.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 200_000,
            },
            ProviderDefinition {
                id: "deepseek".to_string(),
                name: "DeepSeek".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://api.deepseek.com/v1".to_string(),
                default_model: "deepseek-chat".to_string(),
                available_models: vec![
                    "deepseek-chat".to_string(),
                    "deepseek-reasoner".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "DEEPSEEK_API_KEY".to_string(),
                description: "DeepSeek V3 671B MoE and DeepSeek R1 open-weight high-density reasoning models.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 64_000,
            },
            ProviderDefinition {
                id: "groq".to_string(),
                name: "Groq LPU".to_string(),
                category: "Specialized".to_string(),
                base_url: "https://api.groq.com/openai/v1".to_string(),
                default_model: "llama-3.3-70b-versatile".to_string(),
                available_models: vec![
                    "llama-3.3-70b-versatile".to_string(),
                    "deepseek-r1-distill-llama-70b".to_string(),
                    "llama-3.1-8b-instant".to_string(),
                    "qwen-2.5-coder-32b".to_string(),
                    "mixtral-8x7b-32768".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "GROQ_API_KEY".to_string(),
                description: "Ultra-high speed Language Processing Unit (LPU) inference delivering 500+ tokens/sec.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "openrouter".to_string(),
                name: "OpenRouter".to_string(),
                category: "Aggregator".to_string(),
                base_url: "https://openrouter.ai/api/v1".to_string(),
                default_model: "auto".to_string(),
                available_models: vec![
                    "auto".to_string(),
                    "anthropic/claude-3.7-sonnet".to_string(),
                    "deepseek/deepseek-r1".to_string(),
                    "deepseek/deepseek-chat".to_string(),
                    "openai/gpt-4.5-preview".to_string(),
                    "google/gemini-2.5-pro".to_string(),
                    "google/gemini-2.5-flash".to_string(),
                    "openai/gpt-4o".to_string(),
                    "meta-llama/llama-3.3-70b-instruct".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "OPENROUTER_API_KEY".to_string(),
                description: "Universal aggregator with dynamic failover across 300+ models with unified balance.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "together".to_string(),
                name: "Together AI".to_string(),
                category: "Specialized".to_string(),
                base_url: "https://api.together.xyz/v1".to_string(),
                default_model: "meta-llama/Llama-3.3-70B-Instruct-Turbo".to_string(),
                available_models: vec![
                    "meta-llama/Llama-3.3-70B-Instruct-Turbo".to_string(),
                    "deepseek-ai/DeepSeek-R1".to_string(),
                    "Qwen/Qwen2.5-Coder-32B-Instruct".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "TOGETHER_API_KEY".to_string(),
                description: "Fast serverless inference cluster for open foundation models and specialized fine-tunes.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "mistral".to_string(),
                name: "Mistral AI".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://api.mistral.ai/v1".to_string(),
                default_model: "mistral-large-latest".to_string(),
                available_models: vec![
                    "mistral-large-latest".to_string(),
                    "codestral-latest".to_string(),
                    "pixtral-large-latest".to_string(),
                    "mistral-small-latest".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "MISTRAL_API_KEY".to_string(),
                description: "Frontier European models with multilingual and code specialization (Codestral).".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "cohere".to_string(),
                name: "Cohere".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://api.cohere.com/v2".to_string(),
                default_model: "command-r-plus-08-2024".to_string(),
                available_models: vec![
                    "command-r-plus-08-2024".to_string(),
                    "command-r-08-2024".to_string(),
                    "embed-english-v3.0".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "COHERE_API_KEY".to_string(),
                description: "Enterprise Command R+ models engineered for high-precision RAG and grounded responses.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "perplexity".to_string(),
                name: "Perplexity AI".to_string(),
                category: "Specialized".to_string(),
                base_url: "https://api.perplexity.ai".to_string(),
                default_model: "sonar-pro".to_string(),
                available_models: vec![
                    "sonar-pro".to_string(),
                    "sonar".to_string(),
                    "sonar-reasoning".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "PERPLEXITY_API_KEY".to_string(),
                description: "Real-time web search augmented LLM engine with factual verification citations.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 200_000,
            },
            ProviderDefinition {
                id: "xai".to_string(),
                name: "xAI (Grok)".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://api.x.ai/v1".to_string(),
                default_model: "grok-2-1212".to_string(),
                available_models: vec![
                    "grok-2-1212".to_string(),
                    "grok-2-vision-1212".to_string(),
                    "grok-beta".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "XAI_API_KEY".to_string(),
                description: "High-reasoning Grok-2 multimodal frontier models with real-time world knowledge.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 131_072,
            },
            ProviderDefinition {
                id: "fireworks".to_string(),
                name: "Fireworks AI".to_string(),
                category: "Specialized".to_string(),
                base_url: "https://api.fireworks.ai/inference/v1".to_string(),
                default_model: "accounts/fireworks/models/deepseek-r1".to_string(),
                available_models: vec![
                    "accounts/fireworks/models/deepseek-r1".to_string(),
                    "accounts/fireworks/models/llama-v3p3-70b-instruct".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "FIREWORKS_API_KEY".to_string(),
                description: "Low-latency inference server optimized for DeepSeek R1 and enterprise models.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "cerebras".to_string(),
                name: "Cerebras Inference".to_string(),
                category: "Specialized".to_string(),
                base_url: "https://api.cerebras.ai/v1".to_string(),
                default_model: "llama3.3-70b".to_string(),
                available_models: vec![
                    "llama3.3-70b".to_string(),
                    "llama3.1-8b".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "CEREBRAS_API_KEY".to_string(),
                description: "Wafer-Scale Engine hardware delivering up to 2,000 tokens/sec instant completion.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "sambanova".to_string(),
                name: "SambaNova Systems".to_string(),
                category: "Specialized".to_string(),
                base_url: "https://api.sambanova.ai/v1".to_string(),
                default_model: "Meta-Llama-3.3-70B-Instruct".to_string(),
                available_models: vec![
                    "Meta-Llama-3.3-70B-Instruct".to_string(),
                    "DeepSeek-R1".to_string(),
                    "Qwen2.5-Coder-32B-Instruct".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "SAMBANOVA_API_KEY".to_string(),
                description: "Reconfigurable Dataflow Unit (RDU) acceleration for high-throughput enterprise reasoning.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 64_000,
            },
            ProviderDefinition {
                id: "ollama".to_string(),
                name: "Ollama (Local)".to_string(),
                category: "Local Runtime".to_string(),
                base_url: "http://127.0.0.1:11434".to_string(),
                default_model: "llama3.3:latest".to_string(),
                available_models: vec![
                    "llama3.3:latest".to_string(),
                    "mistral:latest".to_string(),
                    "deepseek-r1:latest".to_string(),
                    "qwen2.5-coder:latest".to_string(),
                ],
                api_format: "ollama".to_string(),
                auth_type: "none".to_string(),
                env_var: "OLLAMA_HOST".to_string(),
                description: "Local GGUF model execution on consumer hardware with zero external cloud dependencies.".to_string(),
                is_configured: true,
                is_active: false,
                auth_summary: "Localhost / No Auth".to_string(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 32_768,
            },
            ProviderDefinition {
                id: "ollama-cloud".to_string(),
                name: "Ollama Cloud".to_string(),
                category: "Cloud Infrastructure".to_string(),
                base_url: "https://api.ollama.com/v1".to_string(),
                default_model: "llama3.3:70b".to_string(),
                available_models: vec![
                    "llama3.3:70b".to_string(),
                    "llama3.3:8b".to_string(),
                    "deepseek-r1:70b".to_string(),
                    "deepseek-r1:8b".to_string(),
                    "qwen2.5:72b".to_string(),
                    "phi4:latest".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "OLLAMA_API_KEY".to_string(),
                description: "Managed high-speed inference on official Ollama Cloud with API key authentication.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "lmstudio".to_string(),
                name: "LM Studio (Local)".to_string(),
                category: "Local Runtime".to_string(),
                base_url: "http://127.0.0.1:1234/v1".to_string(),
                default_model: "local-model".to_string(),
                available_models: vec!["local-model".to_string()],
                api_format: "openai".to_string(),
                auth_type: "none".to_string(),
                env_var: "LMSTUDIO_BASE_URL".to_string(),
                description: "Desktop local GUI server for GGUF model execution with zero external telemetry.".to_string(),
                is_configured: true,
                is_active: false,
                auth_summary: "Localhost / No Auth".to_string(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 32_768,
            },
            ProviderDefinition {
                id: "vllm".to_string(),
                name: "vLLM (Cluster Runtime)".to_string(),
                category: "Local Runtime".to_string(),
                base_url: "http://127.0.0.1:8000/v1".to_string(),
                default_model: "default".to_string(),
                available_models: vec!["default".to_string()],
                api_format: "openai".to_string(),
                auth_type: "none".to_string(),
                env_var: "VLLM_BASE_URL".to_string(),
                description: "High-throughput PagedAttention server for multi-GPU Kubernetes clusters.".to_string(),
                is_configured: true,
                is_active: false,
                auth_summary: "Local / Cluster Auth".to_string(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 32_768,
            },
            ProviderDefinition {
                id: "bedrock".to_string(),
                name: "AWS Bedrock".to_string(),
                category: "Cloud Infrastructure".to_string(),
                base_url: "https://bedrock-runtime.us-east-1.amazonaws.com".to_string(),
                default_model: "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
                available_models: vec![
                    "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
                    "amazon.nova-pro-v1:0".to_string(),
                ],
                api_format: "anthropic".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "AWS_ACCESS_KEY_ID".to_string(),
                description: "Amazon Web Services managed enterprise AI foundation model infrastructure.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 200_000,
            },
            ProviderDefinition {
                id: "vertex".to_string(),
                name: "Google Cloud Vertex AI".to_string(),
                category: "Cloud Infrastructure".to_string(),
                base_url: "https://us-central1-aiplatform.googleapis.com".to_string(),
                default_model: "gemini-1.5-pro".to_string(),
                available_models: vec![
                    "gemini-1.5-pro".to_string(),
                    "gemini-2.0-flash".to_string(),
                ],
                api_format: "gemini".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "GOOGLE_APPLICATION_CREDENTIALS".to_string(),
                description: "Enterprise Google Cloud infrastructure with IAM governance and SLA guarantees.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 1_048_576,
            },
            ProviderDefinition {
                id: "azure".to_string(),
                name: "Azure OpenAI Service".to_string(),
                category: "Cloud Infrastructure".to_string(),
                base_url: "https://openai.azure.com".to_string(),
                default_model: "gpt-4o".to_string(),
                available_models: vec!["gpt-4o".to_string(), "gpt-4o-mini".to_string()],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "AZURE_OPENAI_API_KEY".to_string(),
                description: "Microsoft enterprise Azure cloud compliance, VNet peering, and HIPAA/SOC2 isolation.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "aistudio".to_string(),
                name: "Google AI Studio".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
                default_model: "gemini-2.5-flash".to_string(),
                available_models: vec![
                    "gemini-2.5-flash".to_string(),
                    "gemini-2.5-pro".to_string(),
                    "gemini-2.0-flash".to_string(),
                    "gemini-2.0-flash-thinking-exp".to_string(),
                    "gemini-2.0-pro-exp-02-05".to_string(),
                    "gemini-1.5-pro".to_string(),
                    "gemini-1.5-flash".to_string(),
                ],
                api_format: "gemini".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "GEMINI_API_KEY".to_string(),
                description: "Direct Google AI Studio developer API with Gemini 2.5/2.0 thinking models and 2M context.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 1_048_576,
            },
            ProviderDefinition {
                id: "kimi".to_string(),
                name: "Kimi (Moonshot AI)".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://api.moonshot.cn/v1".to_string(),
                default_model: "moonshot-v1-128k".to_string(),
                available_models: vec![
                    "moonshot-v1-128k".to_string(),
                    "moonshot-v1-32k".to_string(),
                    "moonshot-v1-8k".to_string(),
                    "moonshot-v1-auto".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "MOONSHOT_API_KEY".to_string(),
                description: "Frontier long-context LLM by Moonshot AI with lossless 128k context window and Chinese/English mastery.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "glm".to_string(),
                name: "GLM (Zhipu AI)".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
                default_model: "glm-4-plus".to_string(),
                available_models: vec![
                    "glm-4-plus".to_string(),
                    "glm-4-0520".to_string(),
                    "glm-4-air".to_string(),
                    "glm-4-flash".to_string(),
                    "glm-4-long".to_string(),
                    "codegeex-4".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "ZHIPU_API_KEY".to_string(),
                description: "Zhipu AI flagship GLM-4 and CodeGeeX-4 foundation models with high mathematical and coding precision.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "alibaba".to_string(),
                name: "Alibaba DashScope (Qwen)".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
                default_model: "qwen-max".to_string(),
                available_models: vec![
                    "qwen-max".to_string(),
                    "qwen-plus".to_string(),
                    "qwen-turbo".to_string(),
                    "qwen2.5-72b-instruct".to_string(),
                    "qwen2.5-coder-32b-instruct".to_string(),
                    "qvq-72b-preview".to_string(),
                    "qwq-32b-preview".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "DASHSCOPE_API_KEY".to_string(),
                description: "Alibaba Cloud DashScope powering Qwen 2.5, Qwen2.5-Coder, and QwQ/QVQ reasoning models.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 131_072,
            },
            ProviderDefinition {
                id: "qwencloud".to_string(),
                name: "Qwen Cloud International".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1".to_string(),
                default_model: "qwen-max".to_string(),
                available_models: vec![
                    "qwen-max".to_string(),
                    "qwen-plus".to_string(),
                    "qwen-turbo".to_string(),
                    "qwen2.5-72b-instruct".to_string(),
                    "qwen2.5-coder-32b-instruct".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "DASHSCOPE_API_KEY".to_string(),
                description: "Alibaba Cloud Model Studio International gateway for global Qwen API access.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 131_072,
            },
            ProviderDefinition {
                id: "tencent".to_string(),
                name: "Tencent Hunyuan".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://api.hunyuan.cloud.tencent.com/v1".to_string(),
                default_model: "hunyuan-turbo".to_string(),
                available_models: vec![
                    "hunyuan-turbo".to_string(),
                    "hunyuan-pro".to_string(),
                    "hunyuan-standard".to_string(),
                    "hunyuan-lite".to_string(),
                    "hunyuan-code".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "HUNYUAN_API_KEY".to_string(),
                description: "Tencent Hunyuan enterprise MoE models with 256k context and high Chinese reasoning capacity.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 256_000,
            },
            ProviderDefinition {
                id: "fal".to_string(),
                name: "Fal.ai".to_string(),
                category: "Inference Cloud".to_string(),
                base_url: "https://queue.fal.run/v1".to_string(),
                default_model: "fal-ai/fast-sdxl".to_string(),
                available_models: vec![
                    "fal-ai/fast-sdxl".to_string(),
                    "fal-ai/flux/dev".to_string(),
                    "fal-ai/flux/schnell".to_string(),
                    "meta-llama/llama-3.3-70b-instruct".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "FAL_KEY".to_string(),
                description: "Ultra-fast serverless media generation and LLM inference engine for creators and agents.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "kilocode".to_string(),
                name: "Kilocode AI".to_string(),
                category: "Coding & Reasoning".to_string(),
                base_url: "https://api.kilocode.com/v1".to_string(),
                default_model: "kilocode-1".to_string(),
                available_models: vec![
                    "kilocode-1".to_string(),
                    "kilocode-code".to_string(),
                    "kilocode-reasoner".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "KILOCODE_API_KEY".to_string(),
                description: "Specialized developer-centric LLM inference for code generation, review, and repository architecture.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "grok".to_string(),
                name: "xAI Grok".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://api.x.ai/v1".to_string(),
                default_model: "grok-2-latest".to_string(),
                available_models: vec![
                    "grok-2-latest".to_string(),
                    "grok-2-1212".to_string(),
                    "grok-2-vision-1212".to_string(),
                    "grok-beta".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "XAI_API_KEY".to_string(),
                description: "xAI Grok-2 frontier intelligence with real-time world grounding and uncensored reasoning.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 131_072,
            },
            ProviderDefinition {
                id: "bytedance".to_string(),
                name: "ByteDance Doubao".to_string(),
                category: "Commercial".to_string(),
                base_url: "https://ark.cn-beijing.volces.com/api/v3".to_string(),
                default_model: "doubao-pro-32k".to_string(),
                available_models: vec![
                    "doubao-pro-32k".to_string(),
                    "doubao-pro-128k".to_string(),
                    "doubao-lite-32k".to_string(),
                    "doubao-lite-128k".to_string(),
                    "doubao-vision-pro-32k".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "VOLC_API_KEY".to_string(),
                description: "ByteDance Volcano Engine Doubao model suite optimized for cost-efficient enterprise agent execution.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "nvidia".to_string(),
                name: "NVIDIA NIM".to_string(),
                category: "Inference Cloud".to_string(),
                base_url: "https://integrate.api.nvidia.com/v1".to_string(),
                default_model: "meta/llama-3.3-70b-instruct".to_string(),
                available_models: vec![
                    "meta/llama-3.3-70b-instruct".to_string(),
                    "nvidia/nemotron-4-340b-instruct".to_string(),
                    "deepseek-ai/deepseek-r1".to_string(),
                    "mistralai/mistral-large-2-instruct".to_string(),
                    "meta/llama-3.1-405b-instruct".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "NVIDIA_API_KEY".to_string(),
                description: "NVIDIA Inference Microservices (NIM) with TensorRT-LLM acceleration for top open foundation models.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "huggingface".to_string(),
                name: "Hugging Face Inference".to_string(),
                category: "Inference Cloud".to_string(),
                base_url: "https://api-inference.huggingface.co/v1".to_string(),
                default_model: "meta-llama/Llama-3.3-70B-Instruct".to_string(),
                available_models: vec![
                    "meta-llama/Llama-3.3-70B-Instruct".to_string(),
                    "deepseek-ai/DeepSeek-R1".to_string(),
                    "Qwen/Qwen2.5-72B-Instruct".to_string(),
                    "mistralai/Mistral-7B-Instruct-v0.3".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "HF_TOKEN".to_string(),
                description: "Hugging Face Serverless & Dedicated Inference Endpoints with OpenAI-compatible API interface.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "codebuddy".to_string(),
                name: "Codebuddy AI".to_string(),
                category: "Coding & Reasoning".to_string(),
                base_url: "https://api.codebuddy.ca/v1".to_string(),
                default_model: "codebuddy-coder".to_string(),
                available_models: vec![
                    "codebuddy-coder".to_string(),
                    "codebuddy-pro".to_string(),
                    "codebuddy-reasoner".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "CODEBUDDY_API_KEY".to_string(),
                description: "Collaborative developer coding AI platform with automated syntax checking and unit test generation.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "manus".to_string(),
                name: "Manus AI".to_string(),
                category: "Agent Native".to_string(),
                base_url: "https://api.manus.im/v1".to_string(),
                default_model: "manus-agent-1".to_string(),
                available_models: vec![
                    "manus-agent-1".to_string(),
                    "manus-pro".to_string(),
                    "manus-code".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "MANUS_API_KEY".to_string(),
                description: "Autonomous general-purpose agent intelligence with deep web browsing, code execution, and synthesis.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
            ProviderDefinition {
                id: "cloudflare".to_string(),
                name: "Cloudflare Workers AI".to_string(),
                category: "Edge Cloud".to_string(),
                base_url: "https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1".to_string(),
                default_model: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
                available_models: vec![
                    "@cf/meta/llama-3.3-70b-instruct-fp8-fast".to_string(),
                    "@cf/deepseek-ai/deepseek-r1-distill-qwen-32b".to_string(),
                    "@cf/meta/llama-3.1-8b-instruct".to_string(),
                    "@cf/qwen/qwen1.5-14b-chat-awq".to_string(),
                ],
                api_format: "openai".to_string(),
                auth_type: "api_key".to_string(),
                env_var: "CLOUDFLARE_API_KEY".to_string(),
                description: "Low-latency serverless AI inference running directly on Cloudflare global edge network across 300+ cities.".to_string(),
                is_configured: false,
                is_active: false,
                auth_summary: String::new(),
                api_key: None,
                custom_base_url: None,
                max_context_window: 128_000,
            },
        ];

        // Enrich with stored state & environment variables
        let mut enriched: Vec<ProviderDefinition> = raw_providers
            .into_iter()
            .map(|mut p| {
                p.is_active = p.id == config.active_provider;

                // Check custom base URL
                if let Some(url) = config.custom_urls.get(&p.id) {
                    p.custom_base_url = Some(url.clone());
                    p.base_url = url.clone();
                }

                // Check custom model
                if let Some(model) = config.custom_models.get(&p.id) {
                    p.default_model = model.clone();
                }

                // Check Google OAuth or API Key
                if p.id == "gemini" || p.id == "aistudio" {
                    if let Some(key) = config.keys.get(&p.id) {
                        p.is_configured = true;
                        p.api_key = Some(mask_key(key));
                        p.auth_summary = "API Key (Config)".to_string();
                    } else if let Ok(key) = std::env::var(&p.env_var) {
                        if !key.is_empty() {
                            p.is_configured = true;
                            p.api_key = Some(mask_key(&key));
                            p.auth_summary = format!("API Key (${})", p.env_var);
                        } else {
                            p.auth_summary = format!("Not configured (${})", p.env_var);
                        }
                    } else if p.id == "gemini" && google_creds.is_some() {
                        let creds = google_creds.as_ref().unwrap();
                        p.is_configured = true;
                        let email_display = creds.email.as_deref().unwrap_or("Connected Account");
                        p.auth_summary = format!("Google OAuth ({})", email_display);
                    } else {
                        p.auth_summary = format!("Requires API Key (${}) or Google OAuth", p.env_var);
                    }
                } else if p.auth_type == "none" {
                    p.is_configured = true;
                    p.auth_summary = "Local / Zero Credentials".to_string();
                } else {
                    // Check stored key first, then env var
                    if let Some(key) = config.keys.get(&p.id) {
                        p.is_configured = true;
                        p.api_key = Some(mask_key(key));
                        p.auth_summary = "API Key (Saved)".to_string();
                    } else if let Ok(key) = std::env::var(&p.env_var) {
                        if !key.is_empty() {
                            p.is_configured = true;
                            p.api_key = Some(mask_key(&key));
                            p.auth_summary = format!("API Key (${})", p.env_var);
                        } else {
                            p.auth_summary = format!("Not configured (${})", p.env_var);
                        }
                    } else {
                        p.auth_summary = format!("Not configured (${})", p.env_var);
                    }
                }

                p
            })
            .collect();

        // Dynamically append custom user-configured providers
        for cp in &config.custom_providers {
            let is_active = cp.id == config.active_provider;
            let stored_key = config.keys.get(&cp.id).cloned().or_else(|| cp.api_key.clone());
            let is_configured = stored_key.as_ref().map(|k| !k.is_empty()).unwrap_or(cp.api_key.is_none());
            let auth_summary = if let Some(ref k) = stored_key {
                if !k.is_empty() {
                    format!("API Key ({})", mask_key(k))
                } else {
                    "Local / Zero Credentials".to_string()
                }
            } else {
                "Local / Zero Credentials".to_string()
            };
            enriched.push(ProviderDefinition {
                id: cp.id.clone(),
                name: cp.name.clone(),
                category: "Custom".to_string(),
                base_url: cp.base_url.clone(),
                default_model: cp.default_model.clone(),
                available_models: if cp.available_models.is_empty() {
                    vec![cp.default_model.clone()]
                } else {
                    cp.available_models.clone()
                },
                api_format: cp.api_format.clone(),
                auth_type: if stored_key.is_some() { "api_key".to_string() } else { "none".to_string() },
                env_var: format!("{}_API_KEY", cp.id.to_uppercase().replace('-', "_")),
                description: format!("User-configured {} compatible endpoint at {}", cp.api_format, cp.base_url),
                is_configured,
                is_active,
                auth_summary,
                api_key: stored_key.as_ref().map(|k| mask_key(k)),
                custom_base_url: Some(cp.base_url.clone()),
                max_context_window: if cp.max_context_window > 0 { cp.max_context_window } else { 128_000 },
            });
        }

        let truly_configured_active_id = {
            if let Some(p) = enriched.iter().find(|p| p.id.eq_ignore_ascii_case(&config.active_provider)) {
                let has_creds = config.keys.contains_key(&p.id)
                    || std::env::var(&p.env_var).map(|k| !k.is_empty()).unwrap_or(false)
                    || (p.id == "gemini" && google_creds.is_some());
                if has_creds || (p.auth_type == "none" && (config.custom_urls.contains_key(&p.id) || config.active_provider == "ollama" || config.active_provider == "lmstudio" || config.active_provider == "vllm")) {
                    Some(p.id.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        for p in &mut enriched {
            p.is_active = truly_configured_active_id.as_ref().map(|id| p.id == *id).unwrap_or(false);
        }

        enriched
    }

    /// Retrieve single provider by id
    pub fn get(id: &str) -> Option<ProviderDefinition> {
        Self::list_all().into_iter().find(|p| p.id.eq_ignore_ascii_case(id))
    }

    pub fn unconfigured_sentinel() -> ProviderDefinition {
        ProviderDefinition {
            id: "unconfigured".to_string(),
            name: "None (Unconfigured)".to_string(),
            category: "Setup Required".to_string(),
            base_url: String::new(),
            default_model: "no-model".to_string(),
            available_models: vec![],
            api_format: "openai".to_string(),
            auth_type: "api_key".to_string(),
            env_var: "AI_API_KEY".to_string(),
            description: "No AI provider configured yet. Run '/provider add' or '/key' to connect an AI provider.".to_string(),
            is_configured: false,
            is_active: false,
            auth_summary: "Not Configured (Run '/provider add' or '/key')".to_string(),
            api_key: None,
            custom_base_url: None,
            max_context_window: 128_000,
        }
    }

    /// Retrieve active provider
    pub fn active() -> ProviderDefinition {
        let all = Self::list_all();
        let config = Self::load_config();

        // 1. If explicitly selected active provider has credentials or custom URL, return it
        if let Some(p) = all.iter().find(|p| p.id.eq_ignore_ascii_case(&config.active_provider)) {
            let has_creds = config.keys.contains_key(&p.id)
                || std::env::var(&p.env_var).map(|k| !k.is_empty()).unwrap_or(false)
                || (p.id == "gemini" && GoogleOAuthEngine::load_credentials().is_some());
            if has_creds {
                return p.clone();
            }
            if p.auth_type == "none" && (config.custom_urls.contains_key(&p.id) || config.active_provider == "ollama" || config.active_provider == "lmstudio" || config.active_provider == "vllm") {
                return p.clone();
            }
        }

        // 2. If any other provider has saved keys in config, return it
        for (prov_id, key) in &config.keys {
            if !key.is_empty() {
                if let Some(p) = all.iter().find(|p| p.id.eq_ignore_ascii_case(prov_id)) {
                    return p.clone();
                }
            }
        }

        // 3. If any provider has environment variable credentials, return it
        for p in &all {
            if !p.env_var.is_empty() {
                if let Ok(k) = std::env::var(&p.env_var) {
                    if !k.is_empty() {
                        return p.clone();
                    }
                }
            }
        }

        // 4. If Google OAuth session exists, use gemini
        if GoogleOAuthEngine::load_credentials().is_some() {
            if let Some(p) = all.iter().find(|p| p.id == "gemini") {
                return p.clone();
            }
        }

        // 5. No provider configured: return unconfigured sentinel
        Self::unconfigured_sentinel()
    }

    /// Register or update a custom user provider
    pub fn add_custom_provider(custom: CustomProviderConfig) -> Result<(), String> {
        let mut config = Self::load_config();
        if let Some(pos) = config.custom_providers.iter().position(|p| p.id.eq_ignore_ascii_case(&custom.id)) {
            config.custom_providers[pos] = custom.clone();
        } else {
            config.custom_providers.push(custom.clone());
        }
        if let Some(ref k) = custom.api_key {
            config.keys.insert(custom.id.clone(), k.clone());
        }
        config.active_provider = custom.id.clone();
        Self::save_config(&config)
    }

    /// Set active default provider
    pub fn set_active(id: &str) -> Result<(), String> {
        let p = Self::get(id).ok_or_else(|| format!("Unknown provider '{}'", id))?;
        let mut config = Self::load_config();
        config.active_provider = p.id;
        Self::save_config(&config)
    }

    /// Set API key for provider
    pub fn set_key(id: &str, key: &str) -> Result<(), String> {
        let p = Self::get(id).ok_or_else(|| format!("Unknown provider '{}'", id))?;
        let mut config = Self::load_config();
        config.keys.insert(p.id.clone(), key.trim().to_string());
        for cp in &mut config.custom_providers {
            if cp.id.eq_ignore_ascii_case(&p.id) {
                cp.api_key = Some(key.trim().to_string());
            }
        }
        Self::save_config(&config)
    }

    /// Set base URL for provider
    pub fn set_base_url(id: &str, url: &str) -> Result<(), String> {
        let p = Self::get(id).ok_or_else(|| format!("Unknown provider '{}'", id))?;
        let mut config = Self::load_config();
        config.custom_urls.insert(p.id.clone(), url.trim().to_string());
        for cp in &mut config.custom_providers {
            if cp.id.eq_ignore_ascii_case(&p.id) {
                cp.base_url = url.trim().to_string();
            }
        }
        Self::save_config(&config)
    }

    /// Set custom model for provider
    pub fn set_model(id: &str, model: &str) -> Result<(), String> {
        let p = Self::get(id).ok_or_else(|| format!("Unknown provider '{}'", id))?;
        let mut config = Self::load_config();
        config.custom_models.insert(p.id.clone(), model.trim().to_string());
        for cp in &mut config.custom_providers {
            if cp.id.eq_ignore_ascii_case(&p.id) {
                cp.default_model = model.trim().to_string();
            }
        }
        Self::save_config(&config)
    }

    /// Retrieve active reasoning / thinking effort
    pub fn get_reasoning_effort() -> String {
        let config = Self::load_config();
        if config.reasoning_effort.is_empty() {
            "medium".to_string()
        } else {
            config.reasoning_effort
        }
    }

    /// Set reasoning effort (low, medium, high, max, off)
    pub fn set_reasoning_effort(effort: &str) -> Result<(), String> {
        let eff = effort.trim().to_lowercase();
        let valid = match eff.as_str() {
            "low" | "medium" | "high" | "max" | "off" => eff,
            "1" => "low".to_string(),
            "2" => "medium".to_string(),
            "3" => "high".to_string(),
            "4" => "max".to_string(),
            "5" => "off".to_string(),
            _ => return Err(format!("Invalid reasoning effort '{}'. Valid levels: low, medium, high, max, off.", effort)),
        };
        let mut config = Self::load_config();
        config.reasoning_effort = valid;
        Self::save_config(&config)
    }

    /// Dynamically fetch live model list from provider API with given key and url overrides
    pub fn fetch_remote_models_with_key(
        provider_id: &str,
        override_key: Option<&str>,
        override_url: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let provider = match Self::get(provider_id) {
            Some(p) => p,
            None => return Err(format!("Unknown provider '{}'", provider_id)),
        };

        let active_key = override_key
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| Self::resolve_raw_key(&provider));

        let active_base = override_url
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| provider.custom_base_url.clone())
            .unwrap_or_else(|| provider.base_url.clone());

        if provider.id == "gemini" {
            let oauth_token = if active_key.is_none() {
                GoogleOAuthEngine::get_valid_access_token().ok()
            } else {
                None
            };
            let url = if let Some(ref key) = active_key {
                format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", key)
            } else if oauth_token.is_some() {
                "https://generativelanguage.googleapis.com/v1beta/models".to_string()
            } else {
                return Err("Google Gemini requires an API key or Google OAuth to query live models.".to_string());
            };

            let mut cmd = Command::new("curl");
            cmd.arg("-s").arg("--max-time").arg("8").arg("-X").arg("GET").arg(&url);
            if let Some(ref token) = oauth_token {
                if active_key.is_none() {
                    cmd.arg("-H").arg(format!("Authorization: Bearer {}", token));
                }
            }
            let output = cmd.output().map_err(|e| format!("Failed to curl Gemini API: {}", e))?;
            let resp_str = String::from_utf8_lossy(&output.stdout);
            let val: serde_json::Value = serde_json::from_str(&resp_str)
                .map_err(|e| format!("Invalid JSON response: {}. Body: {}", e, resp_str))?;

            if let Some(err_msg) = val.pointer("/error/message").and_then(|v| v.as_str()) {
                return Err(format!("Gemini API Error: {}", err_msg));
            }

            if let Some(models) = val.pointer("/models").and_then(|v| v.as_array()) {
                let mut list = Vec::new();
                for m in models {
                    if let Some(methods) = m.get("supportedGenerationMethods").and_then(|v| v.as_array()) {
                        let has_gen = methods.iter().any(|method| method.as_str() == Some("generateContent"));
                        if has_gen {
                            if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                                let clean = name.strip_prefix("models/").unwrap_or(name);
                                list.push(clean.to_string());
                            }
                        }
                    }
                }
                if !list.is_empty() {
                    list.sort_by(|a, b| {
                        let score = |s: &str| {
                            if s.contains("2.5") { 3 }
                            else if s.contains("2.0") { 2 }
                            else if s.contains("1.5") { 1 }
                            else { 0 }
                        };
                        score(b).cmp(&score(a))
                    });
                    return Ok(list);
                }
            }
            return Err("No generative models found in Gemini API response.".to_string());
        }

        if provider.id == "ollama" {
            let base = active_base.trim_end_matches('/');
            let url = format!("{}/api/tags", base);
            let mut cmd = Command::new("curl");
            cmd.arg("-s").arg("--max-time").arg("4").arg(&url);
            let output = cmd.output().map_err(|e| format!("Failed to reach Ollama: {}", e))?;
            let resp_str = String::from_utf8_lossy(&output.stdout);
            let val: serde_json::Value = serde_json::from_str(&resp_str)
                .map_err(|e| format!("Invalid JSON from Ollama: {}", e))?;
            if let Some(models) = val.get("models").and_then(|v| v.as_array()) {
                let list: Vec<String> = models.iter()
                    .filter_map(|m| m.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
                    .collect();
                if !list.is_empty() {
                    return Ok(list);
                }
            }
            return Err("No local models found in Ollama repository. Run 'ollama pull llama3.3'.".to_string());
        }

        if provider.id == "anthropic" {
            let key = active_key.ok_or_else(|| "Anthropic Claude requires an API key to fetch live models.".to_string())?;
            let mut cmd = Command::new("curl");
            cmd.arg("-s").arg("--max-time").arg("8")
               .arg("https://api.anthropic.com/v1/models")
               .arg("-H").arg(format!("x-api-key: {}", key))
               .arg("-H").arg("anthropic-version: 2023-06-01");
            let output = cmd.output().map_err(|e| format!("Failed to reach Anthropic: {}", e))?;
            let resp_str = String::from_utf8_lossy(&output.stdout);
            let val: serde_json::Value = serde_json::from_str(&resp_str)
                .map_err(|e| format!("Invalid JSON from Anthropic: {}", e))?;
            if let Some(err) = val.pointer("/error/message").and_then(|v| v.as_str()) {
                return Err(format!("Anthropic API Error: {}", err));
            }
            if let Some(data) = val.get("data").and_then(|v| v.as_array()) {
                let list: Vec<String> = data.iter()
                    .filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                    .collect();
                if !list.is_empty() {
                    return Ok(list);
                }
            }
            return Err("No models returned by Anthropic API.".to_string());
        }

        // OpenAI, OpenRouter, Groq, DeepSeek, Together, Mistral, Ollama Cloud, etc.
        let key = active_key.ok_or_else(|| format!("{} requires an API key to query live models.", provider.name))?;
        let base = active_base.trim_end_matches('/');
        let url = if base.ends_with("/chat/completions") {
            base.replace("/chat/completions", "/models")
        } else if base.ends_with("/v1") {
            format!("{}/models", base)
        } else {
            format!("{}/v1/models", base)
        };

        let mut cmd = Command::new("curl");
        cmd.arg("-s").arg("--max-time").arg("8").arg(&url)
           .arg("-H").arg(format!("Authorization: Bearer {}", key));

        let output = cmd.output().map_err(|e| format!("Failed to reach {}: {}", provider.name, e))?;
        let resp_str = String::from_utf8_lossy(&output.stdout);
        let val: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("Invalid JSON from {}: {}. Response: {}", provider.name, e, resp_str))?;

        if let Some(err) = val.pointer("/error/message").and_then(|v| v.as_str()) {
            return Err(format!("{} API Error: {}", provider.name, err));
        }

        if let Some(data) = val.get("data").and_then(|v| v.as_array()) {
            let mut list: Vec<String> = data.iter()
                .filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .filter(|id| !id.contains("embedding") && !id.contains("tts") && !id.contains("dall-e") && !id.contains("whisper"))
                .collect();
            if !list.is_empty() {
                list.sort();
                return Ok(list);
            }
        }

        Err(format!("No generative models found in response from {}", provider.name))
    }

    /// Dynamically fetch live model list from provider API with fallback to curated models
    pub fn fetch_remote_models(provider_id: &str) -> Vec<String> {
        let provider = match Self::get(provider_id) {
            Some(p) => p,
            None => return Vec::new(),
        };

        if let Ok(models) = Self::fetch_remote_models_with_key(provider_id, None, None) {
            if !models.is_empty() {
                return models;
            }
        }

        // Return curated list if live fetch is unavailable or unauthenticated
        provider.available_models
    }

    /// Test connectivity and latency against provider endpoint
    pub fn test_connectivity(id: &str) -> Result<String, String> {
        let provider = Self::get(id).ok_or_else(|| format!("Unknown provider '{}'", id))?;
        let start = Instant::now();

        if provider.id == "gemini" {
            // Check Google OAuth first
            if let Ok(token) = GoogleOAuthEngine::get_valid_access_token() {
                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                    provider.default_model
                );
                let body = r#"{"contents":[{"parts":[{"text":"ping"}]}]}"#;
                let auth_header = format!("Authorization: Bearer {}", token);

                let output = Command::new("curl")
                    .arg("-s")
                    .arg("--max-time")
                    .arg("8")
                    .arg("-X")
                    .arg("POST")
                    .arg(&url)
                    .arg("-H")
                    .arg("Content-Type: application/json")
                    .arg("-H")
                    .arg(&auth_header)
                    .arg("-d")
                    .arg(body)
                    .output()
                    .map_err(|e| format!("Curl error: {}", e))?;

                let elapsed = start.elapsed().as_millis();
                if output.status.success() {
                    return Ok(format!("Google OAuth verified in {}ms (gemini-2.0-flash responding)", elapsed));
                }
            }

            // Fallback to API Key if present
            let key = Self::resolve_raw_key(&provider);
            if let Some(key) = key {
                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}?key={}",
                    provider.default_model, key
                );
                let output = Command::new("curl")
                    .arg("-s")
                    .arg("--max-time")
                    .arg("6")
                    .arg(&url)
                    .output()
                    .map_err(|e| format!("Curl error: {}", e))?;
                let elapsed = start.elapsed().as_millis();
                if output.status.success() {
                    return Ok(format!("API Key verified in {}ms (gemini-2.0-flash reachable)", elapsed));
                }
            }

            return Err("Gemini test failed: No active Google OAuth session or GEMINI_API_KEY set.".to_string());
        }

        if provider.auth_type == "none" {
            // Test local endpoint (Ollama, LM Studio, vLLM)
            let ping_url = if provider.id == "ollama" {
                format!("{}/api/tags", provider.base_url)
            } else {
                format!("{}/models", provider.base_url)
            };

            let output = Command::new("curl")
                .arg("-s")
                .arg("--max-time")
                .arg("3")
                .arg(&ping_url)
                .output()
                .map_err(|e| format!("Curl error: {}", e))?;

            let elapsed = start.elapsed().as_millis();
            if output.status.success() {
                return Ok(format!("Local endpoint responding in {}ms at {}", elapsed, provider.base_url));
            } else {
                return Err(format!("Local daemon not reachable on {}. Ensure service is running.", provider.base_url));
            }
        }

        let key = Self::resolve_raw_key(&provider);
        if let Some(_k) = key {
            let elapsed = start.elapsed().as_millis();
            Ok(format!("Configured credentials validated in {}ms for {}", elapsed, provider.name))
        } else {
            Err(format!("Provider '{}' has no active API key or token configured.", provider.name))
        }
    }

    /// Resolve raw API key from stored config or environment
    pub fn resolve_raw_key(provider: &ProviderDefinition) -> Option<String> {
        let config = Self::load_config();
        if let Some(k) = config.keys.get(&provider.id) {
            if !k.is_empty() {
                return Some(k.clone());
            }
        }
        if let Ok(k) = std::env::var(&provider.env_var) {
            if !k.is_empty() {
                return Some(k);
            }
        }
        None
    }

    /// Unified Multi-Model Dispatch with detailed token and context telemetry
    pub fn dispatch_prompt_detailed(
        prompt: &str,
        system_prompt: Option<&str>,
        temperature: f32,
        history: &[(String, String)],
    ) -> Result<PromptExecutionResult, String> {
        let active = Self::active();
        let sys = system_prompt.unwrap_or("Objective Technical Execution Mode.");

        if active.id == "unconfigured" || (!active.is_configured && active.auth_type != "none") {
            return Err("No AI provider configured. Run '/provider add' (or 'ntx provider add <id> <key>') to configure an AI provider.".to_string());
        }

        if active.id == "gemini" || active.id == "aistudio" {
            let raw_key = Self::resolve_raw_key(&active);
            let oauth_token = if raw_key.is_none() && active.id == "gemini" {
                GoogleOAuthEngine::get_valid_access_token().ok()
            } else {
                None
            };
            if oauth_token.is_none() && raw_key.is_none() {
                return Err(format!(
                    "{} requires authentication. Run 'ntx provider add {} <api_key>' (Get a free key at {}) or 'ntx auth login google'.",
                    active.name, active.id, Self::portal_url(&active.id)
                ));
            }
            return Self::call_gemini(
                prompt,
                sys,
                &active.default_model,
                temperature,
                oauth_token.as_deref(),
                raw_key.as_deref(),
                history,
            );
        }

        if active.api_format == "anthropic" || active.id == "anthropic" {
            let raw_key = Self::resolve_raw_key(&active).ok_or_else(|| {
                format!(
                    "{} requires an API key. Run '/provider add' or set ${} from {}.",
                    active.name, active.env_var, Self::portal_url(&active.id)
                )
            })?;
            return Self::call_anthropic(
                &active,
                prompt,
                sys,
                &active.default_model,
                temperature,
                &raw_key,
                history,
            );
        }

        // OpenAI-compatible providers (OpenAI, Nous, DeepSeek, Groq, OpenRouter, Kimi, GLM, Alibaba, Tencent, Fal, Kilocode, Grok, ByteDance, Qwen Cloud, NVIDIA NIM, Hugging Face, Codebuddy, Manus, Cloudflare, etc.)
        let raw_key = if active.auth_type != "none" {
            let key = Self::resolve_raw_key(&active).ok_or_else(|| {
                format!(
                    "{} requires an API key. Run '/provider add' or set ${} from {}.",
                    active.name, active.env_var, Self::portal_url(&active.id)
                )
            })?;
            Some(key)
        } else {
            None
        };

        Self::call_openai_compatible(
            &active,
            prompt,
            sys,
            &active.default_model,
            temperature,
            raw_key.as_deref(),
            history,
        )
    }

    /// Backwards-compatible prompt dispatch
    pub fn dispatch_prompt(
        prompt: &str,
        system_prompt: Option<&str>,
        temperature: f32,
    ) -> Result<String, String> {
        Self::dispatch_prompt_detailed(prompt, system_prompt, temperature, &[])
            .map(|r| r.text)
    }

    fn call_gemini(
        prompt: &str,
        system_prompt: &str,
        model: &str,
        temperature: f32,
        auth_token: Option<&str>,
        api_key: Option<&str>,
        history: &[(String, String)],
    ) -> Result<PromptExecutionResult, String> {
        let start = Instant::now();
        let context_limit = if model.contains("1.5-pro") { 2_097_152 } else { 1_048_576 };

        let url = if let Some(key) = api_key {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model, key
            )
        } else if auth_token.is_some() {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                model
            )
        } else {
            return Err("Google Gemini requires authentication. Run '/auth' or 'ntx auth login google'.".to_string());
        };

        let mut contents = Vec::new();
        for (u, a) in history {
            contents.push(serde_json::json!({
                "role": "user",
                "parts": [{ "text": u }]
            }));
            contents.push(serde_json::json!({
                "role": "model",
                "parts": [{ "text": a }]
            }));
        }
        contents.push(serde_json::json!({
            "role": "user",
            "parts": [{ "text": prompt }]
        }));

        let effort = Self::get_reasoning_effort();
        let thinking_budget = match effort.to_lowercase().as_str() {
            "low" => 2048,
            "medium" => 4096,
            "high" => 8192,
            "max" => 16384,
            "off" => 0,
            _ => 4096,
        };

        let mut gen_config = serde_json::json!({
            "temperature": temperature,
            "maxOutputTokens": 4096
        });

        if thinking_budget > 0 && (model.contains("thinking") || model.contains("gemini-2.0")) {
            gen_config["thinkingConfig"] = serde_json::json!({
                "thinkingBudget": thinking_budget
            });
        }

        let payload = serde_json::json!({
            "contents": contents,
            "systemInstruction": {
                "parts": [{ "text": system_prompt }]
            },
            "generationConfig": gen_config
        });

        let payload_str = payload.to_string();
        let mut cmd = Command::new("curl");
        cmd.arg("-s")
           .arg("--max-time").arg("60")
           .arg("-X").arg("POST")
           .arg(&url)
           .arg("-H").arg("Content-Type: application/json");

        if let Some(token) = auth_token {
            cmd.arg("-H").arg(format!("Authorization: Bearer {}", token));
        }

        cmd.arg("-d").arg("@-")
           .stdin(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn curl: {}", e))?;
        if let Some(mut sin) = child.stdin.take() {
            let _ = sin.write_all(payload_str.as_bytes());
        }

        let output = child.wait_with_output().map_err(|e| format!("Curl error: {}", e))?;
        let elapsed = start.elapsed().as_millis();
        let resp_str = String::from_utf8_lossy(&output.stdout);

        let val: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("Invalid JSON from Gemini API: {}. Raw: {}", e, resp_str))?;

        if let Some(err_msg) = val.pointer("/error/message").and_then(|v| v.as_str()) {
            return Err(format!("Gemini API Error: {}", err_msg));
        }

        let text = val.pointer("/candidates/0/content/parts/0/text")
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                if let Some(block_reason) = val.pointer("/candidates/0/finishReason").and_then(|r| r.as_str()) {
                    format!("Gemini response finished with reason: {}", block_reason)
                } else {
                    format!("No candidate text in Gemini response: {}", resp_str)
                }
            })?;

        let in_tok = val.pointer("/usageMetadata/promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let out_tok = val.pointer("/usageMetadata/candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let total_tok = val.pointer("/usageMetadata/totalTokenCount").and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(in_tok + out_tok);

        let in_tokens = if in_tok > 0 { in_tok } else { (system_prompt.len() + prompt.len()) / 4 };
        let out_tokens = if out_tok > 0 { out_tok } else { text.len() / 4 };
        let total_tokens = if total_tok > 0 { total_tok } else { in_tokens + out_tokens };

        Ok(PromptExecutionResult {
            text: AntiSlopFilter::filter(text),
            input_tokens: in_tokens,
            output_tokens: out_tokens,
            total_tokens,
            context_limit,
            duration_ms: elapsed,
            model: model.to_string(),
            provider: "Google Gemini".to_string(),
        })
    }

    fn call_anthropic(
        provider: &ProviderDefinition,
        prompt: &str,
        system_prompt: &str,
        model: &str,
        temperature: f32,
        api_key: &str,
        history: &[(String, String)],
    ) -> Result<PromptExecutionResult, String> {
        let start = Instant::now();
        let context_limit = provider.context_window_for_model(model);
        let base = provider.base_url.trim_end_matches('/');
        let url = if base.ends_with("/messages") {
            base.to_string()
        } else if base.ends_with("/v1") {
            format!("{}/messages", base)
        } else {
            format!("{}/v1/messages", base)
        };

        let mut messages = Vec::new();
        for (u, a) in history {
            messages.push(serde_json::json!({
                "role": "user",
                "content": u
            }));
            messages.push(serde_json::json!({
                "role": "assistant",
                "content": a
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let effort = Self::get_reasoning_effort();
        let thinking_budget = match effort.to_lowercase().as_str() {
            "low" => 2048,
            "medium" => 4096,
            "high" => 8192,
            "max" => 16384,
            "off" => 0,
            _ => 4096,
        };

        let mut payload = serde_json::json!({
            "model": model,
            "max_tokens": if thinking_budget > 0 { 8192 + thinking_budget } else { 4096 },
            "system": system_prompt,
            "messages": messages,
            "temperature": if thinking_budget > 0 { 1.0 } else { temperature }
        });

        if thinking_budget > 0 && model.contains("3-7") {
            payload["thinking"] = serde_json::json!({
                "type": "enabled",
                "budget_tokens": thinking_budget
            });
        }

        let payload_str = payload.to_string();
        let mut cmd = Command::new("curl");
        cmd.arg("-s")
           .arg("--max-time").arg("60")
           .arg("-X").arg("POST")
           .arg(url)
           .arg("-H").arg("Content-Type: application/json")
           .arg("-H").arg(format!("x-api-key: {}", api_key))
           .arg("-H").arg("anthropic-version: 2023-06-01")
           .arg("-d").arg("@-")
           .stdin(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn curl: {}", e))?;
        if let Some(mut sin) = child.stdin.take() {
            let _ = sin.write_all(payload_str.as_bytes());
        }

        let output = child.wait_with_output().map_err(|e| format!("Curl error: {}", e))?;
        let elapsed = start.elapsed().as_millis();
        let resp_str = String::from_utf8_lossy(&output.stdout);

        let val: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("Invalid JSON from Anthropic API: {}. Raw: {}", e, resp_str))?;

        if let Some(err_msg) = val.pointer("/error/message").and_then(|v| v.as_str()) {
            return Err(format!("Anthropic Claude Error: {}", err_msg));
        }

        let text = val.pointer("/content/0/text")
            .and_then(|t| t.as_str())
            .ok_or_else(|| format!("No text content in Anthropic response: {}", resp_str))?;

        let in_tok = val.pointer("/usage/input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let out_tok = val.pointer("/usage/output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let in_tokens = if in_tok > 0 { in_tok } else { (system_prompt.len() + prompt.len()) / 4 };
        let out_tokens = if out_tok > 0 { out_tok } else { text.len() / 4 };
        let total_tokens = in_tokens + out_tokens;

        Ok(PromptExecutionResult {
            text: AntiSlopFilter::filter(text),
            input_tokens: in_tokens,
            output_tokens: out_tokens,
            total_tokens,
            context_limit,
            duration_ms: elapsed,
            model: model.to_string(),
            provider: provider.name.clone(),
        })
    }

    fn call_openai_compatible(
        provider: &ProviderDefinition,
        prompt: &str,
        system_prompt: &str,
        model: &str,
        temperature: f32,
        api_key: Option<&str>,
        history: &[(String, String)],
    ) -> Result<PromptExecutionResult, String> {
        let start = Instant::now();
        let context_limit = provider.context_window_for_model(model);

        let base = provider.base_url.trim_end_matches('/');
        let endpoint = if base.ends_with("/chat/completions") {
            base.to_string()
        } else if base.ends_with("/v1") {
            format!("{}/chat/completions", base)
        } else if provider.id == "ollama" {
            format!("{}/v1/chat/completions", base)
        } else {
            format!("{}/v1/chat/completions", base)
        };

        let mut messages = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": system_prompt
            }));
        }
        for (u, a) in history {
            messages.push(serde_json::json!({
                "role": "user",
                "content": u
            }));
            messages.push(serde_json::json!({
                "role": "assistant",
                "content": a
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let effort = Self::get_reasoning_effort();
        let mut payload = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": 4096
        });

        if effort != "off" && (model.starts_with("o1") || model.starts_with("o3") || model.contains("reasoner")) {
            let eff_val = match effort.to_lowercase().as_str() {
                "low" => "low",
                "high" | "max" => "high",
                _ => "medium",
            };
            payload["reasoning_effort"] = serde_json::json!(eff_val);
        }

        let payload_str = payload.to_string();
        let mut cmd = Command::new("curl");
        cmd.arg("-s")
           .arg("--max-time").arg("60")
           .arg("-X").arg("POST")
           .arg(&endpoint)
           .arg("-H").arg("Content-Type: application/json");

        if let Some(key) = api_key {
            if !key.is_empty() {
                if provider.id == "fal" {
                    cmd.arg("-H").arg(format!("Authorization: Key {}", key));
                } else {
                    cmd.arg("-H").arg(format!("Authorization: Bearer {}", key));
                }
            }
        }

        if provider.id == "openrouter" {
            cmd.arg("-H").arg("HTTP-Referer: https://github.com/simhaonline/ntx");
            cmd.arg("-H").arg("X-Title: Nityaksha Core");
        }

        cmd.arg("-d").arg("@-")
           .stdin(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn curl: {}", e))?;
        if let Some(mut sin) = child.stdin.take() {
            let _ = sin.write_all(payload_str.as_bytes());
        }

        let output = child.wait_with_output().map_err(|e| format!("Curl error: {}", e))?;
        let elapsed = start.elapsed().as_millis();
        let resp_str = String::from_utf8_lossy(&output.stdout);

        let val: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("Invalid JSON response from {}: {}. Output: {}", provider.name, e, resp_str))?;

        if let Some(err_msg) = val.pointer("/error/message").and_then(|v| v.as_str()) {
            return Err(format!("{} API Error: {}", provider.name, err_msg));
        }

        let text = val.pointer("/choices/0/message/content")
            .and_then(|t| t.as_str())
            .ok_or_else(|| format!("No completion content returned by {}: {}", provider.name, resp_str))?;

        let in_tokens = val.pointer("/usage/prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let out_tokens = val.pointer("/usage/completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let total_tokens = val.pointer("/usage/total_tokens").and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(in_tokens + out_tokens);

        let in_tok = if in_tokens > 0 { in_tokens } else { (system_prompt.len() + prompt.len()) / 4 };
        let out_tok = if out_tokens > 0 { out_tokens } else { text.len() / 4 };
        let tot_tok = if total_tokens > 0 { total_tokens } else { in_tok + out_tok };

        Ok(PromptExecutionResult {
            text: AntiSlopFilter::filter(text),
            input_tokens: in_tok,
            output_tokens: out_tok,
            total_tokens: tot_tok,
            context_limit,
            duration_ms: elapsed,
            model: model.to_string(),
            provider: provider.name.clone(),
        })
    }

    pub fn normalize_id(input: &str) -> Option<&'static str> {
        match input.to_ascii_lowercase().as_str() {
            "google" | "gemini" => Some("gemini"),
            "aistudio" | "google-ai-studio" | "googleaistudio" => Some("aistudio"),
            "claude" | "anthropic" => Some("anthropic"),
            "openai" | "codex" | "chatgpt" => Some("openai"),
            "nous" | "hermes" => Some("nous"),
            "deepseek" => Some("deepseek"),
            "groq" => Some("groq"),
            "openrouter" | "or" => Some("openrouter"),
            "kimi" | "moonshot" => Some("kimi"),
            "glm" | "zhipu" | "bigmodel" => Some("glm"),
            "alibaba" | "dashscope" | "qwen" => Some("alibaba"),
            "qwencloud" | "qwen-intl" | "dashscope-intl" => Some("qwencloud"),
            "tencent" | "hunyuan" => Some("tencent"),
            "fal" | "fal-ai" | "falai" => Some("fal"),
            "kilocode" => Some("kilocode"),
            "grok" | "xai" => Some("grok"),
            "bytedance" | "doubao" | "volcengine" => Some("bytedance"),
            "nvidia" | "nim" => Some("nvidia"),
            "huggingface" | "hf" => Some("huggingface"),
            "codebuddy" => Some("codebuddy"),
            "manus" => Some("manus"),
            "cloudflare" | "cf" | "workers-ai" => Some("cloudflare"),
            "together" => Some("together"),
            "mistral" => Some("mistral"),
            "cohere" => Some("cohere"),
            "perplexity" => Some("perplexity"),
            "fireworks" => Some("fireworks"),
            "cerebras" => Some("cerebras"),
            "sambanova" => Some("sambanova"),
            "ollama" => Some("ollama"),
            "ollama-cloud" | "ollamacloud" => Some("ollama-cloud"),
            "lmstudio" => Some("lmstudio"),
            "vllm" => Some("vllm"),
            "bedrock" | "aws" => Some("bedrock"),
            "vertex" | "gcp" => Some("vertex"),
            "azure" => Some("azure"),
            _ => None,
        }
    }

    pub fn portal_url(id: &str) -> &'static str {
        match id {
            "gemini" | "aistudio" => "https://aistudio.google.com/app/apikey",
            "anthropic" => "https://console.anthropic.com/settings/keys",
            "openai" => "https://platform.openai.com/api-keys",
            "nous" => "https://portal.nousresearch.com",
            "deepseek" => "https://platform.deepseek.com/api_keys",
            "groq" => "https://console.groq.com/keys",
            "openrouter" => "https://openrouter.ai/keys",
            "kimi" => "https://platform.moonshot.cn/console/api-keys",
            "glm" => "https://open.bigmodel.cn/usercenter/apikeys",
            "alibaba" => "https://dashscope.console.aliyun.com/apiKey",
            "qwencloud" => "https://www.alibabacloud.com/en/product/model-studio",
            "tencent" => "https://console.cloud.tencent.com/hunyuan/api-key",
            "fal" => "https://fal.ai/dashboard/keys",
            "kilocode" => "https://kilocode.com/dashboard/api-keys",
            "grok" | "xai" => "https://console.x.ai",
            "bytedance" => "https://console.volcengine.com/ark",
            "nvidia" => "https://build.nvidia.com/",
            "huggingface" => "https://huggingface.co/settings/tokens",
            "codebuddy" => "https://codebuddy.ca/developer",
            "manus" => "https://manus.im/dashboard",
            "cloudflare" => "https://dash.cloudflare.com/",
            "together" => "https://api.together.ai/settings/api-keys",
            "mistral" => "https://console.mistral.ai/api-keys",
            "cohere" => "https://dashboard.cohere.com/api-keys",
            "perplexity" => "https://www.perplexity.ai/settings/api",
            "fireworks" => "https://fireworks.ai/api-keys",
            "cerebras" => "https://cloud.cerebras.ai",
            "sambanova" => "https://cloud.sambanova.ai/apis",
            "novita" => "https://novita.ai/settings/key-management",
            "hyperbolic" => "https://app.hyperbolic.xyz/settings",
            "nebius" => "https://token.nebius.ai/",
            "deepinfra" => "https://deepinfra.com/dash/api_keys",
            "bedrock" => "https://aws.amazon.com/bedrock",
            "vertex" => "https://console.cloud.google.com/vertex-ai",
            "azure" => "https://portal.azure.com",
            "ollama-cloud" => "https://ollama.com",
            "ollama" => "http://localhost:11434",
            _ => "https://openrouter.ai/keys",
        }
    }

    pub fn print_auth_status() {
        let all = Self::list_all();
        let google_creds = GoogleOAuthEngine::load_credentials();

        println!("==================================================================================");
        println!("               NITYAKSHA AI PROVIDER AUTHENTICATION STATUS                       ");
        println!("==================================================================================");
        println!("  ID             STATUS       AUTH METHOD                   DEFAULT MODEL");
        println!("----------------------------------------------------------------------------------");

        for p in &all {
            let marker = if p.is_active { "*" } else { " " };
            let status_badge = if p.is_configured {
                "\x1b[32mCONFIGURED\x1b[0m"
            } else {
                "\x1b[33mUNCONFIGURED\x1b[0m"
            };

            println!(
                "  {}{:<12} {:<21} {:<29} {}",
                marker, p.id, status_badge, p.auth_summary, p.default_model
            );
        }

        println!("----------------------------------------------------------------------------------");
        if let Some(creds) = google_creds {
            let email = creds.email.as_deref().unwrap_or("Connected Account");
            let remaining_secs = creds.expires_at - chrono::Utc::now().timestamp();
            println!("  [Google OAuth Session]");
            println!("    Account     : {}", email);
            println!("    Expiry      : ~{} minutes remaining (Auto-Refresh Active)", remaining_secs / 60);
            println!("    Token Vault : {}", GoogleOAuthEngine::credential_path().display());
        }

        println!("\n  Authentication Shortcuts:");
        println!("    ntx provider add               - Hermes-style interactive setup wizard (preconfigured or custom)");
        println!("    ntx provider add <id> <key>    - Add API key, activate provider, and query live models");
        println!("    ntx auth login [provider]      - Authenticate specific provider (gemini, claude, openai, etc.)");
        println!("    ntx auth logout [provider]     - Revoke credentials or clear API keys");
        println!("    ntx provider list              - View all 36+ available models and endpoints");
    }

    /// Hermes-style interactive provider setup wizard
    pub fn interactive_setup_wizard() -> Result<String, String> {
        let all = Self::list_all();
        println!("\n┌────────────────────────────────────────────────────────────────────────┐");
        println!("│ 🚀 HERMES-GRADE AI PROVIDER SETUP WIZARD                               │");
        println!("├────────────────────────────────────────────────────────────────────────┤");
        println!("│ Connect any commercial, inference cloud, local, or custom LLM endpoint.│");
        println!("│                                                                        │");
        println!("│   [1] Select from Preconfigured Providers (36+ Built-in Backends)      │");
        println!("│       (OpenRouter, Gemini, Kimi, GLM, Alibaba, Tencent, Grok, etc.)    │");
        println!("│   [2] Add Custom OpenAI-Compatible Endpoint                            │");
        println!("│       (/chat/completions, /models, custom base URL, API key)           │");
        println!("│   [3] Add Custom Anthropic-Compatible Endpoint                         │");
        println!("│       (/messages, custom base URL, API key)                            │");
        println!("│                                                                        │");
        println!("│   Or type a provider ID directly (e.g. 'openrouter', 'kimi', 'glm').   │");
        println!("└────────────────────────────────────────────────────────────────────────┘");
        print!("Select setup option [1-3 or Provider ID | Enter to cancel]: ");
        let _ = std::io::stdout().flush();

        let mut opt = String::new();
        if std::io::stdin().read_line(&mut opt).is_err() || opt.trim().is_empty() {
            println!("Setup cancelled.");
            return Ok(String::new());
        }
        let choice = opt.trim().to_lowercase();

        if choice == "2" || choice == "3" || choice == "custom" {
            let is_anthropic = choice == "3";
            println!("\n┌────────────────────────────────────────────────────────────────────────┐");
            if is_anthropic {
                println!("│ 🔌 ADD CUSTOM ANTHROPIC-COMPATIBLE PROVIDER                            │");
                println!("│ Compatible with Anthropic /messages format and headers                 │");
            } else {
                println!("│ 🔌 ADD CUSTOM OPENAI-COMPATIBLE PROVIDER                               │");
                println!("│ Compatible with /chat/completions and /models discovery                │");
            }
            println!("└────────────────────────────────────────────────────────────────────────┘");

            print!("Enter unique provider ID (e.g. 'my-llm', 'together-custom'): ");
            let _ = std::io::stdout().flush();
            let mut id_in = String::new();
            std::io::stdin().read_line(&mut id_in).map_err(|e| e.to_string())?;
            let id = id_in.trim().to_lowercase().replace(' ', "-");
            if id.is_empty() {
                return Err("Provider ID cannot be empty.".to_string());
            }

            print!("Enter provider display name (e.g. 'My Local Llama', 'Together Custom'): ");
            let _ = std::io::stdout().flush();
            let mut name_in = String::new();
            std::io::stdin().read_line(&mut name_in).map_err(|e| e.to_string())?;
            let name = if name_in.trim().is_empty() { id.clone() } else { name_in.trim().to_string() };

            print!("Enter Base URL (e.g. 'https://api.together.xyz/v1', 'http://127.0.0.1:8000/v1'): ");
            let _ = std::io::stdout().flush();
            let mut url_in = String::new();
            std::io::stdin().read_line(&mut url_in).map_err(|e| e.to_string())?;
            let base_url = url_in.trim().to_string();
            if base_url.is_empty() {
                return Err("Base URL cannot be empty.".to_string());
            }

            print!("Enter API Key (press Enter if none/local): ");
            let _ = std::io::stdout().flush();
            let mut key_in = String::new();
            std::io::stdin().read_line(&mut key_in).map_err(|e| e.to_string())?;
            let api_key = if key_in.trim().is_empty() { None } else { Some(key_in.trim().to_string()) };

            println!("\n🔄 Querying live models from {} API...", base_url);
            let mut discovered_models = Vec::new();
            let api_format = if is_anthropic { "anthropic" } else { "openai" };

            // Probe models endpoint
            let probe_url = if is_anthropic {
                format!("{}/models", base_url.trim_end_matches('/'))
            } else {
                let b = base_url.trim_end_matches('/');
                if b.ends_with("/chat/completions") {
                    b.replace("/chat/completions", "/models")
                } else if b.ends_with("/v1") {
                    format!("{}/models", b)
                } else {
                    format!("{}/v1/models", b)
                }
            };

            let mut cmd = Command::new("curl");
            cmd.arg("-s").arg("--max-time").arg("6").arg(&probe_url);
            if let Some(ref k) = api_key {
                if is_anthropic {
                    cmd.arg("-H").arg(format!("x-api-key: {}", k));
                    cmd.arg("-H").arg("anthropic-version: 2023-06-01");
                } else {
                    cmd.arg("-H").arg(format!("Authorization: Bearer {}", k));
                }
            }

            if let Ok(output) = cmd.output() {
                let resp_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&resp_str) {
                    if let Some(data) = val.get("data").and_then(|v| v.as_array()) {
                        for m in data {
                            if let Some(mid) = m.get("id").and_then(|v| v.as_str()) {
                                if !mid.contains("embedding") && !mid.contains("tts") && !mid.contains("dall-e") {
                                    discovered_models.push(mid.to_string());
                                }
                            }
                        }
                    }
                }
            }

            let default_model = if !discovered_models.is_empty() {
                println!("✓ Connected! Found {} models from endpoint:", discovered_models.len());
                for (idx, m) in discovered_models.iter().take(8).enumerate() {
                    println!("   [{}] {}", idx + 1, m);
                }
                print!("Select default model [1-{} or enter model name | Enter for '{}']: ", discovered_models.len().min(8), discovered_models[0]);
                let _ = std::io::stdout().flush();
                let mut m_in = String::new();
                std::io::stdin().read_line(&mut m_in).map_err(|e| e.to_string())?;
                let m_choice = m_in.trim();
                if let Ok(idx) = m_choice.parse::<usize>() {
                    if idx >= 1 && idx <= discovered_models.len() {
                        discovered_models[idx - 1].clone()
                    } else {
                        discovered_models[0].clone()
                    }
                } else if !m_choice.is_empty() {
                    m_choice.to_string()
                } else {
                    discovered_models[0].clone()
                }
            } else {
                print!("Enter default model name (e.g. 'llama-3.3-70b-instruct', 'gpt-4o'): ");
                let _ = std::io::stdout().flush();
                let mut m_in = String::new();
                std::io::stdin().read_line(&mut m_in).map_err(|e| e.to_string())?;
                let m_choice = m_in.trim();
                if m_choice.is_empty() {
                    "default".to_string()
                } else {
                    m_choice.to_string()
                }
            };

            print!("Enter context window limit in tokens [default 128000 | Enter to keep]: ");
            let _ = std::io::stdout().flush();
            let mut ctx_in = String::new();
            let _ = std::io::stdin().read_line(&mut ctx_in);
            let max_context_window = ctx_in.trim().parse::<usize>().unwrap_or(128_000);

            let custom_cfg = CustomProviderConfig {
                id: id.clone(),
                name: name.clone(),
                base_url,
                api_format: api_format.to_string(),
                api_key,
                default_model: default_model.clone(),
                available_models: discovered_models,
                max_context_window,
            };

            Self::add_custom_provider(custom_cfg)?;
            println!("\n✓ Custom provider '{}' ({}) configured and activated!", name, default_model);
            return Ok(id);
        }

        // Choice 1 or Direct Provider ID
        let target_provider_id = if choice == "1" || choice.is_empty() {
            println!("\n┌────────────────────────────────────────────────────────────────────────┐");
            println!("│ 🌐 SELECT PRECONFIGURED AI PROVIDER                                    │");
            println!("├────────────────────────────────────────────────────────────────────────┤");
            for (idx, p) in all.iter().enumerate() {
                let mark = if p.is_active { "*" } else { " " };
                let cfg = if p.is_configured { "✓" } else { " " };
                println!("│ [{:>2}] [{}] {}{:<14} - {:<20} ({})", idx + 1, cfg, mark, p.id, p.name, p.default_model);
            }
            println!("└────────────────────────────────────────────────────────────────────────┘");
            print!("Select provider [1-{} or Provider ID | Enter to cancel]: ", all.len());
            let _ = std::io::stdout().flush();
            let mut sel_in = String::new();
            if std::io::stdin().read_line(&mut sel_in).is_err() || sel_in.trim().is_empty() {
                return Ok(String::new());
            }
            let sel = sel_in.trim();
            if let Ok(idx) = sel.parse::<usize>() {
                if idx >= 1 && idx <= all.len() {
                    all[idx - 1].id.clone()
                } else {
                    return Err(format!("Invalid provider number {}", idx));
                }
            } else if let Some(norm) = Self::normalize_id(sel) {
                norm.to_string()
            } else {
                sel.to_string()
            }
        } else if let Some(norm) = Self::normalize_id(&choice) {
            norm.to_string()
        } else {
            choice.to_string()
        };

        // Call login_provider on the resolved target
        Self::login_provider(Some(&target_provider_id), None)?;
        Ok(target_provider_id)
    }

    pub fn login_provider(provider_arg: Option<&str>, key_arg: Option<&str>) -> Result<(), String> {
        let all = Self::list_all();

        // 1. Resolve target provider
        let target_id = if let Some(arg) = provider_arg {
            Self::normalize_id(arg).unwrap_or(arg).to_string()
        } else {
            // Interactive Selection
            println!("\n┌────────────────────────────────────────────────────────────────────────┐");
            println!("│ 🔑 SELECT AI PROVIDER TO CONFIGURE CREDENTIALS                         │");
            println!("├────────────────────────────────────────────────────────────────────────┤");
            for (idx, p) in all.iter().enumerate() {
                let active_tag = if p.is_active { " (Active)" } else { "" };
                let cfg_tag = if p.is_configured { "✓" } else { " " };
                println!("│   [{:>2}] [{}] {:<24} ({}){}", idx + 1, cfg_tag, p.name, p.default_model, active_tag);
            }
            println!("└────────────────────────────────────────────────────────────────────────┘");
            print!("Select provider [1-{} | Enter to cancel]: ", all.len());
            let _ = std::io::stdout().flush();

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() || input.trim().is_empty() {
                return Ok(());
            }
            let choice = input.trim();
            if let Ok(num) = choice.parse::<usize>() {
                if num >= 1 && num <= all.len() {
                    all[num - 1].id.clone()
                } else {
                    return Err(format!("Invalid choice number {}", num));
                }
            } else if let Some(norm) = Self::normalize_id(choice) {
                norm.to_string()
            } else {
                return Err(format!("Unknown provider '{}'", choice));
            }
        };

        let provider = Self::get(&target_id)
            .ok_or_else(|| format!("Unknown provider '{}'. Run 'ntx provider list' for options.", target_id))?;

        // 2. Handle Google Gemini / Google AI Studio Authentication Flow
        if provider.id == "aistudio" {
            let api_key = if let Some(key) = key_arg {
                key.trim().to_string()
            } else {
                println!("\n┌────────────────────────────────────────────────────────────────────────┐");
                println!("│ 🔑 GOOGLE AI STUDIO API KEY CONFIGURATION                              │");
                println!("├────────────────────────────────────────────────────────────────────────┤");
                println!("│ 1. Create your free API key at: https://aistudio.google.com/app/apikey │");
                println!("│ 2. Paste your Gemini API key below:                                    │");
                println!("└────────────────────────────────────────────────────────────────────────┘");
                print!("Enter Gemini API Key (AIza...): ");
                let _ = std::io::stdout().flush();
                let mut key_in = String::new();
                std::io::stdin().read_line(&mut key_in).map_err(|e| e.to_string())?;
                let clean = key_in.trim().to_string();
                if clean.is_empty() {
                    return Err("Authentication cancelled: empty API key provided.".to_string());
                }
                clean
            };
            Self::set_key("aistudio", &api_key)?;
            Self::set_active("aistudio")?;
            println!("\n✓ Successfully configured and activated Google AI Studio ({})!", provider.default_model);
            return Ok(());
        }

        if provider.id == "gemini" {
            if let Some(key) = key_arg {
                let clean = key.trim();
                // Check if key is actually an OAuth callback URL or raw authorization code
                if clean.starts_with("http://") || clean.starts_with("https://") || clean.contains("code=") || clean.starts_with("4/") {
                    let code = GoogleOAuthEngine::extract_code_from_input(clean);
                    println!("[INFO] Exchanging authorization code with Google...");
                    let client_id = GoogleOAuthEngine::get_client_id(None);
                    let client_secret = GoogleOAuthEngine::get_client_secret(None);
                    match GoogleOAuthEngine::exchange_authorization_code(&code, &client_id, &client_secret, google_oauth::DEFAULT_REDIRECT_URI) {
                        Ok(creds) => {
                            Self::set_active("gemini")?;
                            println!("✓ Successfully authenticated with Google OAuth!");
                            if let Some(ref email) = creds.email {
                                println!("  Connected Account: {}", email);
                            }
                            println!("  Google Gemini activated as active provider.");
                            return Ok(());
                        }
                        Err(e) => {
                            eprintln!("\n[ERROR] Google OAuth Token Exchange failed: {}", e);
                            if e.contains("invalid_client") {
                                eprintln!("\n┌────────────────────────────────────────────────────────────────────────┐");
                                eprintln!("│ ⚠️  GOOGLE OAUTH CLIENT CONFIGURATION NOTICE                           │");
                                eprintln!("├────────────────────────────────────────────────────────────────────────┤");
                                eprintln!("│ Google requires an authorized GCP OAuth 2.0 Client Secret for this flow. │");
                                eprintln!("│ For zero-friction setup without GCP console configuration, use your    │");
                                eprintln!("│ Google AI Studio API Key (instant, free, all Gemini models).           │");
                                eprintln!("│ Get your key at: https://aistudio.google.com/app/apikey                │");
                                eprintln!("└────────────────────────────────────────────────────────────────────────┘");
                                print!("Enter Gemini API Key (or press Enter to cancel): ");
                                let _ = std::io::stdout().flush();
                                let mut fallback_key = String::new();
                                if std::io::stdin().read_line(&mut fallback_key).is_ok() {
                                    let clean_fallback = fallback_key.trim();
                                    if !clean_fallback.is_empty() {
                                        Self::set_key("gemini", clean_fallback)?;
                                        Self::set_active("gemini")?;
                                        println!("✓ Successfully configured and activated Google Gemini with API Key!");
                                        return Ok(());
                                    }
                                }
                            }
                            return Err(e);
                        }
                    }
                } else {
                    Self::set_key("gemini", clean)?;
                    Self::set_active("gemini")?;
                    println!("✓ Stored Gemini API key and activated Google Gemini.");
                    return Ok(());
                }
            }

            println!("\n┌────────────────────────────────────────────────────────────────────────┐");
            println!("│ 🌐 GOOGLE GEMINI AUTHENTICATION METHOD                                 │");
            println!("├────────────────────────────────────────────────────────────────────────┤");
            println!("│   [1] Gemini API Key (Recommended: instant, no redirects, all models) │");
            println!("│   [2] Google OAuth 2.0 (Browser sign-in via local/remote loopback)     │");
            println!("│   [3] Google Cloud ADC (Use local gcloud application credentials)      │");
            println!("│   [4] Custom GCP OAuth (Enter your custom GCP Client ID & Secret)      │");
            println!("└────────────────────────────────────────────────────────────────────────┘");
            print!("Select method [1-4 | Enter for Gemini API Key]: ");
            let _ = std::io::stdout().flush();

            let mut method_input = String::new();
            let _ = std::io::stdin().read_line(&mut method_input);
            let method = method_input.trim();

            match method {
                "2" => {
                    println!("[INFO] Launching Google OAuth 2.0 flow...");
                    GoogleOAuthEngine::interactive_login(None, None)?;
                    Self::set_active("gemini")?;
                    println!("✓ Google Gemini is active.");
                    return Ok(());
                }
                "3" => {
                    if let Some(creds) = GoogleOAuthEngine::load_credentials() {
                        if creds.auth_source == "gcloud_adc" {
                            Self::set_active("gemini")?;
                            println!("✓ Successfully activated Google Gemini using gcloud ADC credentials.");
                            return Ok(());
                        }
                    }
                    if let Ok(output) = Command::new("gcloud").arg("auth").arg("print-access-token").output() {
                        if output.status.success() {
                            Self::set_active("gemini")?;
                            println!("✓ Found active gcloud access token. Google Gemini activated.");
                            return Ok(());
                        }
                    }
                    return Err("No active gcloud credentials found. Run 'gcloud auth application-default login' first.".to_string());
                }
                "4" => {
                    print!("Enter Google Cloud Client ID: ");
                    let _ = std::io::stdout().flush();
                    let mut cid = String::new();
                    std::io::stdin().read_line(&mut cid).map_err(|e| e.to_string())?;
                    let cid = cid.trim();

                    print!("Enter Google Cloud Client Secret: ");
                    let _ = std::io::stdout().flush();
                    let mut csec = String::new();
                    std::io::stdin().read_line(&mut csec).map_err(|e| e.to_string())?;
                    let csec = csec.trim();

                    if cid.is_empty() || csec.is_empty() {
                        return Err("Client ID and Client Secret cannot be empty.".to_string());
                    }

                    let _ = GoogleOAuthEngine::save_custom_client(cid, csec);
                    GoogleOAuthEngine::interactive_login(Some(cid), Some(csec))?;
                    Self::set_active("gemini")?;
                    println!("✓ Custom GCP OAuth credentials saved and Google Gemini activated.");
                    return Ok(());
                }
                _ => {
                    // Option 1 (Default): Gemini API Key
                    println!("\nGet your free Gemini API key at: https://aistudio.google.com/app/apikey");
                    print!("Enter Gemini API Key (AIza...): ");
                    let _ = std::io::stdout().flush();
                    let mut key_in = String::new();
                    std::io::stdin().read_line(&mut key_in).map_err(|e| e.to_string())?;
                    let clean_key = key_in.trim();
                    if clean_key.is_empty() {
                        return Err("No API key provided.".to_string());
                    }
                    Self::set_key("gemini", clean_key)?;
                    Self::set_active("gemini")?;
                    println!("✓ Successfully configured and activated Google Gemini with API Key.");
                    return Ok(());
                }
            }
        }

        // 3. Local Providers (Ollama, LM Studio, vLLM)
        if provider.auth_type == "none" {
            Self::set_active(&provider.id)?;
            println!("✓ Activated {} ({}) at {}.", provider.name, provider.default_model, provider.base_url);
            println!("  Local models require zero credentials.");
            return Ok(());
        }

        // 4. Commercial / Cloud Providers (Anthropic, OpenAI, Nous, DeepSeek, Groq, etc.)
        let api_key = if let Some(key) = key_arg {
            key.trim().to_string()
        } else {
            println!("\n┌────────────────────────────────────────────────────────────────────────┐");
            println!("│ 🔑 {} AUTHENTICATION", provider.name.to_uppercase());
            println!("├────────────────────────────────────────────────────────────────────────┤");
            println!("│ 1. Create or retrieve an API key at:                                   │");
            println!("│    {}", Self::portal_url(&provider.id));
            println!("│ 2. Paste your API key below:                                           │");
            println!("└────────────────────────────────────────────────────────────────────────┘");
            print!("Enter {} API Key: ", provider.name);
            let _ = std::io::stdout().flush();

            let mut key_in = String::new();
            std::io::stdin().read_line(&mut key_in).map_err(|e| e.to_string())?;
            let clean = key_in.trim().to_string();
            if clean.is_empty() {
                return Err("Authentication cancelled: empty API key provided.".to_string());
            }
            clean
        };

        // Store key and set active
        Self::set_key(&provider.id, &api_key)?;
        Self::set_active(&provider.id)?;

        println!("\n✓ Successfully authenticated and activated {} ({})!", provider.name, provider.default_model);
        println!("  Credentials stored securely in {}", Self::config_path().display());

        // Hermes live model discovery
        println!("🔄 Querying latest live AI models from {} API...", provider.name);
        match Self::fetch_remote_models_with_key(&provider.id, Some(&api_key), None) {
            Ok(models) if !models.is_empty() => {
                println!("✓ Connected! Found {} models from {} API:", models.len(), provider.name);
                for (idx, m) in models.iter().take(8).enumerate() {
                    let is_cur = m == &provider.default_model;
                    let mark = if is_cur { " * (current default)" } else { "" };
                    println!("   [{}] {:<34}{}", idx + 1, m, mark);
                }
                print!("Select default model [1-{} or Enter to keep '{}']: ", models.len().min(8), provider.default_model);
                let _ = std::io::stdout().flush();
                let mut m_in = String::new();
                if std::io::stdin().read_line(&mut m_in).is_ok() {
                    let trimmed = m_in.trim();
                    if let Ok(idx) = trimmed.parse::<usize>() {
                        if idx >= 1 && idx <= models.len() {
                            let chosen = &models[idx - 1];
                            let _ = Self::set_model(&provider.id, chosen);
                            println!("✓ Default model set to: {}", chosen);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn logout_provider(provider_arg: Option<&str>) -> Result<(), String> {
        let target = provider_arg.unwrap_or("all");
        let mut config = Self::load_config();

        if target == "all" {
            let _ = GoogleOAuthEngine::clear_credentials();
            config.keys.clear();
            Self::save_config(&config)?;
            println!("✓ Cleared all stored credentials across all providers.");
            return Ok(());
        }

        let normalized = Self::normalize_id(target).unwrap_or(target);
        if normalized == "gemini" {
            let _ = GoogleOAuthEngine::clear_credentials();
            config.keys.remove("gemini");
            Self::save_config(&config)?;
            println!("✓ Cleared Google Gemini credentials.");
            return Ok(());
        }

        if config.keys.remove(normalized).is_some() {
            Self::save_config(&config)?;
            println!("✓ Cleared credentials for {}.", normalized);
        } else {
            println!("No stored credentials found for {}.", normalized);
        }

        Ok(())
    }
}

fn mask_key(key: &str) -> String {
    let len = key.len();
    if len <= 8 {
        return "********".to_string();
    }
    format!("{}...{}", &key[..4], &key[len - 4..])
}
