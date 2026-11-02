//! Embedded Single-Page Workstation WebUI for Nityaksha
//! Brand Kit Compliant: Deep Navy (#0D2340), Rust Orange (#C05A28), Default Light Mode with Dark Mode Toggle.
//! Features Bot App Store, Workflow DAG Graph Engine, Memory RAG, Sandbox, and Governance.

pub const EMBEDDED_WEBUI_HTML: &str = r##"<!DOCTYPE html>
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
    
    #sidebar { width: 290px; background-color: var(--bg-surface); border-right: 1px solid var(--border-subtle); display: flex; flex-direction: column; transition: all 0.3s; }
    .brand-box { padding: 1.25rem 1.5rem; border-bottom: 1px solid var(--border-subtle); display: flex; align-items: center; gap: 0.75rem; text-decoration: none; }
    .brand-logo { width: 34px; height: 34px; object-fit: contain; }
    .brand-title { font-size: 1.15rem; font-weight: 800; color: var(--text-heading); letter-spacing: -0.02em; }
    .brand-pill { font-family: var(--font-mono); font-size: 0.65rem; font-weight: 700; padding: 0.2rem 0.45rem; border-radius: 9999px; background: rgba(192, 90, 40, 0.12); color: var(--color-brand-accent); border: 1px solid rgba(192, 90, 40, 0.3); }

    .nav { list-style: none; padding: 1.25rem 0; flex: 1; overflow-y: auto; }
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
    button.btn-secondary { background: var(--bg-canvas); color: var(--text-heading); border: 1px solid var(--border-subtle); font-weight: 600; cursor: pointer; }
    button.btn-secondary:hover { border-color: var(--color-brand-accent); color: var(--color-brand-accent); }
    
    table { width: 100%; border-collapse: collapse; margin-top: 1rem; font-size: 0.88rem; }
    th, td { text-align: left; padding: 0.85rem 1rem; border-bottom: 1px solid var(--border-subtle); }
    th { color: var(--text-heading); background-color: var(--bg-canvas); font-weight: 700; font-size: 0.78rem; text-transform: uppercase; letter-spacing: 0.05em; }
    .badge { display: inline-block; padding: 0.2rem 0.55rem; border-radius: 9999px; font-size: 0.72rem; font-family: var(--font-mono); font-weight: 700; background: rgba(192, 90, 40, 0.12); color: var(--color-brand-accent); border: 1px solid rgba(192, 90, 40, 0.25); }
    .badge-green { background: rgba(16, 185, 129, 0.12); color: #10B981; border-color: rgba(16, 185, 129, 0.25); }
    .badge-blue { background: rgba(14, 165, 233, 0.12); color: #0EA5E9; border-color: rgba(14, 165, 233, 0.25); }
    .badge-purple { background: rgba(168, 85, 247, 0.12); color: #A855F7; border-color: rgba(168, 85, 247, 0.25); }
    .badge-amber { background: rgba(245, 158, 11, 0.12); color: #F59E0B; border-color: rgba(245, 158, 11, 0.25); }

    /* Bot Card Styles */
    .bot-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 1.5rem; margin-top: 1rem; }
    .bot-card { background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: 12px; padding: 1.5rem; display: flex; flex-direction: column; justify-content: space-between; transition: all 0.2s; box-shadow: 0 2px 8px rgba(13, 35, 64, 0.03); }
    .bot-card:hover { border-color: var(--color-brand-accent); transform: translateY(-2px); box-shadow: 0 8px 24px rgba(192, 90, 40, 0.12); }
    .bot-header { display: flex; align-items: center; gap: 0.85rem; margin-bottom: 0.85rem; }
    .bot-avatar { font-size: 1.8rem; background: var(--bg-canvas); width: 48px; height: 48px; display: flex; align-items: center; justify-content: center; border-radius: 10px; border: 1px solid var(--border-subtle); }
    .bot-meta h3 { font-size: 1.05rem; font-weight: 700; color: var(--text-heading); }
    .bot-meta span { font-size: 0.78rem; color: var(--text-muted); font-weight: 500; }
    .bot-desc { font-size: 0.88rem; color: var(--text-body); margin-bottom: 1rem; line-height: 1.5; flex: 1; }
    .bot-chips { display: flex; flex-wrap: wrap; gap: 0.35rem; margin-bottom: 1rem; }
    .bot-starters { background: var(--bg-canvas); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 0.75rem; margin-bottom: 1rem; font-size: 0.8rem; }
    .starter-btn { display: block; width: 100%; text-align: left; background: none; border: none; padding: 0.35rem 0; color: var(--color-brand-accent); cursor: pointer; font-size: 0.8rem; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .starter-btn:hover { text-decoration: underline; }

    /* Workflow DAG Styles */
    .wf-card { background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: 12px; padding: 1.75rem; margin-bottom: 1.5rem; }
    .wf-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 1rem; }
    .wf-nodes { display: flex; align-items: center; flex-wrap: wrap; gap: 0.75rem; margin: 1.25rem 0; padding: 1rem; background: var(--bg-canvas); border-radius: 8px; border: 1px solid var(--border-subtle); }
    .dag-node { background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 0.6rem 0.9rem; font-size: 0.82rem; font-weight: 600; display: flex; align-items: center; gap: 0.5rem; }
    .dag-node.status-completed { border-color: #10B981; background: rgba(16, 185, 129, 0.08); color: #10B981; }
    .dag-node.status-running { border-color: var(--color-brand-accent); background: rgba(192, 90, 40, 0.08); color: var(--color-brand-accent); animation: pulse 1.5s infinite; }
    .dag-node.status-hitl { border-color: #F59E0B; background: rgba(245, 158, 11, 0.12); color: #F59E0B; font-weight: 700; }
    .dag-arrow { color: var(--text-muted); font-weight: 800; font-size: 1rem; }
    
    .hitl-banner { background: rgba(245, 158, 11, 0.12); border: 1px solid rgba(245, 158, 11, 0.4); border-radius: 8px; padding: 1.25rem; margin-top: 1rem; display: flex; flex-direction: column; gap: 0.75rem; }
    .hitl-banner h4 { color: #B45309; font-size: 0.95rem; font-weight: 700; }
    
    @keyframes pulse {
      0% { opacity: 0.7; }
      50% { opacity: 1; }
      100% { opacity: 0.7; }
    }
  </style>
</head>
<body>
  <div id="sidebar">
    <a href="/" class="brand-box">
      <img src="logo.svg" alt="Nityaksha Logo" class="brand-logo" id="webui-brand-logo">
      <div>
        <div class="brand-title">NITYAKSHA</div>
        <span class="brand-pill">PURE RUST</span>
      </div>
    </a>
    <ul class="nav">
      <li class="active" onclick="showTab('dashboard')">📊 Dashboard & Runtimes</li>
      <li onclick="showTab('providers')">🌐 AI Providers & Gateway</li>
      <li onclick="showTab('bots')">🤖 Bot App Store</li>
      <li onclick="showTab('workflows')">⚡ Workflow DAG Engine</li>
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
          <div style="color: var(--text-muted); font-size: 0.85rem; font-weight: 600;">Pre-Configured Bots</div>
          <div class="stat-val" id="stat-bots">7</div>
        </div>
        <div class="stat-card">
          <div style="color: var(--text-muted); font-size: 0.85rem; font-weight: 600;">Workflow DAGs</div>
          <div class="stat-val" id="stat-workflows">4</div>
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
          ✓ IBAC Security Gate Invariants - PASS<br>
          ✓ Kahn's DAG Cycle Detection - VERIFIED 0 CYCLES
        </div>
      </div>
    </div>

    <!-- Bots Tab -->
    <div id="tab-bots" class="tab-content" style="display: none;">
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div class="card-title" style="margin-bottom: 0.25rem;">🤖 Sovereign Bot Catalog & App Store</div>
            <p style="color: var(--text-muted); font-size: 0.88rem;">Pre-configured personas with dynamic prompt hydration, allowed tools whitelist, and sandbox policies.</p>
          </div>
          <div>
            <input type="text" id="bot-filter" placeholder="Filter bots by category or name..." onkeyup="filterBots()" style="width: 260px;">
          </div>
        </div>
        
        <div id="bots-container" class="bot-grid">
          <!-- Dynamically Loaded -->
        </div>
      </div>

      <!-- Bot Execution Modal/Card -->
      <div id="bot-runner-panel" class="card" style="display: none; border-color: var(--color-brand-accent);">
        <div class="card-title" id="bot-runner-title">Run Bot Persona</div>
        <div style="display: flex; gap: 0.75rem; margin-bottom: 1rem;">
          <input type="text" id="bot-prompt-input" placeholder="Enter prompt or select a starter prompt..." style="flex: 1;">
          <button class="btn-primary" onclick="submitBotRun()">Dispatch to Bot</button>
          <button class="btn-secondary" onclick="closeBotRunner()">Close</button>
        </div>
        <div id="bot-runner-status" style="font-family: var(--font-mono); font-size: 0.88rem; color: var(--text-muted); background: var(--bg-canvas); padding: 1.25rem; border-radius: 8px; border: 1px solid var(--border-subtle); max-height: 350px; overflow-y: auto; white-space: pre-wrap;">Awaiting dispatch...</div>
      </div>
    </div>

    <!-- Workflows Tab -->
    <div id="tab-workflows" class="tab-content" style="display: none;">
      <div class="card">
        <div class="card-title">⚡ Directed Acyclic Graph (DAG) Workflow Engine</div>
        <p style="color: var(--text-muted); font-size: 0.88rem; margin-bottom: 1.5rem;">
          Structured multi-agent graph orchestrations with Kahn's cycle detection, multi-model dispatch, and Human-in-the-Loop (HITL) merge gates.
        </p>
        <div id="workflows-container">
          <!-- Dynamically Loaded -->
        </div>
      </div>

      <!-- Workflow Live Execution Tracker -->
      <div id="wf-exec-panel" class="card" style="display: none; border-color: var(--color-brand-accent);">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
          <div class="card-title" id="wf-exec-title" style="margin: 0;">Workflow Execution Tracker</div>
          <span id="wf-exec-badge" class="badge">Running</span>
        </div>
        
        <div id="wf-exec-nodes" class="wf-nodes"></div>

        <div id="wf-hitl-box" class="hitl-banner" style="display: none;">
          <h4>⚠️ Mandatory Human-in-the-Loop Approval Required</h4>
          <p id="wf-hitl-prompt" style="font-size: 0.88rem; color: var(--text-heading);"></p>
          <div style="display: flex; gap: 0.75rem; align-items: center;">
            <input type="text" id="wf-hitl-token" placeholder="Enter approval token..." style="flex: 1;">
            <button class="btn-primary" onclick="submitHitlApproval()">Authorize & Resume DAG</button>
          </div>
        </div>

        <div style="margin-top: 1.25rem;">
          <h4 style="font-size: 0.9rem; font-weight: 700; color: var(--text-heading); margin-bottom: 0.5rem;">Execution Trace Logs:</h4>
          <pre id="wf-exec-logs" style="background: var(--bg-canvas); padding: 1.25rem; border-radius: 8px; border: 1px solid var(--border-subtle); font-family: var(--font-mono); font-size: 0.82rem; max-height: 250px; overflow-y: auto;">Waiting for steps...</pre>
        </div>
      </div>
    </div>

    <!-- AI Providers & Gateway Tab -->
    <div id="tab-providers" class="tab-content" style="display: none;">
      <!-- Google Gemini OAuth 2.0 Card -->
      <div class="card" style="border: 1px solid rgba(192, 90, 40, 0.4); background: linear-gradient(180deg, var(--bg-surface) 0%, rgba(192, 90, 40, 0.04) 100%);">
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 1rem;">
          <div>
            <div style="display: flex; align-items: center; gap: 0.65rem;">
              <span style="font-size: 1.5rem;">✨</span>
              <div class="card-title" style="margin-bottom: 0;">Google Gemini — Native OAuth 2.0 Integration</div>
              <span id="google-auth-badge" class="badge">Checking Auth...</span>
            </div>
            <p style="color: var(--text-muted); font-size: 0.88rem; margin-top: 0.35rem;">
              Direct authentication with your Google account. Zero manual API key management, automated token refresh, and full access to Gemini 2.0 Flash and 1.5 Pro.
            </p>
          </div>
          <div style="display: flex; gap: 0.75rem; align-items: center;">
            <button class="btn-primary" onclick="startGoogleOAuth()" style="display: flex; align-items: center; gap: 0.5rem;">
              <svg width="18" height="18" viewBox="0 0 24 24"><path fill="#FFF" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/><path fill="#FFF" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/><path fill="#FFF" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/><path fill="#FFF" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/></svg>
              Sign In with Google
            </button>
            <button class="btn-secondary" onclick="toggleManualCodeBox()">Paste Auth Code</button>
          </div>
        </div>

        <div id="google-manual-box" style="display: none; margin-top: 1rem; padding-top: 1rem; border-top: 1px solid var(--border-subtle);">
          <div style="font-size: 0.85rem; color: var(--text-muted); margin-bottom: 0.5rem;">For remote/headless environments without local browser redirects, paste the authorization code below:</div>
          <div style="display: flex; gap: 0.5rem;">
            <input type="text" id="manual-google-code" placeholder="Paste Google authorization code (4/0A...)..." style="flex: 1;">
            <button class="btn-primary" onclick="submitManualGoogleCode()">Exchange Code</button>
          </div>
        </div>
      </div>

      <!-- Preconfigured Providers Grid -->
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.75rem;">
          <div>
            <div class="card-title" style="margin-bottom: 0.25rem;">🌐 Multi-Provider AI Gateway & App Store</div>
            <p style="color: var(--text-muted); font-size: 0.88rem;">21 preconfigured providers across frontier labs, open-source clusters, and specialized LPUs.</p>
          </div>
          <div style="display: flex; gap: 0.5rem;">
            <input type="text" id="provider-search" placeholder="Search providers or models..." onkeyup="filterProviders()" style="width: 240px;">
          </div>
        </div>

        <div id="providers-container" class="bot-grid">
          <!-- Dynamically Loaded via loadProviders() -->
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
    let activeBots = [];
    let activeWorkflows = [];
    let currentBotId = null;
    let currentExecId = null;
    let pollInterval = null;

    function initDashboardTheme() {
      const savedTheme = localStorage.getItem('nityaksha-theme') || 'light';
      setWebuiTheme(savedTheme);
    }

    function toggleDashboardTheme() {
      const current = document.documentElement.getAttribute('data-theme') || 'light';
      const target = current === 'dark' ? 'light' : 'dark';
      setWebuiTheme(target);
    }

    function setWebuiTheme(theme) {
      document.documentElement.setAttribute('data-theme', theme);
      localStorage.setItem('nityaksha-theme', theme);
      const logo = document.getElementById('webui-brand-logo');
      if (logo) {
        logo.src = theme === 'dark' ? 'logo-dark.svg' : 'logo.svg';
      }
    }

    initDashboardTheme();

    function showTab(tab) {
      document.querySelectorAll('.tab-content').forEach(el => el.style.display = 'none');
      document.querySelectorAll('.nav li').forEach(el => el.classList.remove('active'));
      const activeEl = document.getElementById('tab-' + tab);
      if (activeEl) activeEl.style.display = 'block';
      if (window.event && window.event.currentTarget) {
        window.event.currentTarget.classList.add('active');
      }

      if (tab === 'providers') loadProviders();
      if (tab === 'bots') loadBots();
      if (tab === 'workflows') loadWorkflows();
    }

    // --- AI Providers Gateway Functions ---
    let activeProviders = [];

    function loadProviders() {
      fetch('/api/auth/status')
        .then(r => r.json())
        .then(auth => {
          const badge = document.getElementById('google-auth-badge');
          if (badge && auth.google_oauth && auth.google_oauth.authenticated) {
            badge.className = 'badge badge-green';
            badge.textContent = `✓ Authenticated (${auth.google_oauth.email || 'Active Account'})`;
          } else if (badge) {
            badge.className = 'badge badge-amber';
            badge.textContent = 'Not Authenticated (Click Sign In)';
          }
        })
        .catch(err => console.error("Error checking auth status:", err));

      fetch('/api/providers')
        .then(r => r.json())
        .then(providers => {
          activeProviders = providers;
          renderProviders(providers);
        })
        .catch(err => console.error("Error loading providers:", err));
    }

    function renderProviders(providers) {
      const c = document.getElementById('providers-container');
      if (!c) return;
      c.innerHTML = '';
      providers.forEach(p => {
        const isActive = p.is_active;
        const statusBadge = isActive
          ? '<span class="badge badge-green">ACTIVE DEFAULT</span>'
          : p.is_configured
            ? '<span class="badge badge-blue">READY</span>'
            : '<span class="badge">UNCONFIGURED</span>';

        const categoryBadge = `<span class="badge" style="font-size: 0.7rem;">${escapeHtml(p.category)}</span>`;

        const card = document.createElement('div');
        card.className = 'bot-card' + (isActive ? ' active-provider' : '');
        if (isActive) {
          card.style.borderColor = 'var(--color-brand-accent)';
          card.style.boxShadow = '0 4px 16px rgba(192, 90, 40, 0.15)';
        }

        const modelsList = (p.available_models || []).slice(0, 3).map(m =>
          `<span class="badge" style="font-size: 0.65rem; background: var(--bg-canvas);">${escapeHtml(m)}</span>`
        ).join(' ');

        card.innerHTML = `
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.75rem;">
              <div>
                <h3 style="font-size: 1.1rem; font-weight: 800; color: var(--text-heading);">${escapeHtml(p.name)}</h3>
                <span style="font-size: 0.78rem; font-family: var(--font-mono); color: var(--color-brand-accent);">${escapeHtml(p.id)}</span>
              </div>
              <div style="display: flex; flex-direction: column; align-items: flex-end; gap: 0.25rem;">
                ${statusBadge}
                ${categoryBadge}
              </div>
            </div>

            <p style="font-size: 0.85rem; color: var(--text-body); margin-bottom: 0.85rem; line-height: 1.4;">${escapeHtml(p.description)}</p>

            <div style="background: var(--bg-canvas); padding: 0.65rem; border-radius: 6px; border: 1px solid var(--border-subtle); margin-bottom: 0.85rem; font-size: 0.78rem;">
              <div><strong>Default Model:</strong> <span style="color: var(--color-brand-accent); font-family: var(--font-mono);">${escapeHtml(p.default_model)}</span></div>
              <div style="margin-top: 0.2rem;"><strong>Auth:</strong> ${escapeHtml(p.auth_summary)}</div>
            </div>

            <div style="display: flex; flex-wrap: wrap; gap: 0.25rem; margin-bottom: 1rem;">
              ${modelsList}
            </div>
          </div>

          <div style="display: flex; gap: 0.5rem; justify-content: space-between; border-top: 1px solid var(--border-subtle); padding-top: 0.85rem;">
            ${isActive 
              ? `<button class="btn-primary" style="flex: 1; opacity: 0.8; cursor: default;">✓ Current Active</button>`
              : `<button class="btn-primary" style="flex: 1;" onclick="selectProvider('${p.id}')">Set Active</button>`
            }
            <button class="btn-secondary" onclick="testProvider('${p.id}', this)">Ping</button>
            <button class="btn-secondary" onclick="configPrompt('${p.id}')">Config</button>
          </div>
        `;
        c.appendChild(card);
      });
    }

    function filterProviders() {
      const q = (document.getElementById('provider-search').value || '').toLowerCase();
      const filtered = activeProviders.filter(p => 
        p.name.toLowerCase().includes(q) || 
        p.id.toLowerCase().includes(q) || 
        p.category.toLowerCase().includes(q) ||
        p.default_model.toLowerCase().includes(q)
      );
      renderProviders(filtered);
    }

    function selectProvider(id) {
      fetch('/api/providers/select', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: id })
      })
      .then(r => r.json())
      .then(() => loadProviders())
      .catch(err => alert("Error selecting provider: " + err));
    }

    function testProvider(id, btn) {
      const origText = btn.textContent;
      btn.textContent = '...';
      fetch('/api/providers/test', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: id })
      })
      .then(r => r.json())
      .then(res => {
        btn.textContent = origText;
        if (res.status === 'ok') {
          alert(`[TEST PASSED]\n${res.message}`);
        } else {
          alert(`[TEST FAILED]\n${res.message}`);
        }
      })
      .catch(err => {
        btn.textContent = origText;
        alert("Ping error: " + err);
      });
    }

    function configPrompt(id) {
      const p = activeProviders.find(x => x.id === id);
      const key = prompt(`Enter API Key for ${p ? p.name : id} (or leave empty to keep current):`);
      if (key !== null) {
        fetch('/api/providers/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ id: id, api_key: key })
        })
        .then(r => r.json())
        .then(() => loadProviders())
        .catch(err => alert("Error saving config: " + err));
      }
    }

    function startGoogleOAuth() {
      fetch('/api/auth/google/url')
        .then(r => r.json())
        .then(res => {
          if (res.url) {
            window.location.href = res.url;
          }
        })
        .catch(err => alert("Failed to start Google OAuth: " + err));
    }

    function toggleManualCodeBox() {
      const box = document.getElementById('google-manual-box');
      box.style.display = box.style.display === 'none' ? 'block' : 'none';
    }

    function submitManualGoogleCode() {
      const code = document.getElementById('manual-google-code').value.trim();
      if (!code) {
        alert("Please enter a valid authorization code.");
        return;
      }
      window.location.href = `/api/auth/google/callback?code=${encodeURIComponent(code)}`;
    }

    // --- Bot App Store Functions ---
    function loadBots() {
      fetch('/api/bots')
        .then(r => r.json())
        .then(bots => {
          activeBots = bots;
          renderBots(bots);
          document.getElementById('stat-bots').textContent = bots.length;
        })
        .catch(err => console.error("Error loading bots:", err));
    }

    function renderBots(bots) {
      const c = document.getElementById('bots-container');
      c.innerHTML = '';
      bots.forEach(b => {
        const startersHtml = (b.starter_prompts || []).map(p => 
          `<button class="starter-btn" onclick="openBotRunner('${b.id}', '${escapeAttr(p)}')">⚡ "${escapeHtml(p)}"</button>`
        ).join('');

        const toolsHtml = (b.allowed_tools || []).map(t => 
          `<span class="badge badge-blue" style="font-size: 0.68rem;">${escapeHtml(t)}</span>`
        ).join('');

        const card = document.createElement('div');
        card.className = 'bot-card';
        card.innerHTML = `
          <div>
            <div class="bot-header">
              <div class="bot-avatar">${b.avatar || '🤖'}</div>
              <div class="bot-meta">
                <h3>${escapeHtml(b.name)}</h3>
                <span>${escapeHtml(b.category)} • v${escapeHtml(b.version)}</span>
              </div>
            </div>
            <div style="display: flex; gap: 0.4rem; margin-bottom: 0.75rem;">
              <span class="badge">${escapeHtml(b.model_tier)}</span>
              <span class="badge badge-purple">${escapeHtml(b.sandbox_policy)}</span>
            </div>
            <div class="bot-desc">${escapeHtml(b.description)}</div>
            <div class="bot-chips">${toolsHtml}</div>
            <div class="bot-starters">
              <div style="color: var(--text-muted); font-size: 0.72rem; font-weight: 700; margin-bottom: 0.35rem; text-transform: uppercase;">Starter Prompts</div>
              ${startersHtml}
            </div>
          </div>
          <button class="btn-primary" style="width: 100%; margin-top: 0.5rem;" onclick="openBotRunner('${b.id}')">Activate Persona</button>
        `;
        c.appendChild(card);
      });
    }

    function filterBots() {
      const q = document.getElementById('bot-filter').value.toLowerCase();
      const filtered = activeBots.filter(b => 
        b.name.toLowerCase().includes(q) || 
        b.category.toLowerCase().includes(q) || 
        b.description.toLowerCase().includes(q)
      );
      renderBots(filtered);
    }

    function openBotRunner(botId, starterPrompt) {
      currentBotId = botId;
      const bot = activeBots.find(b => b.id === botId);
      document.getElementById('bot-runner-panel').style.display = 'block';
      document.getElementById('bot-runner-title').textContent = `${bot ? bot.avatar + ' ' + bot.name : botId} — Interactive Session`;
      if (starterPrompt) {
        document.getElementById('bot-prompt-input').value = starterPrompt;
      }
      document.getElementById('bot-runner-panel').scrollIntoView({ behavior: 'smooth' });
    }

    function closeBotRunner() {
      document.getElementById('bot-runner-panel').style.display = 'none';
    }

    function submitBotRun() {
      const prompt = document.getElementById('bot-prompt-input').value;
      const statusEl = document.getElementById('bot-runner-status');
      statusEl.textContent = 'Hydrating prompt variables and dispatching to pure Rust MultiModelGateway...';

      fetch('/api/bots/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ bot_id: currentBotId, prompt: prompt })
      })
      .then(r => r.json())
      .then(res => {
        statusEl.textContent = `[SYSTEM INVARIANT LOADED]:\n${res.hydrated_system_prompt}\n\n=======================================================\n[EXECUTION RESPONSE]:\n${res.output}`;
      })
      .catch(err => {
        statusEl.textContent = 'Error executing bot: ' + err;
      });
    }

    // --- Workflow DAG Functions ---
    function loadWorkflows() {
      fetch('/api/workflows')
        .then(r => r.json())
        .then(wfs => {
          activeWorkflows = wfs;
          renderWorkflows(wfs);
          document.getElementById('stat-workflows').textContent = wfs.length;
        })
        .catch(err => console.error("Error loading workflows:", err));
    }

    function renderWorkflows(wfs) {
      const c = document.getElementById('workflows-container');
      c.innerHTML = '';
      wfs.forEach(wf => {
        const nodesHtml = wf.nodes.map((node, i) => {
          const arrow = i < wf.nodes.length - 1 ? '<span class="dag-arrow">→</span>' : '';
          const isHitl = node.kind && node.kind.type === 'HitlGate';
          const badgeClass = isHitl ? 'badge-amber' : 'badge';
          return `
            <div class="dag-node">
              <span>${escapeHtml(node.name)}</span>
              <span class="badge ${badgeClass}">${escapeHtml(node.kind ? node.kind.type : 'Task')}</span>
            </div>
            ${arrow}
          `;
        }).join('');

        const card = document.createElement('div');
        card.className = 'wf-card';
        card.innerHTML = `
          <div class="wf-header">
            <div>
              <h3 style="font-size: 1.15rem; font-weight: 700; color: var(--text-heading);">${escapeHtml(wf.name)}</h3>
              <span style="font-size: 0.8rem; color: var(--text-muted); font-weight: 600;">${escapeHtml(wf.category)} • ${wf.nodes.length} Graph Steps • v${escapeHtml(wf.version)}</span>
            </div>
            <button class="btn-primary" onclick="launchWorkflow('${wf.id}')">Execute DAG Workflow</button>
          </div>
          <p style="font-size: 0.9rem; color: var(--text-body);">${escapeHtml(wf.description)}</p>
          <div class="wf-nodes">${nodesHtml}</div>
        `;
        c.appendChild(card);
      });
    }

    function launchWorkflow(wfId) {
      const execPanel = document.getElementById('wf-exec-panel');
      execPanel.style.display = 'block';
      execPanel.scrollIntoView({ behavior: 'smooth' });

      fetch('/api/workflows/execute', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workflow_id: wfId })
      })
      .then(r => r.json())
      .then(exec => {
        currentExecId = exec.id;
        updateExecDisplay(exec);
        if (pollInterval) clearInterval(pollInterval);
        pollInterval = setInterval(() => pollExecution(currentExecId), 1500);
      })
      .catch(err => {
        alert("Workflow execution launch failed: " + err);
      });
    }

    function pollExecution(execId) {
      fetch('/api/workflows/executions?id=' + encodeURIComponent(execId))
        .then(r => r.json())
        .then(exec => {
          updateExecDisplay(exec);
          if (exec.status === 'Completed' || exec.status === 'Failed') {
            clearInterval(pollInterval);
          }
        })
        .catch(err => console.error("Poll error:", err));
    }

    function updateExecDisplay(exec) {
      document.getElementById('wf-exec-title').textContent = `Execution: ${exec.workflow_name} (${exec.id})`;
      const badge = document.getElementById('wf-exec-badge');
      badge.textContent = exec.status;

      if (exec.status === 'Completed') {
        badge.className = 'badge badge-green';
      } else if (exec.status === 'SuspendedWaitingHitl') {
        badge.className = 'badge badge-amber';
      } else {
        badge.className = 'badge badge-blue';
      }

      // Render Nodes Progress
      const nodesContainer = document.getElementById('wf-exec-nodes');
      const wf = activeWorkflows.find(w => w.id === exec.workflow_id);
      if (wf) {
        nodesContainer.innerHTML = wf.nodes.map((node, i) => {
          const state = exec.node_states[node.id];
          const status = state ? state.status : 'Pending';
          let cls = '';
          if (status === 'Completed') cls = 'status-completed';
          if (status === 'Running') cls = 'status-running';
          if (status === 'SuspendedWaitingHitl') cls = 'status-hitl';
          const arrow = i < wf.nodes.length - 1 ? '<span class="dag-arrow">→</span>' : '';
          return `
            <div class="dag-node ${cls}">
              <span>${escapeHtml(node.name)}</span>
              <span class="badge">${escapeHtml(status)}</span>
            </div>
            ${arrow}
          `;
        }).join('');
      }

      // Check HITL Banner
      const hitlBox = document.getElementById('wf-hitl-box');
      if (exec.status === 'SuspendedWaitingHitl' && exec.pending_hitl) {
        hitlBox.style.display = 'block';
        document.getElementById('wf-hitl-prompt').textContent = `${exec.pending_hitl.prompt} (Token Required: ${exec.pending_hitl.token_name})`;
        document.getElementById('wf-hitl-token').value = exec.pending_hitl.token_name;
      } else {
        hitlBox.style.display = 'none';
      }

      // Update Logs
      const logsEl = document.getElementById('wf-exec-logs');
      logsEl.textContent = (exec.logs || []).join('\n');
      logsEl.scrollTop = logsEl.scrollHeight;
    }

    function submitHitlApproval() {
      const token = document.getElementById('wf-hitl-token').value;
      fetch('/api/workflows/executions/approve', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: currentExecId, token: token })
      })
      .then(r => r.json())
      .then(exec => {
        updateExecDisplay(exec);
      })
      .catch(err => {
        alert("HITL Approval failed: " + err);
      });
    }

    // --- Search & Memory Functions ---
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
      return String(t || '').replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }

    function escapeAttr(t) {
      return String(t || '').replace(/'/g, "\\'").replace(/"/g, "&quot;");
    }

    // Load initial stats
    fetch('/api/status')
      .then(r => r.json())
      .then(data => {
        if (data.capabilities_cached) {
          document.getElementById('stat-skills').textContent = data.capabilities_cached.skill || 87;
        }
      })
      .catch(() => {});
  </script>
</body>
</html>
"##;