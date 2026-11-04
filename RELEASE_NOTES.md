# Nityaksha AI Concierge Platform v1.0.0

Engineered for high-velocity, deterministic AI in native compiled code with ContainerOS-grade POSIX isolation, zero-dependency embedded database memory stack, and multi-model autonomous agency.

---

## 🚀 Key Highlights & Architectural Invariants

- **Zero External Runtime Dependencies**: Zero Python runtime, zero Node.js daemon. Built from scratch with sub-5ms cold startup and sub-5MB baseline memory footprint.
- **Universal AI Provider Gateway (21 Backends)**:
  - **Google Gemini**: Dual-mode authentication via **Google OAuth 2.0 PKCE** (`https://www.googleapis.com/auth/cloud-platform`, OpenID, email, profile) and **AI Studio API Keys**, with automatic background token refresh and `gcloud` ADC fallback.
  - **Anthropic Claude**: Guided onboarding with console API key validation (`console.anthropic.com`).
  - **OpenAI / Codex**: Platform key validation (`platform.openai.com`).
  - **Nous Portal / Hermes**: Enterprise portal integration (`portal.nousresearch.com`).
  - **Specialized LPUs & Accelerators**: Groq LPU (llama-3.3-70b), Cerebras (2,000 tok/s), SambaNova RDU.
  - **High-Throughput Aggregators**: OpenRouter, Together AI, Mistral (Codestral), DeepSeek (R1 / Chat), Cohere, Perplexity (Sonar Pro), xAI (Grok), Fireworks.
  - **Sovereign Local Runtime**: Ollama, LM Studio, vLLM (zero cloud credentials required).
  - **Enterprise Cloud Infrastructures**: AWS Bedrock, GCP Vertex AI, Azure OpenAI Service.
- **Guided Interactive Concierge Shell (`ntx` / `ntx.exe`)**:
  - **Workspace Directory Trust Verification**: Intercepts unconfined folder executions with `[Y/n]` trust prompts, isolating repository mutations within strict permission boundaries.
  - **5 Selectable Presentation Palettes**: Rust Brand (Orange & Deep Navy), Luminous Cyan & Steel, Matrix Emerald, Cyberpunk Neon, Clean Minimal.
  - **Dynamic Context Prompt Bar**: Live indicator of workspace path, active Git branch, trust state, and model tier.
  - **Native Slash Commands**: `/help`, `/provider`, `/auth`, `/theme`, `/trust`, `/bot`, `/workflow`, `/model`, `/doctor`, `/status`, `/clear`, `/exit`.
- **Preconfigured Autonomous Bot Catalog (App Store Architecture)**:
  - 7 specialized bots: `ntx-bot-concierge-prime`, `ntx-bot-code-architect`, `ntx-bot-quant-risk`, `ntx-bot-security-redteam`, `ntx-bot-devops-sre`, `ntx-bot-anti-slop-strategist`, `ntx-bot-legal-compliance`.
- **Directed Acyclic Graph (DAG) Multi-Agent Workflow Runner**:
  - Topological cycle detection via Kahn's algorithm (0 cycles invariant).
  - Multi-tier model routing per node (FastFlash -> Balanced -> DeepReasoning).
  - Human-in-the-Loop (HITL) cryptographic approval gates intercepting destructive actions.
- **Embedded Workstation WebUI & REST API**:
  - Native embedded HTTP server listening on `:8080`.
  - Serves direct downloads for installers (`/install.sh`, `/install.ps1`, `/install-ios.sh`, `/ntx.exe`).

---

## 📦 Quick Installation Commands

### Linux & macOS (x86_64, aarch64, Apple Silicon M1-M4)
```bash
curl -fsSL https://simhaonline.ai/install.sh | bash
```

### Windows (PowerShell 5.1+, Windows 10/11, Windows Server)
Run standard PowerShell (no ExecutionPolicy bypass required):
```powershell
irm https://simhaonline.ai/install.ps1 | iex
```

### Standalone Windows Binary Direct Download
```powershell
curl -LO https://simhaonline.ai/ntx.exe
Unblock-File .\ntx.exe
.\ntx.exe
```

### iOS & iPadOS (iSH Shell, a-Shell, UTM)
```sh
curl -fsSL https://simhaonline.ai/install-ios.sh | sh
```

### Cargo Crate
```bash
cargo install --git https://github.com/simhaonline/ntx.git ntx
```

---

## 🔐 Provider Authentication Quickstart

```bash
# Interactive provider login menu (21 providers)
ntx auth login

# Authenticate Google Gemini via Google OAuth 2.0 PKCE or API Key
ntx auth login google

# Authenticate Anthropic Claude
ntx auth login claude

# Authenticate OpenAI / Codex
ntx auth login openai

# View real-time authentication status across all backends
ntx auth status
```

---

## 🛡️ Binary Integrity Checksums (SHA-256)

```text
1acffcd4c72d962776146336b943473e0423e90fd3d0e4c3797e2b23c9065f74  ntx
0ec3e9e8e6975867faa1ed4c76e8ab657471df77068e23188b8ee1b2776945c8  ntx.exe
```

### Verification
```bash
# Verify Linux
sha256sum -c <<< "1acffcd4c72d962776146336b943473e0423e90fd3d0e4c3797e2b23c9065f74  ntx"

# Verify Windows (PowerShell)
Get-FileHash .\ntx.exe -Algorithm SHA256
```
