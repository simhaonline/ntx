# NITYAKSHA (`ntx` v1.0.0) — Updated Scope of Work (As-Built) & Engineering Gap Analysis

**Repository:** `https://github.com/simhaonline/ntx.git`  
**Live Workstation Gateway:** `http://46.62.188.249:8080`  
**Total Codebase Footprint:** `17,093 LOC` across `37` core modules  
**Compiled Binary Targets:** Linux `x86_64` (`ntx`) & Windows `x86_64` (`ntx.exe`)

---

## Part 1: Updated Scope of Work (Completed & Deployed Modules)

### 1. Universal AI Provider Gateway & Multi-Model Router (`4,569 LOC`)
- **Modules:** [`src/provider/mod.rs`](file:///root/install/nityaksha-core/src/provider/mod.rs) (`3,523 LOC`), [`src/provider/google_oauth.rs`](file:///root/install/nityaksha-core/src/provider/google_oauth.rs) (`701 LOC`), [`src/provider/router.rs`](file:///root/install/nityaksha-core/src/provider/router.rs) (`345 LOC`), [`src/router.rs`](file:///root/install/nityaksha-core/src/router.rs) (`57 LOC`).
- **Delivered Capabilities:**
  - **36+ Pre-Configured & Custom Providers:** Google Gemini, Anthropic Claude, OpenAI, OpenRouter, Nous Portal, Kimi (Moonshot), Zhipu GLM, MiniMax, Xiaomi MiMo, Alibaba DashScope (Qwen), DeepSeek, Groq LPU, Cerebras, SambaNova, Together AI, Mistral, Cohere, Perplexity, xAI Grok, Fireworks, Ollama, LM Studio, vLLM, AWS Bedrock, GCP Vertex AI, Azure OpenAI, HuggingFace, Nvidia NIM, Replicate, Baseten, Clarifai, Cloudflare Workers AI, and Custom OpenAI/Anthropic endpoints.
  - **Authentication Lifecycle:** Multi-tier credential resolution (`ENV` -> `~/.config/nityaksha/providers.json` -> Google OAuth 2.0 PKCE / Device Code / ADC token store).
  - **Multi-Dimension Intent Router (`AiRouterEngine`):** Routes prompts across 6 task classes (`SimpleQuery`, `CodeGeneration`, `DeepArchitecture`, `SecurityAudit`, `DataAnalysis`, `CreativeSynthesis`) into `FastFlash`, `Balanced`, or `DeepReasoning` tiers with dynamic token budgets (`off` to `max` / 16,384 tokens) and automatic fallback chains.

### 2. Interactive Terminal Concierge Shell & CLI Dispatcher (`5,241 LOC`)
- **Modules:** [`src/cli/mod.rs`](file:///root/install/nityaksha-core/src/cli/mod.rs) (`2,884 LOC`), [`src/cli/interactive.rs`](file:///root/install/nityaksha-core/src/cli/interactive.rs) (`2,357 LOC`).
- **Delivered Capabilities:**
  - **Antigravity-Grade Minimalist Overview & 7-Tab Cockpit:** `[1] Overview`, `[2] Swarm`, `[3] Memory`, `[4] Cost`, `[5] Missions`, `[6] Providers`, `[?] Help`.
  - **Workspace Trust Gate:** Directory-level authorization (`~/.config/nityaksha/interactive.json`) before file/command execution.
  - **5 Selectable Color Palettes:** Sovereign Amber & Obsidian, Luminous Cyan & Steel, Matrix Emerald, Synthwave Violet, Clean Minimal.
  - **Live Turn Telemetry:** Per-turn and cumulative token counters, context window utilization gauge, latency measurement, and USD cost estimation.
  - **Non-Interactive & Headless Modes:** `ntx -p` (`text`, `json`, `stream-json`), `ntx doctor`, `ntx status`, `ntx review`, `ntx diff`, `ntx apply`.

### 3. Embedded Workstation WebUI & REST API Server (`2,734 LOC`)
- **Modules:** [`src/server/webui.rs`](file:///root/install/nityaksha-core/src/server/webui.rs) (`1,746 LOC`), [`src/server/mod.rs`](file:///root/install/nityaksha-core/src/server/mod.rs) (`988 LOC`).
- **Delivered Capabilities:**
  - **Publicly Accessible Dashboard (`http://46.62.188.249:8080`):** Serves the interactive 7-panel SPA directly at `/`, `/index.html`, `/app`, and `/dashboard`.
  - **Dual Theme Engine (Light Mode Default + Dark Mode Toggle):** Instant zero-flicker theme switching in both Top Header Bar and Sidebar Footer with `localStorage` persistence.
  - **Responsive Fixed-Width Table Layout:** Zero horizontal clipping across AI Provider Hub, Capability Catalog, and Memory Explorer.
  - **16 REST API Endpoints:** `/health`, `/api/status`, `/api/providers`, `/api/providers/select`, `/api/providers/config`, `/api/providers/test`, `/api/auth/status`, `/api/auth/google/url`, `/api/auth/google/callback`, `/api/catalog`, `/api/memory/query`, `/api/sandbox/run`, `/api/filter/slop`, `/api/chat`, plus cross-platform installer routes (`/install.sh`, `/install.ps1`, `/install-ios.sh`).

### 4. Autonomous AI Execution Harness & Dual-Path AI Judge (`978 LOC`)
- **Modules:** [`src/harness/mod.rs`](file:///root/install/nityaksha-core/src/harness/mod.rs), [`src/harness/eval_judge.rs`](file:///root/install/nityaksha-core/src/harness/eval_judge.rs), [`src/harness/budget.rs`](file:///root/install/nityaksha-core/src/harness/budget.rs), [`src/harness/governance.rs`](file:///root/install/nityaksha-core/src/harness/governance.rs), [`src/harness/ast_pruner.rs`](file:///root/install/nityaksha-core/src/harness/ast_pruner.rs), [`src/harness/prompt_loop.rs`](file:///root/install/nityaksha-core/src/harness/prompt_loop.rs), [`src/harness/memory.rs`](file:///root/install/nityaksha-core/src/harness/memory.rs), [`src/harness/adapter.rs`](file:///root/install/nityaksha-core/src/harness/adapter.rs).
- **Delivered Capabilities:**
  - **Dual-Path AI Judge (`AiJudge`):** Stage 1 deterministic syntax/delimiter balance & prohibited marker gate (`TODO`, `FIXME`, `unimplemented!()`, `unwrap()`) + Stage 2 weighted rubric scoring (`0.40 * Correctness + 0.35 * Security + 0.25 * Efficiency`) and pre-execution unit test synthesis.
  - **Hard Loop Breaker & Budget Governor (`RuntimeBudget`):** Enforces max iterations, wall-clock timeout, memory/CPU caps, and SHA-256 file-mutation oscillation detection.
  - **Governance & Secret Sanitizer (`GovernanceGate`):** UID/GID privilege drop, cryptographic HITL approval tokens (`NTX-HITL-...`), SHA-256 hash-chained audit ledger, and automatic API key/PEM redaction.
  - **AST Pruner (`AstPruner`):** Extracts symbol line ranges (`fn`, `struct`, `enum`, `impl`, `trait`, `def`, `class`) to inject targeted slices instead of full files.

### 5. Agent Memory Stack & Hybrid RAG Database Engine (`1,460 LOC`)
- **Modules:** [`src/db/relational.rs`](file:///root/install/nityaksha-core/src/db/relational.rs), [`src/db/vector.rs`](file:///root/install/nityaksha-core/src/db/vector.rs), [`src/db/hybrid.rs`](file:///root/install/nityaksha-core/src/db/hybrid.rs), [`src/db/multimodal.rs`](file:///root/install/nityaksha-core/src/db/multimodal.rs), [`src/db/columnar.rs`](file:///root/install/nityaksha-core/src/db/columnar.rs), [`src/db/versioning.rs`](file:///root/install/nityaksha-core/src/db/versioning.rs), [`src/db/mod.rs`](file:///root/install/nityaksha-core/src/db/mod.rs).
- **Delivered Capabilities:**
  - **Relational WAL Engine:** Persistent ACID tables for Capabilities, Agent Sessions, Swarms, Trajectories, Bots, Workflows, and Spatial Memory Palace (`Wing -> Room -> Drawer`).
  - **Dense Vector + BM25 Hybrid RAG:** Unrolled 4-lane dot-product/cosine similarity fused with sparse BM25 lexical scoring ($\alpha \cdot \text{Dense} + (1 - \alpha) \cdot \text{BM25}$).
  - **Multimodal CAS & MVCC Versioning:** Content-addressable SHA-256 blob storage and parent-hash commit snapshots for state rollback.

### 6. DAG Workflow Orchestrator, Bot Registry & POSIX Sandbox (`1,488 LOC`)
- **Modules:** [`src/workflow/mod.rs`](file:///root/install/nityaksha-core/src/workflow/mod.rs) (`868 LOC`), [`src/bot/mod.rs`](file:///root/install/nityaksha-core/src/bot/mod.rs) (`217 LOC`), [`src/sandbox/mod.rs`](file:///root/install/nityaksha-core/src/sandbox/mod.rs) (`403 LOC` across `isolation.rs`, `cgroup.rs`, `ibac.rs`).
- **Delivered Capabilities:**
  - **Kahn's Algorithm DAG Engine:** Cycle-free topological sorting, parallel ready-node resolution, conditional branching, and HITL suspension/resumption across 4 default enterprise DAG workflows.
  - **7 Pre-Configured Domain Bots:** Sovereign Concierge, Systems Code Architect, Quant & Market Microstructure, Red-Team Security Auditor, SRE Incident Responder, Anti-Slop Strategist, and Legal Compliance Specialist.
  - **ContainerOS POSIX Sandbox:** Linux namespace isolation (`Mount`, `UTS`, `IPC`, `PID`, `Net`), cgroups v2 quotas (`memory.max`, `pids.max`, `cpu.max`), Docker socket blocking, and IBAC intent gating.

---

## Part 2: Missing / Partially Stubbed Parts (Engineering Gap Analysis)

While all CLI commands, TUI views, REST API endpoints, provider adapters, and sandboxing primitives compile and run, the following **8 specific subsystems** currently rely on deterministic local heuristics, synchronous execution, or simulated stubs rather than full production implementations:

| # | Subsystem | File Location | Current Implementation Status | What Is Missing for Full Production |
| :--- | :--- | :--- | :--- | :--- |
| **1** | **Autonomous Agentic Tool-Calling Loop (ReAct Loop)** | [`src/cli/interactive.rs`](file:///root/install/nityaksha-core/src/cli/interactive.rs#L1982-L2091), [`src/provider/mod.rs`](file:///root/install/nityaksha-core/src/provider/mod.rs) | Sends prompt + 20-turn history to the active LLM and renders the markdown response in a single turn. | **Multi-step autonomous tool execution loop:** Parsing `tool_calls` (`bash`, `read_file`, `write_file`, `edit_file`, `grep`) from the LLM response, executing them inside `SandboxManager`, feeding tool outputs back to the LLM, and looping until task completion. |
| **2** | **True Token-by-Token SSE Streaming** | [`src/provider/mod.rs`](file:///root/install/nityaksha-core/src/provider/mod.rs), [`src/server/mod.rs`](file:///root/install/nityaksha-core/src/server/mod.rs) | Uses synchronous blocking HTTP requests (`curl` / blocking JSON response) and prints/returns the completed message at once. | **Server-Sent Events (`stream: true`) parser:** Real-time incremental token streaming to the terminal stdout and WebUI chat panel via SSE/WebSockets. |
| **3** | **Real Neural Vector Embeddings** | [`src/db/vector.rs`](file:///root/install/nityaksha-core/src/db/vector.rs#L210-L231) | `embed_text()` uses a deterministic SHA-256 word-hash projection into $N$-dimensional float space (works offline for exact/lexical overlap, not semantic synonyms). | **Live Embedding API / Local ONNX:** Calling OpenAI `text-embedding-3-small`, Gemini `text-embedding-004`, Ollama `/api/embeddings`, or an embedded fastembed/ONNX model for true semantic similarity. |
| **4** | **Live LLM-Backed Judge (Stage 2 Rubric Call)** | [`src/harness/eval_judge.rs`](file:///root/install/nityaksha-core/src/harness/eval_judge.rs#L53-L114) | Stage 1 syntax/prohibited-keyword check is real; Stage 2 rubric scores (`0.95 / 0.90 / 0.85`) are computed via static code heuristics rather than querying a judge model. | **Secondary Judge Model Dispatch:** Dispatching the candidate diff + rubric prompt (`ntx-prompt-judge-rubric.md`) to a secondary verifier LLM and parsing structured JSON scores. |
| **5** | **Live MCP Client JSON-RPC 2.0 Transport** | [`src/cli/mod.rs`](file:///root/install/nityaksha-core/src/cli/mod.rs#L1254-L1268) | `ntx mcp list` and `ntx mcp test` verify the catalog manifest metadata, but do not spawn the child MCP binary over `stdio` or exchange JSON-RPC `initialize` / `tools/list` packets. | **Full MCP Host Client:** Spawning child MCP server processes over `stdin`/`stdout`, performing JSON-RPC 2.0 handshake (`initialize`, `tools/list`, `tools/call`), and exposing MCP tools to the agent loop. |
| **6** | **Full Tree-Sitter / Syn AST Parser** | [`src/harness/ast_pruner.rs`](file:///root/install/nityaksha-core/src/harness/ast_pruner.rs#L30-L120) | Uses line-by-line prefix matching (`fn `, `pub fn `, `struct `, `def `, `class `) and brace-depth counting to isolate symbol line ranges. | **Full Syntax Tree Parsing:** Integrating `syn` (for Rust) or `tree-sitter` grammars for multi-language call-graph resolution, macro expansion, and type-aware pruning. |
| **7** | **WebUI Interactive Panels for Bots, Workflows & Chat** | [`src/server/webui.rs`](file:///root/install/nityaksha-core/src/server/webui.rs), [`src/server/mod.rs`](file:///root/install/nityaksha-core/src/server/mod.rs) | WebUI has 7 live tabs (Overview, Provider Hub, Google OAuth, Capability Catalog, Hybrid Memory, POSIX Sandbox, Anti-Slop). | **Interactive Chat, DAG Builder & Bot Launcher Tabs in WebUI:** Adding `/api/bots` and `/api/workflows` REST endpoints and dedicated WebUI tabs to chat with bots and visually trigger/approve DAG workflows in the browser. |
| **8** | **Real LoRA Fine-Tuning & Remote URL Crawler** | [`src/cli/mod.rs`](file:///root/install/nityaksha-core/src/cli/mod.rs), [`src/scraper/mod.rs`](file:///root/install/nityaksha-core/src/scraper/mod.rs) | `ntx train export` exports real JSONL trajectories, while `ntx train run` simulates epoch loss progress. `ScraperEngine` parses local files/strings unless fetched first. | **Live Training & HTTP Scraper:** Connecting `ntx train run` to OpenAI/Vertex/unsloth fine-tuning jobs, and adding recursive HTTP crawling + HTML-to-Markdown DOM stripping in `ScraperEngine`. |
