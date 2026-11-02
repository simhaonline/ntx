//! Embedded Single-Page Workstation WebUI for Nityaksha

pub const EMBEDDED_WEBUI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Nityaksha AI Agent Control Plane (Pure Rust)</title>
  <style>
    :root {
      --bg: #0d1117; --card-bg: #161b22; --border: #30363d;
      --text: #c9d1d9; --text-bright: #f0f6fc; --accent: #58a6ff;
      --accent-green: #2ea043; --accent-red: #da3633; --accent-purple: #bc8cff;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace; }
    body { background-color: var(--bg); color: var(--text); display: flex; height: 100vh; overflow: hidden; }
    #sidebar { width: 280px; background-color: var(--card-bg); border-right: 1px solid var(--border); display: flex; flex-direction: column; }
    .brand { padding: 1.25rem; font-size: 1.1rem; font-weight: bold; color: var(--accent); border-bottom: 1px solid var(--border); display: flex; align-items: center; gap: 0.5rem; }
    .nav { list-style: none; padding: 1rem 0; flex: 1; }
    .nav li { padding: 0.75rem 1.25rem; cursor: pointer; display: flex; align-items: center; gap: 0.75rem; transition: background 0.2s; }
    .nav li:hover, .nav li.active { background-color: rgba(88, 166, 255, 0.1); color: var(--accent); border-left: 3px solid var(--accent); }
    #main { flex: 1; display: flex; flex-direction: column; overflow-y: auto; padding: 2rem; }
    .header { margin-bottom: 2rem; display: flex; justify-content: space-between; align-items: center; }
    .card { background-color: var(--card-bg); border: 1px solid var(--border); border-radius: 8px; padding: 1.5rem; margin-bottom: 1.5rem; }
    .card-title { font-size: 1.2rem; color: var(--text-bright); margin-bottom: 1rem; }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1rem; margin-bottom: 1.5rem; }
    .stat-card { background: rgba(255, 255, 255, 0.02); border: 1px solid var(--border); border-radius: 6px; padding: 1rem; }
    .stat-val { font-size: 1.8rem; font-weight: bold; color: var(--accent); margin-top: 0.25rem; }
    button, input, select, textarea { background-color: #0d1117; border: 1px solid var(--border); color: var(--text-bright); padding: 0.5rem 1rem; border-radius: 6px; font-size: 0.9rem; }
    button { background-color: #21262d; cursor: pointer; font-weight: bold; transition: 0.2s; }
    button:hover { background-color: #30363d; border-color: #8b949e; }
    table { width: 100%; border-collapse: collapse; margin-top: 1rem; font-size: 0.85rem; }
    th, td { text-align: left; padding: 0.75rem; border-bottom: 1px solid var(--border); }
    th { color: var(--text-bright); background-color: rgba(255, 255, 255, 0.02); }
    .badge { display: inline-block; padding: 0.2rem 0.5rem; border-radius: 12px; font-size: 0.75rem; font-weight: bold; background: rgba(88, 166, 255, 0.2); color: var(--accent); }
  </style>
</head>
<body>
  <div id="sidebar">
    <div class="brand">⬡ Nityaksha Control Plane (Rust)</div>
    <ul class="nav">
      <li class="active" onclick="showTab('dashboard')">📊 Dashboard & Runtimes</li>
      <li onclick="showTab('catalog')">📦 Capability Catalog</li>
      <li onclick="showTab('memory')">🧠 Agent Memory & RAG</li>
      <li onclick="showTab('sandbox')">🛡️ Container Sandbox</li>
    </ul>
    <div style="padding: 1rem; font-size: 0.75rem; border-top: 1px solid var(--border); color: #8b949e;">
      Engine: Pure Rust (Zero Python)<br>Architecture: Zero-Clone SoT
    </div>
  </div>

  <div id="main">
    <div class="header">
      <div>
        <h1 style="color: var(--text-bright); font-size: 1.8rem;">Nityaksha Workstation Control Plane</h1>
        <p style="color: #8b949e; margin-top: 0.25rem;">Autonomous Multi-Agent Infrastructure & Pure Rust Memory Stack</p>
      </div>
      <div>
        <span class="badge" style="background: rgba(46, 160, 67, 0.2); color: var(--accent-green); font-size: 0.9rem; padding: 0.4rem 0.8rem;">● Engine Active (:8080)</span>
      </div>
    </div>

    <!-- Dashboard Tab -->
    <div id="tab-dashboard" class="tab-content">
      <div class="grid">
        <div class="stat-card">
          <div style="color: #8b949e; font-size: 0.85rem;">Active Runtimes</div>
          <div class="stat-val" id="stat-runtimes">4</div>
        </div>
        <div class="stat-card">
          <div style="color: #8b949e; font-size: 0.85rem;">Total Capabilities</div>
          <div class="stat-val" id="stat-capabilities">66</div>
        </div>
        <div class="stat-card">
          <div style="color: #8b949e; font-size: 0.85rem;">Registered Sources</div>
          <div class="stat-val" id="stat-sources">4</div>
        </div>
        <div class="stat-card">
          <div style="color: #8b949e; font-size: 0.85rem;">Memory Vectors Cached</div>
          <div class="stat-val" id="stat-vectors">1024</div>
        </div>
      </div>

      <div class="card">
        <div class="card-title">Framework Health & Doctor Invariants</div>
        <div id="doctor-results" style="font-family: monospace; font-size: 0.85rem; line-height: 1.8; color: var(--accent-green);">
          ✓ Policy Configuration (/etc/nityaksha/policy.yaml) - ACTIVE<br>
          ✓ Relational Memory Store (/var/lib/nityaksha/db) - OPERATIONAL<br>
          ✓ Container Sandbox Jail (/srv/agent-workspaces) - CONFINED<br>
          ✓ Content-Addressed Storage CAS (/var/lib/nityaksha/cas-cache) - READY<br>
          ✓ IBAC Security Gate - ENFORCED
        </div>
      </div>
    </div>

    <!-- Catalog Tab -->
    <div id="tab-catalog" class="tab-content" style="display: none;">
      <div class="card">
        <div class="card-title">Proprietary Capability Catalog (Simha Suite)</div>
        <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem;">
          <input type="text" id="catalog-search" placeholder="Search proprietary skills, agents, MCPs..." style="flex: 1;" onkeyup="searchCatalog()">
          <button onclick="searchCatalog()">Search</button>
        </div>
        <table>
          <thead>
            <tr><th>Kind</th><th>Proprietary Name</th><th>Origin Repository</th><th>Version</th><th>Description</th></tr>
          </thead>
          <tbody id="catalog-table-body">
            <tr><td><span class="badge">skill</span></td><td>ntx-buy-cycles</td><td>simhaonline/icskills</td><td>1.0.0</td><td>Buy cycles with credit card...</td></tr>
            <tr><td><span class="badge">skill</span></td><td>ntx-avoid-ai-writing</td><td>simhaonline/avoid-ai-writing</td><td>1.0.0</td><td>Audit and rewrite AI writing patterns...</td></tr>
            <tr><td><span class="badge">mcp</span></td><td>ntx-questdb</td><td>simhaonline/ntx</td><td>1.0.0</td><td>High-throughput market data reader...</td></tr>
            <tr><td><span class="badge">agent</span></td><td>ntx-market-data-auditor</td><td>internal/agent-personas</td><td>2.0.0</td><td>Tick data integrity auditor...</td></tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Memory Tab -->
    <div id="tab-memory" class="tab-content" style="display: none;">
      <div class="card">
        <div class="card-title">Hybrid Vector & Semantic Memory RAG Query</div>
        <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem;">
          <input type="text" id="rag-query" placeholder="Ask questions or search agent episodic trajectories..." style="flex: 1;">
          <button onclick="queryRag()">Query RAG</button>
        </div>
        <div id="rag-results" style="font-family: monospace; font-size: 0.85rem; color: #8b949e;">
          Enter a query to run hybrid semantic dense cosine + sparse BM25 retrieval over agent memories.
        </div>
      </div>
    </div>

    <!-- Sandbox Tab -->
    <div id="tab-sandbox" class="tab-content" style="display: none;">
      <div class="card">
        <div class="card-title">Confined Sandbox Command Runner</div>
        <p style="color: #8b949e; font-size: 0.85rem; margin-bottom: 1rem;">
          Executes inside Linux namespaces with cgroups v2 quotas and IBAC security gate evaluation.
        </p>
        <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem;">
          <input type="text" id="sandbox-cmd" placeholder="e.g. ls -la /srv/agent-workspaces" style="flex: 1;" value="uname -a && id">
          <button onclick="runSandbox()">Execute Confined</button>
        </div>
        <pre id="sandbox-output" style="background: #0d1117; padding: 1rem; border-radius: 6px; border: 1px solid var(--border); font-size: 0.85rem; max-height: 250px; overflow-y: auto;">Awaiting execution...</pre>
      </div>
    </div>
  </div>

  <script>
    function showTab(tab) {
      document.querySelectorAll('.tab-content').forEach(el => el.style.display = 'none');
      document.querySelectorAll('.nav li').forEach(el => el.classList.remove('active'));
      document.getElementById('tab-' + tab).style.display = 'block';
      event.currentTarget.classList.add('active');
    }

    async function searchCatalog() {
      const q = document.getElementById('catalog-search').value;
      try {
        const res = await fetch('/api/catalog?q=' + encodeURIComponent(q));
        const data = await res.json();
        const tbody = document.getElementById('catalog-table-body');
        tbody.innerHTML = '';
        data.forEach(item => {
          tbody.innerHTML += `<tr>
            <td><span class="badge">${item.kind}</span></td>
            <td style="color: var(--text-bright); font-weight: bold;">${item.name}</td>
            <td style="color: #8b949e;">${item.origin_repo}</td>
            <td>${item.version}</td>
            <td>${item.description.slice(0, 45)}</td>
          </tr>`;
        });
      } catch (e) {
        console.error(e);
      }
    }

    async function queryRag() {
      const q = document.getElementById('rag-query').value;
      const resEl = document.getElementById('rag-results');
      resEl.innerText = "Querying pure Rust hybrid vector engine...";
      try {
        const res = await fetch('/api/memory/query', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query: q, top_k: 3, alpha: 0.5 })
        });
        const data = await res.json();
        resEl.innerHTML = JSON.stringify(data, null, 2);
      } catch (e) {
        resEl.innerText = "Error querying memory: " + e;
      }
    }

    async function runSandbox() {
      const cmd = document.getElementById('sandbox-cmd').value;
      const outEl = document.getElementById('sandbox-output');
      outEl.innerText = "Executing confined process...";
      try {
        const res = await fetch('/api/sandbox/run', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ command: cmd, dry_run: false })
        });
        const data = await res.json();
        outEl.innerText = "Exit Code: " + data.exit_code + "\n\n" + data.stdout + "\n" + data.stderr;
      } catch (e) {
        outEl.innerText = "Error: " + e;
      }
    }
  </script>
</body>
</html>"#;
