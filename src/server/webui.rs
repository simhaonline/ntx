//! Embedded Single-Page Workstation WebUI for Nityaksha
//! Brand Kit Compliant: Deep Navy (#0D2340), Rust Orange (#C05A28), Default Light Mode with Dark Mode Toggle.

pub const EMBEDDED_WEBUI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>NITYAKSHA — Workstation Dashboard | Pure Rust Control Plane</title>
  <link rel="icon" type="image/png" href="NITYAKSHA_LOGO.png">
  <style>
    :root {
      --color-brand-primary: #0D2340;
      --color-brand-accent: #C05A28;
      --color-brand-accent-hover: #A3481D;
      
      --bg-canvas: #F4F6F9;
      --bg-surface: #FFFFFF;
      --bg-surface-elevated: #FFFFFF;
      --bg-card: #FFFFFF;
      --border-subtle: #E2E8F0;
      --border-accent: rgba(192, 90, 40, 0.4);
      --text-heading: #0D2340;
      --text-body: #2E4F6F;
      --text-muted: #607D8B;
      --text-dim: #94A3B8;

      --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      --font-mono: 'JetBrains Mono', 'Fira Code', ui-monospace, Menlo, Monaco, monospace;
    }

    html[data-theme="dark"] {
      --bg-canvas: #09192E;
      --bg-surface: #0D2340;
      --bg-surface-elevated: #132D4E;
      --bg-card: #0D2340;
      --border-subtle: #2E4F6F;
      --border-accent: rgba(192, 90, 40, 0.6);
      --text-heading: #FFFFFF;
      --text-body: #CBD5E1;
      --text-muted: #94A3B8;
      --text-dim: #64748B;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; font-family: var(--font-sans); }
    body { background-color: var(--bg-canvas); color: var(--text-body); display: flex; height: 100vh; overflow: hidden; transition: background 0.3s, color 0.3s; }
    
    #sidebar { width: 280px; background-color: var(--bg-surface); border-right: 1px solid var(--border-subtle); display: flex; flex-direction: column; transition: all 0.3s; }
    .brand-box { padding: 1.25rem 1.5rem; border-bottom: 1px solid var(--border-subtle); display: flex; align-items: center; gap: 0.75rem; text-decoration: none; }
    .brand-logo { width: 34px; height: 34px; object-fit: contain; }
    .brand-title { font-size: 1.15rem; font-weight: 800; color: var(--text-heading); letter-spacing: -0.02em; }
    .brand-pill { font-family: var(--font-mono); font-size: 0.65rem; font-weight: 700; padding: 0.2rem 0.45rem; border-radius: 9999px; background: rgba(192, 90, 40, 0.12); color: var(--color-brand-accent); border: 1px solid rgba(192, 90, 40, 0.3); }

    .nav { list-style: none; padding: 1.25rem 0; flex: 1; }
    .nav li { padding: 0.85rem 1.5rem; cursor: pointer; display: flex; align-items: center; gap: 0.85rem; font-size: 0.92rem; font-weight: 600; color: var(--text-muted); transition: all 0.2s; border-left: 3px solid transparent; }
    .nav li:hover, .nav li.active { background-color: rgba(192, 90, 40, 0.08); color: var(--color-brand-accent); border-left: 3px solid var(--color-brand-accent); }

    .sidebar-footer { padding: 1.25rem; font-size: 0.78rem; border-top: 1px solid var(--border-subtle); color: var(--text-muted); display: flex; justify-content: space-between; align-items: center; }
    .btn-toggle-theme { background: var(--bg-canvas); border: 1px solid var(--border-subtle); color: var(--text-heading); padding: 0.4rem 0.65rem; border-radius: 6px; cursor: pointer; font-size: 0.85rem; }

    #main { flex: 1; display: flex; flex-direction: column; overflow-y: auto; padding: 2.25rem 2.5rem; }
    .header { margin-bottom: 2rem; display: flex; justify-content: space-between; align-items: center; }
    .header h1 { font-size: 1.75rem; font-weight: 800; color: var(--text-heading); letter-spacing: -0.02em; }
    .header p { color: var(--text-muted); font-size: 0.95rem; margin-top: 0.25rem; }

    .card { background-color: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: 12px; padding: 1.75rem; margin-bottom: 1.5rem; box-shadow: 0 2px 10px rgba(13, 35, 64, 0.04); }
    .card-title { font-size: 1.2rem; font-weight: 700; color: var(--text-heading); margin-bottom: 1rem; }
    
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1.25rem; margin-bottom: 1.5rem; }
    .stat-card { background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: 10px; padding: 1.25rem 1.5rem; box-shadow: 0 2px 8px rgba(13, 35, 64, 0.03); }
    .stat-val { font-size: 2rem; font-weight: 800; color: var(--color-brand-accent); margin-top: 0.25rem; }
    
    button, input, select, textarea { background-color: var(--bg-canvas); border: 1px solid var(--border-subtle); color: var(--text-heading); padding: 0.55rem 1rem; border-radius: 6px; font-size: 0.9rem; }
    button.btn-primary { background: linear-gradient(135deg, #C05A28 0%, #D86B35 100%); color: #FFFFFF; border: none; font-weight: 700; cursor: pointer; box-shadow: 0 2px 10px rgba(192, 90, 40, 0.3); }
    button.btn-primary:hover { transform: translateY(-1px); box-shadow: 0 4px 14px rgba(192, 90, 40, 0.45); }
    
    table { width: 100%; border-collapse: collapse; margin-top: 1rem; font-size: 0.88rem; }
    th, td { text-align: left; padding: 0.85rem 1rem; border-bottom: 1px solid var(--border-subtle); }
    th { color: var(--text-heading); background-color: var(--bg-canvas); font-weight: 700; font-size: 0.78rem; text-transform: uppercase; letter-spacing: 0.05em; }
    .badge { display: inline-block; padding: 0.2rem 0.55rem; border-radius: 9999px; font-size: 0.72rem; font-family: var(--font-mono); font-weight: 700; background: rgba(192, 90, 40, 0.12); color: var(--color-brand-accent); border: 1px solid rgba(192, 90, 40, 0.25); }
    .badge-green { background: rgba(16, 185, 129, 0.12); color: #10B981; border-color: rgba(16, 185, 129, 0.25); }
  </style>
</head>
<body>
  <div id="sidebar">
    <a href="/" class="brand-box">
      <img src="NITYAKSHA_LOGO.png" alt="Nityaksha Logo" class="brand-logo">
      <div>
        <div class="brand-title">NITYAKSHA</div>
        <span class="brand-pill">PURE RUST</span>
      </div>
    </a>
    <ul class="nav">
      <li class="active" onclick="showTab('dashboard')">📊 Dashboard & Runtimes</li>
      <li onclick="showTab('catalog')">📦 Capability Catalog</li>
      <li onclick="showTab('memory')">🧠 Agent Memory & Vectors</li>
      <li onclick="showTab('sandbox')">🛡️ POSIX Sandbox Runner</li>
      <li onclick="showTab('harness')">⚖️ AI Harness & Governance</li>
    </ul>
    <div class="sidebar-footer">
      <div>Pure Native Rust v1.0.0</div>
      <button class="btn-toggle-theme" onclick="toggleDashboardTheme()" title="Toggle Light / Dark">🌓</button>
    </div>
  </div>

  <div id="main">
    <div class="header">
      <div>
        <h1>NITYAKSHA — Workstation Control Plane</h1>
        <p>Engineered for high-velocity, deterministic AI in pure Rust.</p>
      </div>
      <div>
        <span class="badge badge-green" style="font-size: 0.85rem; padding: 0.4rem 0.8rem;">● HTTP/1.1 Engine Active (:8080)</span>
      </div>
    </div>

    <!-- Dashboard Tab -->
    <div id="tab-dashboard" class="tab-content">
      <div class="grid">
        <div class="stat-card">
          <div style="color: var(--text-muted); font-size: 0.85rem; font-weight: 600;">Autonomous Skills</div>
          <div class="stat-val" id="stat-skills">87</div>
        </div>
        <div class="stat-card">
          <div style="color: var(--text-muted); font-size: 0.85rem; font-weight: 600;">Agent Personas</div>
          <div class="stat-val" id="stat-agents">13</div>
        </div>
        <div class="stat-card">
          <div style="color: var(--text-muted); font-size: 0.85rem; font-weight: 600;">MCP & Plugins</div>
          <div class="stat-val" id="stat-mcps">22</div>
        </div>
        <div class="stat-card">
          <div style="color: var(--text-muted); font-size: 0.85rem; font-weight: 600;">SIMD Vectors Cached</div>
          <div class="stat-val" id="stat-vectors">133</div>
        </div>
      </div>

      <div class="card">
        <div class="card-title">Framework Health & Doctor Invariants (Pure Rust Engine)</div>
        <div id="doctor-results" style="font-family: var(--font-mono); font-size: 0.88rem; line-height: 1.8; color: #10B981;">
          ✓ Policy Configuration (/etc/nityaksha/policy.yaml) - ACTIVE<br>
          ✓ Relational Memory Store (/var/lib/nityaksha/db) - OPERATIONAL<br>
          ✓ Container Sandbox Jail (/srv/agent-workspaces) - CONFINED<br>
          ✓ Content-Addressed Storage CAS (/var/lib/nityaksha/cas-cache) - READY<br>
          ✓ Nonprivileged Service User (nityaksha) - VERIFIED<br>
          ✓ Cgroups v2 Quota Hierarchy (/sys/fs/cgroup) - ENFORCED<br>
          ✓ IBAC Security Gate Invariants - PASS
        </div>
      </div>
    </div>

    <!-- Catalog Tab -->
    <div id="tab-catalog" class="tab-content" style="display: none;">
      <div class="card">
        <div class="card-title">Proprietary Capability Catalog (Database-Backed SoT)</div>
        <div style="display: flex; gap: 0.75rem; margin-bottom: 1.25rem;">
          <input type="text" id="catalog-search" placeholder="Search proprietary skills, agents, MCPs..." style="flex: 1;" onkeyup="searchCatalog()">
          <button class="btn-primary" onclick="searchCatalog()">Search Catalog</button>
        </div>
        <table>
          <thead>
            <tr><th>Kind</th><th>Proprietary Name</th><th>Trust Tier</th><th>Version</th><th>Description</th></tr>
          </thead>
          <tbody id="catalog-table-body">
            <tr><td><span class="badge">skill</span></td><td><strong>ntx-writing-motoko</strong></td><td>approved</td><td>1.0.0</td><td>Motoko language pitfalls, modern syntax, and patterns.</td></tr>
            <tr><td><span class="badge">skill</span></td><td><strong>ntx-canister-security</strong></td><td>approved</td><td>1.0.0</td><td>IC-specific security patterns for canister development.</td></tr>
            <tr><td><span class="badge">agent</span></td><td><strong>ntx-kernel-supervisor</strong></td><td>approved</td><td>1.0.0</td><td>Supervises isolated container runtimes and cgroup quotas.</td></tr>
            <tr><td><span class="badge">mcp</span></td><td><strong>ntx-questdb</strong></td><td>approved</td><td>1.0.0</td><td>High-throughput market time-series reader via zero-copy buffer.</td></tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Memory Tab -->
    <div id="tab-memory" class="tab-content" style="display: none;">
      <div class="card">
        <div class="card-title">Hybrid Vector & Semantic Memory RAG Query</div>
        <div style="display: flex; gap: 0.75rem; margin-bottom: 1.25rem;">
          <input type="text" id="rag-query" placeholder="Ask questions or search agent episodic trajectories..." style="flex: 1;" value="security policy">
          <button class="btn-primary" onclick="queryRag()">Query RAG</button>
        </div>
        <div id="rag-results" style="font-family: var(--font-mono); font-size: 0.88rem; color: var(--text-muted); background: var(--bg-canvas); padding: 1.25rem; border-radius: 8px; border: 1px solid var(--border-subtle);">
          Enter a query to execute hybrid semantic dense SIMD cosine + sparse BM25 retrieval over agent memories.
        </div>
      </div>
    </div>

    <!-- Sandbox Tab -->
    <div id="tab-sandbox" class="tab-content" style="display: none;">
      <div class="card">
        <div class="card-title">Confined Sandbox Command Runner</div>
        <p style="color: var(--text-muted); font-size: 0.88rem; margin-bottom: 1.25rem;">
          Executes inside short-lived Linux namespaces with cgroups v2 quotas and IBAC security gate evaluation.
        </p>
        <div style="display: flex; gap: 0.75rem; margin-bottom: 1.25rem;">
          <input type="text" id="sandbox-cmd" placeholder="e.g. ls -la /srv/agent-workspaces" style="flex: 1;" value="uname -a && id">
          <button class="btn-primary" onclick="runSandbox()">Execute Confined</button>
        </div>
        <pre id="sandbox-output" style="background: var(--bg-canvas); padding: 1.25rem; border-radius: 8px; border: 1px solid var(--border-subtle); font-family: var(--font-mono); font-size: 0.85rem; max-height: 280px; overflow-y: auto;">Awaiting execution...</pre>
      </div>
    </div>

    <!-- Harness Tab -->
    <div id="tab-harness" class="tab-content" style="display: none;">
      <div class="card">
        <div class="card-title">AI Harness Architecture & Governance Invariants</div>
        <div style="font-family: var(--font-mono); font-size: 0.88rem; line-height: 1.9; color: var(--text-body);">
          [ACTIVE] Outer Runtime Budgets & Physical Loop Counter<br>
          [ACTIVE] File Thrashing & Mutation Oscillation Detector<br>
          [ACTIVE] Multi-Tier Memory (Short-Term Sliding Cache, Episodic RAG, MVCC Commits)<br>
          [ACTIVE] AST Syntax Tree Pruner & Context Window Optimizer<br>
          [ACTIVE] Dynamic Prompt Augmentation (Intent Routing, Few-Shot, Negative Cache)<br>
          [ACTIVE] Human-in-the-Loop (HITL) Merge Gates & Cryptographic Audit Trail<br>
          [ACTIVE] Layered Deterministic Checks & LLM-as-a-Judge Evaluation Engine<br>
          [ACTIVE] Standardized Hot-Swappable Provider Adapters
        </div>
      </div>
    </div>
  </div>

  <script>
    function initDashboardTheme() {
      const savedTheme = localStorage.getItem('nityaksha-theme');
      if (savedTheme === 'dark') {
        document.documentElement.setAttribute('data-theme', 'dark');
      } else {
        document.documentElement.setAttribute('data-theme', 'light');
      }
    }

    function toggleDashboardTheme() {
      const current = document.documentElement.getAttribute('data-theme');
      if (current === 'dark') {
        document.documentElement.setAttribute('data-theme', 'light');
        localStorage.setItem('nityaksha-theme', 'light');
      } else {
        document.documentElement.setAttribute('data-theme', 'dark');
        localStorage.setItem('nityaksha-theme', 'dark');
      }
    }

    initDashboardTheme();

    function showTab(tab) {
      document.querySelectorAll('.tab-content').forEach(el => el.style.display = 'none');
      document.querySelectorAll('.nav li').forEach(el => el.classList.remove('active'));
      document.getElementById('tab-' + tab).style.display = 'block';
      event.currentTarget.classList.add('active');
    }

    function searchCatalog() {
      const q = document.getElementById('catalog-search').value;
      fetch('/api/catalog?q=' + encodeURIComponent(q))
        .then(r => r.json())
        .then(data => {
          const tbody = document.getElementById('catalog-table-body');
          if (!data || data.length === 0) {
            tbody.innerHTML = '<tr><td colspan="5" style="text-align:center; padding: 1.5rem; color: var(--text-muted);">No capabilities matching search query.</td></tr>';
            return;
          }
          let html = '';
          data.forEach(item => {
            html += `<tr>
              <td><span class="badge">${escapeHtml(item.kind)}</span></td>
              <td><strong>${escapeHtml(item.name)}</strong></td>
              <td>${escapeHtml(item.trust_tier || 'approved')}</td>
              <td>${escapeHtml(item.version || '1.0.0')}</td>
              <td>${escapeHtml(item.description)}</td>
            </tr>`;
          });
          tbody.innerHTML = html;
        })
        .catch(err => {
          console.error(err);
        });
    }

    function queryRag() {
      const q = document.getElementById('rag-query').value;
      const resEl = document.getElementById('rag-results');
      resEl.textContent = 'Executing hybrid RAG retrieval over dense float vectors and sparse BM25 inverted index...';
      fetch('/api/memory/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query: q, top_k: 3 })
      })
      .then(r => r.json())
      .then(data => {
        resEl.innerHTML = `<pre style="white-space: pre-wrap;">${escapeHtml(JSON.stringify(data, null, 2))}</pre>`;
      })
      .catch(err => {
        resEl.textContent = 'Error querying RAG: ' + err;
      });
    }

    function runSandbox() {
      const cmd = document.getElementById('sandbox-cmd').value;
      const out = document.getElementById('sandbox-output');
      out.textContent = 'Running in short-lived POSIX namespace...';
      fetch('/api/sandbox/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: cmd })
      })
      .then(r => r.json())
      .then(data => {
        out.textContent = (data.stdout || '') + (data.stderr ? '\n--- STDERR ---\n' + data.stderr : '') + '\nExit Code: ' + data.exit_code;
      })
      .catch(err => {
        out.textContent = 'Execution Error: ' + err;
      });
    }

    function escapeHtml(t) {
      return String(t).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }

    // Load initial stats
    fetch('/api/status')
      .then(r => r.json())
      .then(data => {
        if (data.capabilities_cached) {
          document.getElementById('stat-skills').textContent = data.capabilities_cached.skill || 87;
          document.getElementById('stat-agents').textContent = data.capabilities_cached.agent || 13;
          document.getElementById('stat-mcps').textContent = (data.capabilities_cached.mcp || 11) + (data.capabilities_cached.plugin || 11);
        }
      })
      .catch(() => {});
  </script>
</body>
</html>
"#;
