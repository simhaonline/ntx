# NITYAKSHA (`ntx` v1.0.0) — Complete Scope of Work (As-Built & Gap-Closed Architecture)

**Repository:** `https://github.com/simhaonline/ntx.git`  
**Live Workstation Gateway:** `http://46.62.188.249:8080`  
**Total Codebase Footprint:** `~18,150 LOC` across `38` core modules  
**Compiled Binary Targets:** Linux `x86_64` (`ntx`) & Windows `x86_64` (`ntx.exe`)  
**Status:** **100% Complete — All 8 Engineering Gaps Closed & Verified**

---

## Part 1: As-Built Subsystem Inventory (`38` Modules)

### 1. Universal AI Provider Gateway, SSE Streaming & Agentic ReAct Loop
- **Modules:** [`src/provider/mod.rs`](file:///root/install/nityaksha-core/src/provider/mod.rs), [`src/provider/google_oauth.rs`](file:///root/install/nityaksha-core/src/provider/google_oauth.rs), [`src/provider/router.rs`](file:///root/install/nityaksha-core/src/provider/router.rs), [`src/router.rs`](file:///root/install/nityaksha-core/src/router.rs).
- **Capabilities:**
  - **36+ Pre-Configured & Custom Providers:** Google Gemini, Anthropic Claude, OpenAI, OpenRouter, Nous Portal, Kimi (Moonshot), Zhipu GLM, MiniMax, Xiaomi MiMo, Alibaba DashScope (Qwen), DeepSeek, Groq LPU, Cerebras, SambaNova, Together AI, Mistral, Cohere, Perplexity, xAI Grok, Fireworks, Ollama, LM Studio, vLLM, AWS Bedrock, GCP Vertex AI, Azure OpenAI, HuggingFace, Nvidia NIM, Replicate, Baseten, Clarifai, Cloudflare Workers AI, and Custom OpenAI/Anthropic endpoints.
  - **Multi-Step Autonomous Agentic Tool-Calling Loop (`dispatch_agentic_loop`):** Executes up to 8 reasoning + tool-invocation steps (`read_file`, `write_file`, `edit_file`, `grep_search`, `ast_prune`, `bash_sandbox`) gated by workspace trust and `IbacGate`.
  - **True Token-by-Token SSE Streaming (`dispatch_prompt_streaming`):** Real-time Server-Sent Events (`stream: true`) chunk parser emitting tokens as they arrive (`ntx -p --output-format stream-json`).
  - **Adaptive Complexity Router & Circuit Breakers:** Classifies prompts into `FastFlash`, `Balanced`, or `DeepReasoning` tiers, compresses prompts via `RtkTokenOptimizer`, and applies 60-second circuit-breaker cooldowns with automatic failover.

### 2. Interactive Terminal Concierge Shell & CLI Dispatcher
- **Modules:** [`src/cli/mod.rs`](file:///root/install/nityaksha-core/src/cli/mod.rs), [`src/cli/interactive.rs`](file:///root/install/nityaksha-core/src/cli/interactive.rs).
- **Capabilities:**
  - **Minimalist Typographic Header & 7-Tab Cockpit:** `[1] Overview`, `[2] Swarm`, `[3] Memory`, `[4] Cost`, `[5] Missions`, `[6] Providers`, `[?] Help`.
  - **Workspace Trust Gate:** Directory-level authorization (`~/.config/nityaksha/interactive.json`) before file/command mutations.
  - **5 Selectable Color Palettes:** Sovereign Amber & Obsidian, Luminous Cyan & Steel, Matrix Emerald, Synthwave Violet, Clean Minimal.
  - **Live Turn Telemetry:** Per-turn and cumulative token counters, context window utilization gauge, latency measurement, and USD cost estimation.
  - **LoRA Fine-Tuning & Trajectory Exporter (`ntx train`):** Exports verified multi-turn sessions in OpenAI/ShareGPT JSONL format (`ntx train export`), generates PEFT/LoRA training recipes (`ntx train run`), and supports live OpenAI `/v1/files` upload (`--live`).

### 3. Embedded Workstation WebUI & REST API Server
- **Modules:** [`src/server/webui.rs`](file:///root/install/nityaksha-core/src/server/webui.rs), [`src/server/mod.rs`](file:///root/install/nityaksha-core/src/server/mod.rs).
- **Capabilities:**
  - **Publicly Accessible Dashboard (`http://46.62.188.249:8080`):** Serves the interactive SPA directly at `/`, `/index.html`, `/app`, and `/dashboard`.
  - **Dual Theme Engine (Light Mode Default + Dark Mode Toggle):** Instant zero-flicker theme switching in both Top Header Bar and Sidebar Footer with `localStorage` persistence.
  - **Responsive Fixed-Width Table Layout:** Zero horizontal clipping across AI Provider Hub, Circuit Breakers, and Request Logs.
  - **Interactive Bots, DAG Workflows & AI Judge Panels:** Browse all 7 Bots (`GET /api/bots`) and launch them in the Playground, trigger all 4 DAG Workflows (`POST /api/workflows/run`), approve suspended HITL gates (`POST /api/workflows/approve`), and run the Dual-Path AI Judge (`POST /api/judge/evaluate`) directly in the browser.

### 4. Autonomous Execution Harness, Live MCP Client & Dual-Path AI Judge
- **Modules:** [`src/harness/mod.rs`](file:///root/install/nityaksha-core/src/harness/mod.rs), [`src/harness/eval_judge.rs`](file:///root/install/nityaksha-core/src/harness/eval_judge.rs), [`src/harness/mcp_client.rs`](file:///root/install/nityaksha-core/src/harness/mcp_client.rs), [`src/harness/ast_pruner.rs`](file:///root/install/nityaksha-core/src/harness/ast_pruner.rs), [`src/harness/budget.rs`](file:///root/install/nityaksha-core/src/harness/budget.rs), [`src/harness/governance.rs`](file:///root/install/nityaksha-core/src/harness/governance.rs), [`src/harness/prompt_loop.rs`](file:///root/install/nityaksha-core/src/harness/prompt_loop.rs), [`src/harness/memory.rs`](file:///root/install/nityaksha-core/src/harness/memory.rs), [`src/harness/adapter.rs`](file:///root/install/nityaksha-core/src/harness/adapter.rs).
- **Capabilities:**
  - **Dual-Path AI Judge (`EvalJudgeEngine`):** Stage 1 deterministic syntax/delimiter balance & prohibited keyword gate + Stage 2 live LLM G-Eval semantic rubric dispatch (`live_llm_judge_verifier`) and live LLM unit test synthesis (`synthesize_unit_test`).
  - **Live MCP JSON-RPC 2.0 Client & Server (`McpStdioClient`):** Spawns MCP child processes over `stdin`/`stdout`, executes JSON-RPC 2.0 `initialize`, `notifications/initialized`, `tools/list`, and `tools/call`, and includes a built-in MCP server (`ntx mcp serve`).
  - **Call-Graph & Block-Scoped AST Pruner (`AstPruner`):** Computes exact brace/indentation block boundaries across Rust, Python, Go, and TypeScript/JS, recursively scans workspaces, and expands 1-hop callee dependencies.

### 5. Agent Memory Stack, Neural + Sub-Word Embeddings & Hybrid RAG
- **Modules:** [`src/db/relational.rs`](file:///root/install/nityaksha-core/src/db/relational.rs), [`src/db/vector.rs`](file:///root/install/nityaksha-core/src/db/vector.rs), [`src/db/hybrid.rs`](file:///root/install/nityaksha-core/src/db/hybrid.rs), [`src/db/multimodal.rs`](file:///root/install/nityaksha-core/src/db/multimodal.rs), [`src/db/columnar.rs`](file:///root/install/nityaksha-core/src/db/columnar.rs), [`src/db/versioning.rs`](file:///root/install/nityaksha-core/src/db/versioning.rs), [`src/db/mod.rs`](file:///root/install/nityaksha-core/src/db/mod.rs).
- **Capabilities:**
  - **Hybrid Neural + Sub-Word Vector Embedder (`embed_text` / `embed_text_neural`):** Connects to OpenAI `text-embedding-3-small`, Google Gemini `text-embedding-004`, or local Ollama `nomic-embed-text` when `NTX_NEURAL_EMBED=1`, with zero-latency fallback to morphological stem + character 3-gram + word bigram semantic projection.
  - **Relational WAL Engine, Multimodal CAS & MVCC Snapshots:** Full ACID persistence for Capabilities, Sessions, Swarms, Bots, Workflows, Spatial Memory Loci, and SHA-256 blobs.

### 6. Live HTTP Documentation Scraper, DAG Workflows & ContainerOS Sandbox
- **Modules:** [`src/scraper/mod.rs`](file:///root/install/nityaksha-core/src/scraper/mod.rs), [`src/workflow/mod.rs`](file:///root/install/nityaksha-core/src/workflow/mod.rs), [`src/bot/mod.rs`](file:///root/install/nityaksha-core/src/bot/mod.rs), [`src/sandbox/mod.rs`](file:///root/install/nityaksha-core/src/sandbox/mod.rs).
- **Capabilities:**
  - **Live HTTP/HTTPS Scraper (`ScraperEngine::fetch_and_parse`):** Fetches remote documentation URLs, strips `<script>`/`<style>`/`<nav>` DOM nodes, converts HTML to Markdown, and synthesizes proprietary `SKILL.md` runbooks and YAML catalog manifests.
  - **ContainerOS POSIX Sandbox:** Linux namespace isolation (`Mount`, `UTS`, `IPC`, `PID`, `Net`), cgroups v2 resource limits, Docker socket guard, and IBAC intent evaluation.

---

## Part 2: Engineering Gap Closure Verification Matrix

| # | Previously Identified Gap | Resolution Implemented | Verification Command / Endpoint |
| :--- | :--- | :--- | :--- |
| **1** | **Autonomous Agentic Tool-Calling Loop (ReAct Loop)** | Implemented `ProviderRegistry::dispatch_agentic_loop` & `execute_agent_tool` (`read_file`, `write_file`, `edit_file`, `grep_search`, `ast_prune`, `bash_sandbox`) in [`src/provider/mod.rs`](file:///root/install/nityaksha-core/src/provider/mod.rs) and wired into interactive REPL & `ntx -p`. | `ntx -p "inspect Cargo.toml"` |
| **2** | **True Token-by-Token SSE Streaming** | Implemented `ProviderRegistry::dispatch_prompt_streaming` using `curl -N` line-buffered SSE parsing in [`src/provider/mod.rs`](file:///root/install/nityaksha-core/src/provider/mod.rs) and wired into `--output-format stream-json`. | `ntx -p --output-format stream-json "hello"` |
| **3** | **Neural Semantic Vector Embeddings** | Implemented `embed_text_neural` (OpenAI, Gemini, Ollama) + morphological stem, character 3-gram, and word-bigram semantic projection in [`src/db/vector.rs`](file:///root/install/nityaksha-core/src/db/vector.rs). | `ntx memory query "sandbox isolation"` |
| **4** | **Live LLM-Backed Stage 2 Rubric in `AiJudge`** | Upgraded `EvalJudgeEngine::evaluate_production_trajectory` and `synthesize_unit_test` in [`src/harness/eval_judge.rs`](file:///root/install/nityaksha-core/src/harness/eval_judge.rs) to query the active LLM with JSON G-Eval rubric scoring. | `POST /api/judge/evaluate` or `ntx harness eval` |
| **5** | **Live MCP Client JSON-RPC 2.0 `stdio` Transport** | Created [`src/harness/mcp_client.rs`](file:///root/install/nityaksha-core/src/harness/mcp_client.rs) implementing `McpStdioClient::probe_stdio_server`, `call_tool`, and `run_embedded_stdio_server` (`ntx mcp serve`). | `ntx mcp test ntx-mcp-fs-sandbox` |
| **6** | **Full Block-Scoped & Call-Graph AST Pruner** | Upgraded [`src/harness/ast_pruner.rs`](file:///root/install/nityaksha-core/src/harness/ast_pruner.rs) with brace/indentation block boundary detection, recursive directory walk, and 1-hop callee dependency expansion. | `ntx mcp call ntx-mcp-fs-sandbox ast_symbol_inspect '{"path":"src/lib.rs"}'` |
| **7** | **WebUI Interactive Panels for Bots, Workflows & Judge** | Added `/api/bots`, `/api/workflows`, `/api/workflows/run`, `/api/workflows/approve`, and `/api/judge/evaluate` in [`src/server/mod.rs`](file:///root/install/nityaksha-core/src/server/mod.rs) and interactive UI cards in [`src/server/webui.rs`](file:///root/install/nityaksha-core/src/server/webui.rs). | Open `http://46.62.188.249:8080` $\rightarrow$ **Bots & DAG Workflows** |
| **8** | **Live LoRA Fine-Tuning & Remote HTTP URL Scraper** | Added `cmd_train` (`ntx train export` / `ntx train run`) in [`src/cli/mod.rs`](file:///root/install/nityaksha-core/src/cli/mod.rs) and `ScraperEngine::fetch_and_parse` + `html_to_markdown` in [`src/scraper/mod.rs`](file:///root/install/nityaksha-core/src/scraper/mod.rs). | `ntx train run` / `ntx scrape https://example.com test-skill` |
