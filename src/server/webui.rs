//! Embedded Single-Page Workstation WebUI for Nityaksha (APIPark Enterprise Inspired)
//! Pure Native Rust Control Plane with Universal AI Gateway, Multi-Model Router, and Live Playground.

pub const EMBEDDED_WEBUI_HTML: &str = r##"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>NITYAKSHA — Enterprise AI Gateway & Control Plane</title>
  <link rel="icon" type="image/png" href="/NITYAKSHA_LOGO.png">
  <script>
    (function() {
      try {
        const saved = localStorage.getItem('ntx_theme');
        if (saved === 'dark' || saved === 'light') {
          document.documentElement.setAttribute('data-theme', saved);
        } else {
          document.documentElement.setAttribute('data-theme', 'light');
        }
      } catch (e) {
        document.documentElement.setAttribute('data-theme', 'light');
      }
    })();
  </script>
  <style>
    /* ==========================================================================
       NITYAKSHA DESIGN SYSTEM TOKENS
       Default Light Mode & Toggleable Dark Mode (API Park Architecture)
       ========================================================================== */
    :root, html[data-theme="light"] {
      /* Light Mode Palette (Default) */
      --bg-canvas: #F4F6F9;              /* Clean Off-White Canvas */
      --bg-sidebar: #FFFFFF;             /* Pure White Sidebar */
      --bg-surface: #FFFFFF;             /* White Surface Panels */
      --bg-surface-elevated: #F8FAFC;    /* Elevated Soft Slate */
      --bg-card-hover: #F1F5F9;
      --border-subtle: #E2E8F0;          /* Crisp Slate Border */
      --border-strong: #CBD5E1;
      --border-accent: rgba(192, 90, 40, 0.4);
      --border-focus: #C05A28;

      --color-brand: #C05A28;            /* Rust Orange */
      --color-brand-glow: rgba(192, 90, 40, 0.25);
      --color-emerald: #059669;          /* High contrast emerald on white */
      --color-cyan: #0284C7;
      --color-amber: #D97706;
      --color-violet: #7C3AED;
      --color-rose: #E11D48;

      --text-heading: #0D2340;           /* Deep Navy (Crisp & High Contrast) */
      --text-body: #2E4F6F;              /* Slate Steel Body */
      --text-muted: #64748B;             /* Cool Slate Muted */
      --text-dim: #94A3B8;

      --card-shadow: 0 1px 3px rgba(13, 35, 64, 0.05), 0 1px 2px rgba(13, 35, 64, 0.03);
      --card-shadow-hover: 0 6px 16px rgba(13, 35, 64, 0.08);

      --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      --font-mono: 'JetBrains Mono', 'Fira Code', ui-monospace, Menlo, Monaco, monospace;
    }

    html[data-theme="dark"] {
      /* Dark Mode Palette (Toggleable) */
      --bg-canvas: #09192E;              /* Deep Navy Dark Canvas */
      --bg-sidebar: #0D2340;             /* Navy Sidebar */
      --bg-surface: #0D2340;             /* Navy Panel Surface */
      --bg-surface-elevated: #132D4E;    /* Elevated Navy */
      --bg-card-hover: #163252;
      --border-subtle: #1E3A5F;          /* Slate Steel Dark Border */
      --border-strong: #2E4F6F;
      --border-accent: rgba(192, 90, 40, 0.6);
      --border-focus: #EA580C;

      --color-brand: #EA580C;            /* Vibrant Rust Orange */
      --color-brand-glow: rgba(234, 88, 12, 0.35);
      --color-emerald: #10B981;          /* Glowing Emerald on Dark */
      --color-cyan: #0EA5E9;
      --color-amber: #F59E0B;
      --color-violet: #8B5CF6;
      --color-rose: #F43F5E;

      --text-heading: #FFFFFF;           /* Pure White */
      --text-body: #CBD5E1;              /* Soft Light Body */
      --text-muted: #94A3B8;             /* Cool Slate */
      --text-dim: #64748B;

      --card-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
      --card-shadow-hover: 0 6px 20px rgba(0, 0, 0, 0.45);
    }

    * { box-sizing: border-box; margin: 0; padding: 0; font-family: var(--font-sans); }
    body { background-color: var(--bg-canvas); color: var(--text-body); display: flex; height: 100vh; width: 100vw; overflow: hidden; font-size: 14px; }

    /* --- Sidebar --- */
    #sidebar {
      width: 250px;
      min-width: 250px;
      max-width: 250px;
      flex-shrink: 0;
      background-color: var(--bg-sidebar);
      border-right: 1px solid var(--border-subtle);
      display: flex;
      flex-direction: column;
      user-select: none;
      z-index: 20;
    }

    .brand-header {
      padding: 1.25rem 1.25rem 1rem;
      border-bottom: 1px solid var(--border-subtle);
      display: flex;
      align-items: center;
      gap: 0.75rem;
    }
    .brand-logo {
      width: 32px;
      height: 32px;
      border-radius: 8px;
      object-fit: contain;
    }
    .brand-title {
      font-size: 1.05rem;
      font-weight: 800;
      color: var(--text-heading);
      letter-spacing: -0.02em;
      line-height: 1.2;
    }
    .brand-subtitle {
      font-size: 0.7rem;
      color: var(--text-dim);
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }
    .badge-ver {
      font-size: 0.65rem;
      padding: 0.15rem 0.4rem;
      border-radius: 9999px;
      background: rgba(234, 88, 12, 0.15);
      color: var(--color-brand);
      border: 1px solid rgba(234, 88, 12, 0.3);
      font-family: var(--font-mono);
      font-weight: 700;
      margin-left: auto;
    }

    .status-bar-box {
      padding: 0.65rem 1.25rem;
      background: rgba(16, 185, 129, 0.06);
      border-bottom: 1px solid var(--border-subtle);
      display: flex;
      align-items: center;
      gap: 0.5rem;
      font-size: 0.75rem;
      font-weight: 600;
      color: var(--color-emerald);
    }
    .status-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--color-emerald);
      box-shadow: 0 0 8px var(--color-emerald);
      animation: pulse-dot 2s infinite;
    }
    @keyframes pulse-dot {
      0%, 100% { opacity: 1; transform: scale(1); }
      50% { opacity: 0.5; transform: scale(0.85); }
    }

    .nav-list {
      list-style: none;
      padding: 0.75rem 0.5rem;
      flex: 1;
      overflow-y: auto;
      display: flex;
      flex-direction: column;
      gap: 0.2rem;
    }
    .nav-item {
      padding: 0.65rem 0.85rem;
      border-radius: 8px;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 0.75rem;
      font-size: 0.85rem;
      font-weight: 600;
      color: var(--text-muted);
      transition: all 0.15s ease;
    }
    .nav-item:hover {
      background-color: var(--bg-surface-elevated);
      color: var(--text-heading);
    }
    .nav-item.active {
      background-color: rgba(234, 88, 12, 0.12);
      color: var(--color-brand);
      border-left: 3px solid var(--color-brand);
    }
    .nav-icon { font-size: 1.1rem; width: 20px; text-align: center; }
    .nav-count {
      margin-left: auto;
      font-size: 0.7rem;
      padding: 0.15rem 0.45rem;
      border-radius: 9999px;
      background: var(--bg-surface-elevated);
      color: var(--text-dim);
      font-family: var(--font-mono);
    }
    .nav-item.active .nav-count {
      background: rgba(234, 88, 12, 0.2);
      color: var(--color-brand);
    }

    .sidebar-footer {
      padding: 0.85rem 1rem;
      border-top: 1px solid var(--border-subtle);
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      font-size: 0.75rem;
    }
    .active-provider-summary {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.4rem 0.6rem;
      border-radius: 6px;
      background: var(--bg-surface-elevated);
      border: 1px solid var(--border-subtle);
    }
    .footer-actions {
      display: flex;
      align-items: center;
      justify-content: space-between;
      color: var(--text-dim);
    }
    .theme-btn {
      background: var(--bg-surface-elevated);
      border: 1px solid var(--border-subtle);
      border-radius: 6px;
      padding: 0.35rem 0.65rem;
      color: var(--text-heading);
      cursor: pointer;
      font-size: 0.8rem;
      font-weight: 600;
      display: inline-flex;
      align-items: center;
      gap: 0.35rem;
      transition: all 0.15s;
    }
    .theme-btn:hover { background: var(--bg-card-hover); border-color: var(--border-accent); }
    .theme-toggle-btn {
      display: inline-flex;
      align-items: center;
      gap: 0.45rem;
      padding: 0.35rem 0.75rem;
      border-radius: 8px;
      font-size: 0.8rem;
      font-weight: 600;
      background: var(--bg-surface);
      border: 1px solid var(--border-subtle);
      color: var(--text-heading);
      cursor: pointer;
      box-shadow: var(--card-shadow);
      transition: all 0.15s ease;
    }
    .theme-toggle-btn:hover {
      background: var(--bg-surface-elevated);
      border-color: var(--border-accent);
      transform: translateY(-1px);
    }

    /* --- Main Content Area --- */
    #main {
      flex: 1 1 0;
      min-width: 0;
      width: 0;
      display: flex;
      flex-direction: column;
      overflow: hidden;
      position: relative;
    }

    /* --- Top Header Bar --- */
    .top-header {
      height: 60px;
      min-height: 60px;
      flex-shrink: 0;
      background-color: var(--bg-sidebar);
      border-bottom: 1px solid var(--border-subtle);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 1.5rem;
      z-index: 10;
    }
    .breadcrumb {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      font-size: 0.85rem;
      font-weight: 600;
      color: var(--text-muted);
    }
    .breadcrumb-title { color: var(--text-heading); font-weight: 700; }
    .top-tools {
      display: flex;
      align-items: center;
      gap: 0.85rem;
    }
    .top-select-box {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      background: var(--bg-surface);
      border: 1px solid var(--border-subtle);
      border-radius: 8px;
      padding: 0.3rem 0.75rem;
      font-size: 0.82rem;
      box-shadow: var(--card-shadow);
    }
    .top-select-box label { font-size: 0.75rem; color: var(--text-dim); font-weight: 600; }
    .top-select-box select {
      background: none;
      border: none;
      color: var(--text-heading);
      font-weight: 600;
      outline: none;
      cursor: pointer;
    }
    option {
      background-color: var(--bg-surface);
      color: var(--text-heading);
    }
    .token-saved-pill {
      display: flex;
      align-items: center;
      gap: 0.4rem;
      padding: 0.35rem 0.75rem;
      border-radius: 9999px;
      background: rgba(5, 150, 105, 0.08);
      border: 1px solid rgba(5, 150, 105, 0.25);
      color: var(--color-emerald);
      font-size: 0.78rem;
      font-family: var(--font-mono);
      font-weight: 700;
    }

    .content-scroll {
      flex: 1 1 0;
      min-width: 0;
      overflow-y: auto;
      overflow-x: hidden;
      padding: 1.25rem 1.5rem;
    }

    /* --- Shared UI Components --- */
    .section-view { display: none; width: 100%; min-width: 0; }
    .section-view.active { display: block; animation: fadeIn 0.2s ease; }
    @keyframes fadeIn { from { opacity: 0; transform: translateY(3px); } to { opacity: 1; transform: translateY(0); } }

    .view-title-block { margin-bottom: 1.25rem; }
    .view-title-block h1 { font-size: 1.35rem; font-weight: 800; color: var(--text-heading); letter-spacing: -0.02em; }
    .view-title-block p { font-size: 0.85rem; color: var(--text-muted); margin-top: 0.2rem; }

    .card {
      background-color: var(--bg-surface);
      border: 1px solid var(--border-subtle);
      border-radius: 12px;
      padding: 1.25rem 1.5rem;
      margin-bottom: 1.25rem;
      box-shadow: var(--card-shadow);
      transition: background-color 0.2s, border-color 0.2s, box-shadow 0.2s;
    }
    .card-title {
      font-size: 1rem;
      font-weight: 700;
      color: var(--text-heading);
      margin-bottom: 0.85rem;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }

    /* KPI Grid */
    .kpi-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
      gap: 1rem;
      margin-bottom: 1.25rem;
    }
    .kpi-card {
      background: var(--bg-surface);
      border: 1px solid var(--border-subtle);
      border-radius: 10px;
      padding: 1.15rem 1.25rem;
      display: flex;
      flex-direction: column;
      gap: 0.4rem;
      box-shadow: var(--card-shadow);
      transition: all 0.2s;
    }
    .kpi-card:hover {
      border-color: var(--border-accent);
      transform: translateY(-1px);
      box-shadow: var(--card-shadow-hover);
    }
    .kpi-label { font-size: 0.75rem; font-weight: 600; text-transform: uppercase; color: var(--text-dim); letter-spacing: 0.05em; display: flex; align-items: center; justify-content: space-between; }
    .kpi-val { font-size: 1.75rem; font-weight: 800; color: var(--text-heading); font-family: var(--font-mono); line-height: 1.2; }
    .kpi-sub { font-size: 0.75rem; color: var(--text-muted); display: flex; align-items: center; gap: 0.4rem; }

    /* Tables */
    .table-container {
      width: 100%;
      max-width: 100%;
      overflow-x: auto;
      border: 1px solid var(--border-subtle);
      border-radius: 10px;
      background: var(--bg-surface);
      box-shadow: var(--card-shadow);
      -webkit-overflow-scrolling: touch;
    }
    table { width: 100%; min-width: 860px; border-collapse: collapse; font-size: 0.82rem; }
    th, td { padding: 0.65rem 0.85rem; text-align: left; border-bottom: 1px solid var(--border-subtle); vertical-align: middle; }
    th {
      background-color: var(--bg-surface-elevated);
      color: var(--text-dim);
      font-weight: 700;
      font-size: 0.72rem;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      white-space: nowrap;
    }
    tr:last-child td { border-bottom: none; }
    tbody tr:hover { background-color: var(--bg-card-hover); }

    /* Badges */
    .badge {
      display: inline-flex;
      align-items: center;
      gap: 0.35rem;
      padding: 0.2rem 0.55rem;
      border-radius: 9999px;
      font-size: 0.7rem;
      font-family: var(--font-mono);
      font-weight: 700;
      white-space: nowrap;
    }
    .badge-green { background: rgba(16, 185, 129, 0.12); color: #10B981; border: 1px solid rgba(16, 185, 129, 0.3); }
    .badge-amber { background: rgba(245, 158, 11, 0.12); color: #F59E0B; border: 1px solid rgba(245, 158, 11, 0.3); }
    .badge-blue { background: rgba(14, 165, 233, 0.12); color: #0EA5E9; border: 1px solid rgba(14, 165, 233, 0.3); }
    .badge-orange { background: rgba(234, 88, 12, 0.12); color: #EA580C; border: 1px solid rgba(234, 88, 12, 0.3); }
    .badge-gray { background: var(--bg-surface-elevated); color: var(--text-muted); border: 1px solid var(--border-subtle); }
    .badge-red { background: rgba(244, 63, 94, 0.12); color: #F43F5E; border: 1px solid rgba(244, 63, 94, 0.3); }

    /* Buttons & Inputs */
    button {
      padding: 0.35rem 0.65rem;
      border-radius: 6px;
      font-size: 0.78rem;
      font-weight: 600;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 0.35rem;
      transition: all 0.15s;
      white-space: nowrap;
      flex-shrink: 0;
    }
    .btn-brand {
      background: linear-gradient(135deg, #C05A28 0%, #A3481D 100%);
      color: #FFFFFF;
      border: none;
      box-shadow: 0 1px 4px rgba(192, 90, 40, 0.25);
    }
    .btn-brand:hover { transform: translateY(-1px); box-shadow: 0 3px 8px rgba(192, 90, 40, 0.4); }
    .btn-outline {
      background: var(--bg-surface);
      border: 1px solid var(--border-subtle);
      color: var(--text-body);
    }
    .btn-outline:hover { background: var(--bg-surface-elevated); border-color: var(--border-accent); color: var(--text-heading); }
    .btn-sm { padding: 0.22rem 0.45rem; font-size: 0.74rem; border-radius: 5px; line-height: 1.2; }
      border-radius: 6px;
      font-size: 0.82rem;
      font-weight: 600;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 0.4rem;
      transition: all 0.15s;
    }
    .btn-brand {
      background: linear-gradient(135deg, #EA580C 0%, #C2410C 100%);
      color: #FFFFFF;
      border: none;
      box-shadow: 0 2px 8px rgba(234, 88, 12, 0.3);
    }
    .btn-brand:hover { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(234, 88, 12, 0.45); }
    .btn-outline {
      background: transparent;
      border: 1px solid var(--border-subtle);
      color: var(--text-body);
    }
    .btn-outline:hover { background: var(--bg-surface-elevated); border-color: var(--border-accent); color: var(--text-heading); }
    .btn-sm { padding: 0.25rem 0.55rem; font-size: 0.75rem; }

    input, select, textarea {
      background-color: var(--bg-canvas);
      border: 1px solid var(--border-subtle);
      border-radius: 6px;
      color: var(--text-heading);
      padding: 0.45rem 0.75rem;
      font-size: 0.85rem;
      outline: none;
      transition: border-color 0.15s;
    }
    input:focus, select:focus, textarea:focus { border-color: var(--border-focus); }

    /* Filter Toolbar */
    .filter-bar {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 1rem;
      margin-bottom: 1rem;
      flex-wrap: wrap;
    }
    .filter-tabs { display: flex; gap: 0.4rem; flex-wrap: wrap; }
    .filter-tab {
      padding: 0.35rem 0.75rem;
      border-radius: 6px;
      font-size: 0.78rem;
      font-weight: 600;
      cursor: pointer;
      background: var(--bg-surface);
      border: 1px solid var(--border-subtle);
      color: var(--text-muted);
    }
    .filter-tab.active { background: rgba(234, 88, 12, 0.15); color: var(--color-brand); border-color: var(--color-brand); }

    /* Playground / Chat */
    .playground-split {
      display: grid;
      grid-template-columns: 360px 1fr;
      gap: 1.25rem;
    }
    @media (max-width: 960px) { .playground-split { grid-template-columns: 1fr; } }
    .config-panel { display: flex; flex-direction: column; gap: 1rem; }
    .field-group { display: flex; flex-direction: column; gap: 0.35rem; }
    .field-label { font-size: 0.75rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.04em; }
    .chat-output-box {
      background: var(--bg-surface);
      border: 1px solid var(--border-subtle);
      border-radius: 10px;
      padding: 1.25rem;
      min-height: 400px;
      display: flex;
      flex-direction: column;
    }
    .chat-response-text {
      flex: 1;
      font-family: var(--font-sans);
      line-height: 1.6;
      white-space: pre-wrap;
      overflow-y: auto;
      color: var(--text-body);
      font-size: 0.9rem;
    }

    /* Modal */
    .modal-backdrop {
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0, 0, 0, 0.7);
      backdrop-filter: blur(4px);
      display: none;
      align-items: center;
      justify-content: center;
      z-index: 100;
    }
    .modal-box {
      background: var(--bg-surface);
      border: 1px solid var(--border-subtle);
      border-radius: 12px;
      width: 90%;
      max-width: 520px;
      padding: 1.5rem;
      box-shadow: 0 16px 36px rgba(0, 0, 0, 0.5);
    }
    .modal-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
    .modal-header h3 { font-size: 1.15rem; color: var(--text-heading); font-weight: 700; }
    .modal-close { background: none; border: none; font-size: 1.2rem; color: var(--text-dim); cursor: pointer; }
    .modal-footer { display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 1.25rem; }

    /* Auto Row Special Highlight */
    .row-auto-highlight {
      background: linear-gradient(90deg, rgba(234, 88, 12, 0.08) 0%, rgba(245, 158, 11, 0.04) 100%) !important;
      font-weight: 600;
    }
  </style>
</head>
<body>

  <!-- ==================== SIDEBAR ==================== -->
  <aside id="sidebar">
    <div class="brand-header">
      <img src="/NITYAKSHA_LOGO.png" alt="Nityaksha" class="brand-logo" onerror="this.style.display='none'">
      <div>
        <div class="brand-title">NITYAKSHA</div>
        <div class="brand-subtitle">AI Gateway & Router</div>
      </div>
      <span class="badge-ver">v1.0.0</span>
    </div>

    <div class="status-bar-box">
      <div class="status-dot"></div>
      <span id="gateway-status-badge">Gateway Online · 46.62.188.249:8080</span>
    </div>

    <ul class="nav-list">
      <li class="nav-item active" onclick="switchNav('overview')">
        <span class="nav-icon">📊</span>
        <span>Overview & Analytics</span>
      </li>
      <li class="nav-item" onclick="switchNav('providers')">
        <span class="nav-icon">⚡</span>
        <span>AI Provider Hub</span>
        <span class="nav-count" id="nav-count-prov">39</span>
      </li>
      <li class="nav-item" onclick="switchNav('router')">
        <span class="nav-icon">🔀</span>
        <span>Smart Router & Breakers</span>
      </li>
      <li class="nav-item" onclick="switchNav('playground')">
        <span class="nav-icon">🧪</span>
        <span>Playground / Tester</span>
      </li>
      <li class="nav-item" onclick="switchNav('logs')">
        <span class="nav-icon">📜</span>
        <span>Request Logs</span>
      </li>
      <li class="nav-item" onclick="switchNav('access')">
        <span class="nav-icon">🔑</span>
        <span>API Keys & Integration</span>
      </li>
      <li class="nav-item" onclick="switchNav('bots')">
        <span class="nav-icon">🤖</span>
        <span>Bots & DAG Workflows</span>
      </li>
    </ul>

    <div class="sidebar-footer">
      <div class="active-provider-summary">
        <span style="color:var(--color-brand); font-size:1rem;">◈</span>
        <div style="overflow:hidden; text-overflow:ellipsis; white-space:nowrap;">
          <div style="font-size:0.7rem; color:var(--text-dim); font-weight:600;">ACTIVE PROVIDER</div>
          <div id="footer-active-prov" style="font-weight:700; color:var(--text-heading);">Auto (Smart Free Router)</div>
        </div>
      </div>
      <div class="footer-actions">
        <button class="theme-btn" onclick="toggleTheme()" id="theme-btn" title="Toggle Light / Dark mode">
          <span id="sidebar-theme-icon">🌙</span>
          <span id="sidebar-theme-label">Dark</span>
        </button>
        <a href="https://github.com/simhaonline/ntx" target="_blank" style="color:var(--text-dim); text-decoration:none; font-weight:600;">GitHub ↗</a>
      </div>
    </div>
  </aside>

  <!-- ==================== MAIN CONTENT ==================== -->
  <main id="main">
    <!-- Top Header -->
    <header class="top-header">
      <div class="breadcrumb">
        <span>Gateway</span>
        <span>/</span>
        <span class="breadcrumb-title" id="breadcrumb-title">Overview & Analytics</span>
      </div>
      <div class="top-tools">
        <button class="theme-toggle-btn" onclick="toggleTheme()" id="top-theme-btn" title="Toggle Light / Dark mode">
          <span id="top-theme-icon">🌙</span>
          <span id="top-theme-label">Dark Mode</span>
        </button>
        <div class="token-saved-pill">
          <span>⚡ RTK Saved:</span>
          <span id="top-tokens-saved">0</span>
          <span>tok</span>
        </div>
        <div class="top-select-box">
          <label>ACTIVE AI:</label>
          <select id="quick-provider-select" onchange="quickSwitchProvider(this.value)">
            <option value="auto">Auto (Smart Free Router)</option>
          </select>
        </div>
      </div>
    </header>

    <div class="content-scroll">

      <!-- ==================== 1. OVERVIEW & ANALYTICS ==================== -->
      <section id="view-overview" class="section-view active">
        <div class="view-title-block">
          <h1>Service Overview & Traffic Telemetry</h1>
          <p>Real-time gateway performance, upstream token throughput, and circuit-breaker telemetry inspired by APIPark.</p>
        </div>

        <!-- KPI Metrics -->
        <div class="kpi-grid">
          <div class="kpi-card">
            <div class="kpi-label">
              <span>Total API Requests</span>
              <span style="color:var(--color-emerald);">● LIVE</span>
            </div>
            <div class="kpi-val" id="kpi-total-req">1,482</div>
            <div class="kpi-sub">
              <span style="color:var(--color-emerald); font-weight:700;">99.98%</span>
              <span>success rate · 0 downtime</span>
            </div>
          </div>

          <div class="kpi-card">
            <div class="kpi-label">
              <span>RTK Tokens Saved</span>
              <span style="color:var(--color-brand);">⚡ 28.4% SAVINGS</span>
            </div>
            <div class="kpi-val" id="kpi-tokens-saved" style="color:var(--color-brand);">0</div>
            <div class="kpi-sub">
              <span>Whitespace & redundancy optimization</span>
            </div>
          </div>

          <div class="kpi-card">
            <div class="kpi-label">
              <span>Avg Upstream Latency</span>
              <span id="kpi-latency-badge" class="badge badge-green">FAST</span>
            </div>
            <div class="kpi-val" id="kpi-avg-latency">218 <span style="font-size:1rem; color:var(--text-dim);">ms</span></div>
            <div class="kpi-sub">
              <span>p95: 390ms · Circuit breakers: 0 active</span>
            </div>
          </div>

          <div class="kpi-card">
            <div class="kpi-label">
              <span>Active Provider Pool</span>
              <span class="badge badge-orange">ZERO-COST</span>
            </div>
            <div class="kpi-val" id="kpi-active-pool">Auto (Smart)</div>
            <div class="kpi-sub">
              <span>39 backends · Cascading failover ready</span>
            </div>
          </div>
        </div>

        <!-- Telemetry Charts -->
        <div class="card">
          <div class="card-title">
            <span>Hourly Request Throughput & Token Savings</span>
            <span style="font-size:0.75rem; color:var(--text-dim); font-weight:500;">Last 24 Hours</span>
          </div>
          <div style="height:140px; display:flex; align-items:flex-end; gap:8px; padding-top:10px;">
            <svg width="100%" height="100%" viewBox="0 0 800 120" preserveAspectRatio="none">
              <defs>
                <linearGradient id="barGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="#EA580C" stop-opacity="0.9"/>
                  <stop offset="100%" stop-color="#EA580C" stop-opacity="0.2"/>
                </linearGradient>
              </defs>
              <rect x="10" y="70" width="22" height="50" rx="3" fill="url(#barGrad)"/>
              <rect x="42" y="55" width="22" height="65" rx="3" fill="url(#barGrad)"/>
              <rect x="74" y="80" width="22" height="40" rx="3" fill="url(#barGrad)"/>
              <rect x="106" y="90" width="22" height="30" rx="3" fill="url(#barGrad)"/>
              <rect x="138" y="95" width="22" height="25" rx="3" fill="url(#barGrad)"/>
              <rect x="170" y="85" width="22" height="35" rx="3" fill="url(#barGrad)"/>
              <rect x="202" y="60" width="22" height="60" rx="3" fill="url(#barGrad)"/>
              <rect x="234" y="40" width="22" height="80" rx="3" fill="url(#barGrad)"/>
              <rect x="266" y="25" width="22" height="95" rx="3" fill="url(#barGrad)"/>
              <rect x="298" y="30" width="22" height="90" rx="3" fill="url(#barGrad)"/>
              <rect x="330" y="15" width="22" height="105" rx="3" fill="url(#barGrad)"/>
              <rect x="362" y="20" width="22" height="100" rx="3" fill="url(#barGrad)"/>
              <rect x="394" y="35" width="22" height="85" rx="3" fill="url(#barGrad)"/>
              <rect x="426" y="45" width="22" height="75" rx="3" fill="url(#barGrad)"/>
              <rect x="458" y="30" width="22" height="90" rx="3" fill="url(#barGrad)"/>
              <rect x="490" y="10" width="22" height="110" rx="3" fill="url(#barGrad)"/>
              <rect x="522" y="25" width="22" height="95" rx="3" fill="url(#barGrad)"/>
              <rect x="554" y="40" width="22" height="80" rx="3" fill="url(#barGrad)"/>
              <rect x="586" y="35" width="22" height="85" rx="3" fill="url(#barGrad)"/>
              <rect x="618" y="50" width="22" height="70" rx="3" fill="url(#barGrad)"/>
              <rect x="650" y="20" width="22" height="100" rx="3" fill="url(#barGrad)"/>
              <rect x="682" y="15" width="22" height="105" rx="3" fill="url(#barGrad)"/>
              <rect x="714" y="30" width="22" height="90" rx="3" fill="url(#barGrad)"/>
              <rect x="746" y="18" width="22" height="102" rx="3" fill="url(#barGrad)"/>
            </svg>
          </div>
        </div>

        <!-- Upstream Health Summary Table -->
        <div class="card">
          <div class="card-title">
            <span>Primary Provider Upstreams & Circuit Breakers</span>
            <button class="btn-outline btn-sm" onclick="refreshRouterState()">Refresh State</button>
          </div>
          <div class="table-container">
            <table>
              <thead>
                <tr>
                  <th style="min-width:180px;">Target Provider</th>
                  <th style="width:130px; text-align:center;">Circuit Breaker</th>
                  <th style="width:90px; text-align:right;">Success</th>
                  <th style="width:90px; text-align:right;">Failures</th>
                  <th style="width:110px; text-align:right;">Avg Latency</th>
                  <th style="min-width:140px;">Status / Notes</th>
                </tr>
              </thead>
              <tbody id="overview-breakers-table">
                <tr><td colspan="6" style="text-align:center; padding:1.5rem;">Loading telemetry...</td></tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <!-- ==================== 2. AI PROVIDER HUB ==================== -->
      <section id="view-providers" class="section-view">
        <div class="view-title-block">
          <h1>AI Provider Hub & Upstream Endpoints</h1>
          <p>Configure credentials, test latency, and manage default models across 39 AI providers. <strong>Auto (Smart Free Router)</strong> is enabled at index 0.</p>
        </div>

        <div class="filter-bar">
          <div class="filter-tabs">
            <span class="filter-tab active" onclick="filterProviders('all', this)">All (39)</span>
            <span class="filter-tab" onclick="filterProviders('smart', this)">Smart Router</span>
            <span class="filter-tab" onclick="filterProviders('ready', this)">Active / Ready</span>
            <span class="filter-tab" onclick="filterProviders('commercial', this)">Commercial</span>
            <span class="filter-tab" onclick="filterProviders('local', this)">Local Sovereign</span>
            <span class="filter-tab" onclick="filterProviders('cloud', this)">Cloud Infra</span>
          </div>
          <input type="text" id="provider-search" placeholder="Search provider or model..." oninput="searchProviders(this.value)" style="width:260px;">
        </div>

        <div class="table-container">
          <table>
            <thead>
              <tr>
                <th style="width:38px; text-align:center;">#</th>
                <th style="width:190px;">Provider Name</th>
                <th style="width:115px; text-align:center;">Category</th>
                <th style="width:200px;">Default Model</th>
                <th style="width:105px; text-align:center;">Status</th>
                <th style="width:155px;">Auth / Credentials</th>
                <th style="width:170px; text-align:right;">Actions</th>
              </tr>
            </thead>
            <tbody id="providers-table-body">
              <tr><td colspan="7" style="text-align:center; padding:2rem;">Loading providers matrix...</td></tr>
            </tbody>
          </table>
        </div>
      </section>

      <!-- ==================== 3. SMART ROUTER & CIRCUIT BREAKERS ==================== -->
      <section id="view-router" class="section-view">
        <div class="view-title-block">
          <h1>Smart Router & Circuit Breaker Engine</h1>
          <p>Real-time cooldown monitoring, RTK prompt token compression, and adaptive complexity classification.</p>
        </div>

        <div class="kpi-grid">
          <div class="kpi-card">
            <div class="kpi-label"><span>Circuit Breaker State</span></div>
            <div class="kpi-val" style="color:var(--color-emerald);">HEALTHY</div>
            <div class="kpi-sub"><span>60s cooldown applied on 429/503 errors</span></div>
          </div>
          <div class="kpi-card">
            <div class="kpi-label"><span>RTK Compression</span></div>
            <div class="kpi-val" style="color:var(--color-brand);" id="router-rtk-saved">0 tok</div>
            <div class="kpi-sub"><span>Real-time token savings counter</span></div>
          </div>
          <div class="kpi-card">
            <div class="kpi-label"><span>Adaptive Routing</span></div>
            <div class="kpi-val" style="color:var(--color-cyan);">3 TIERS</div>
            <div class="kpi-sub"><span>FastFlash · Balanced · DeepReasoning</span></div>
          </div>
        </div>

        <!-- Simulators Grid -->
        <div style="display:grid; grid-template-columns:1fr 1fr; gap:1.25rem; margin-bottom:1.5rem;">
          <!-- RTK Compressor Test -->
          <div class="card">
            <div class="card-title">
              <span>🧪 Live RTK Token Compressor Simulator</span>
            </div>
            <p style="font-size:0.8rem; color:var(--text-muted); margin-bottom:0.75rem;">
              Test how Nityaksha strips redundant blank lines and collapses spaces without mutating code indentation.
            </p>
            <textarea id="sim-compress-input" rows="4" style="width:100%; margin-bottom:0.75rem;" placeholder="Paste text with excessive whitespace and blank lines..."></textarea>
            <div style="display:flex; justify-content:space-between; align-items:center;">
              <button class="btn-brand" onclick="runSimCompress()">Compress Prompt</button>
              <div id="sim-compress-res" style="font-family:var(--font-mono); font-size:0.8rem; color:var(--color-emerald); font-weight:700;"></div>
            </div>
          </div>

          <!-- Complexity Classifier Test -->
          <div class="card">
            <div class="card-title">
              <span>🧠 Adaptive Complexity Classifier Simulator</span>
            </div>
            <p style="font-size:0.8rem; color:var(--text-muted); margin-bottom:0.75rem;">
              Test LLMRouter query classification into FastFlash, Balanced, or DeepReasoning tiers.
            </p>
            <textarea id="sim-classify-input" rows="4" style="width:100%; margin-bottom:0.75rem;" placeholder="e.g. 'Step by step prove Fermat's Last Theorem using elliptic curves'"></textarea>
            <div style="display:flex; justify-content:space-between; align-items:center;">
              <button class="btn-brand" onclick="runSimClassify()">Classify Query</button>
              <div id="sim-classify-res" style="font-family:var(--font-mono); font-size:0.8rem; color:var(--color-cyan); font-weight:700;"></div>
            </div>
          </div>
        </div>

        <div class="card">
          <div class="card-title">
            <span>Circuit Breaker Status Matrix</span>
            <button class="btn-outline btn-sm" onclick="resetAllBreakers()">Reset Breakers to 0</button>
          </div>
          <div class="table-container">
            <table>
              <thead>
                <tr>
                  <th style="min-width:180px;">Target Provider</th>
                  <th style="width:130px; text-align:center;">Current State</th>
                  <th style="width:110px; text-align:right;">Total Requests</th>
                  <th style="width:90px; text-align:right;">Successes</th>
                  <th style="width:90px; text-align:right;">Failures</th>
                  <th style="width:110px; text-align:right;">Avg Latency</th>
                  <th style="width:140px; text-align:center;">Cooldown Status</th>
                </tr>
              </thead>
              <tbody id="router-full-table">
                <tr><td colspan="7" style="text-align:center; padding:1.5rem;">Loading circuit breakers...</td></tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <!-- ==================== 4. PLAYGROUND / TESTER ==================== -->
      <section id="view-playground" class="section-view">
        <div class="view-title-block">
          <h1>API Playground & Multi-Model Chat Runner</h1>
          <p>Interactive test bench for executing prompts directly against any configured AI provider or the <strong>Auto Free Router</strong>.</p>
        </div>

        <div class="playground-split">
          <!-- Left Config -->
          <div class="config-panel">
            <div class="card">
              <div class="card-title"><span>Execution Parameters</span></div>
              <div class="field-group">
                <label class="field-label">Target Model</label>
                <select id="play-model" style="width:100%;">
                  <option value="auto-free">auto-free (Smart Free Tier Router)</option>
                  <option value="gemini-3.6-flash">gemini-3.6-flash</option>
                  <option value="gpt-4o">gpt-4o</option>
                  <option value="claude-3-7-sonnet-20250219">claude-3-7-sonnet</option>
                  <option value="deepseek-chat">deepseek-chat</option>
                </select>
              </div>

              <div class="field-group" style="margin-top:0.75rem;">
                <label class="field-label">System Prompt</label>
                <textarea id="play-sys" rows="3" style="width:100%;">Objective Technical Execution Mode. Zero fluff.</textarea>
              </div>

              <div class="field-group" style="margin-top:0.75rem;">
                <label class="field-label">Temperature: <span id="temp-val">0.7</span></label>
                <input type="range" min="0" max="1" step="0.05" value="0.7" oninput="document.getElementById('temp-val').textContent=this.value" id="play-temp">
              </div>

              <div style="margin-top:1.25rem;">
                <button class="btn-brand" style="width:100%; justify-content:center;" onclick="sendPlaygroundRequest()" id="btn-play-send">
                  <span>⚡ Execute Prompt</span>
                </button>
              </div>
            </div>

            <div class="card">
              <div class="card-title"><span>Starter Quick Prompts</span></div>
              <div style="display:flex; flex-direction:column; gap:0.4rem;">
                <button class="btn-outline btn-sm" onclick="setPlayPrompt('Explain zero-copy deserialization in Rust with safety invariants.')">Rust Zero-Copy Deserialization</button>
                <button class="btn-outline btn-sm" onclick="setPlayPrompt('Write a Pine Script v6 strategy with Fair Value Gap (FVG) and Killzones.')">Pine Script v6 FVG Strategy</button>
                <button class="btn-outline btn-sm" onclick="setPlayPrompt('Analyze microservice circuit-breaker patterns under 429 rate limit storm.')">Circuit Breaker Microservice Audit</button>
              </div>
            </div>
          </div>

          <!-- Right Output -->
          <div style="display:flex; flex-direction:column; gap:1rem;">
            <div class="card" style="padding:1rem;">
              <label class="field-label" style="margin-bottom:0.4rem; display:block;">User Prompt</label>
              <textarea id="play-prompt" rows="4" style="width:100%; font-size:0.9rem;" placeholder="Type your query or instruction here..."></textarea>
            </div>

            <div class="chat-output-box">
              <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:0.75rem; border-bottom:1px solid var(--border-subtle); padding-bottom:0.5rem;">
                <span class="field-label">Assistant Response</span>
                <div style="display:flex; gap:0.5rem; align-items:center;">
                  <span id="play-telemetry-pill" class="badge badge-gray" style="display:none;"></span>
                  <button class="btn-outline btn-sm" onclick="copyPlayResponse()">Copy</button>
                </div>
              </div>
              <div class="chat-response-text" id="play-response">Send a prompt using the button on the left to inspect response output, tokens consumed, and upstream latency.</div>
            </div>
          </div>
        </div>
      </section>

      <!-- ==================== 5. REQUEST LOGS ==================== -->
      <section id="view-logs" class="section-view">
        <div class="view-title-block">
          <h1>Request Logs & Telemetry Audit</h1>
          <p>Inspect real-time gateway completions, HTTP status codes, upstream latency, and token allocations.</p>
        </div>

        <div class="card">
          <div class="card-title">
            <span>Recent API Invocations</span>
            <button class="btn-outline btn-sm" onclick="clearLogsTable()">Clear Logs</button>
          </div>
          <div class="table-container">
            <table>
              <thead>
                <tr>
                  <th style="width:120px;">Timestamp</th>
                  <th style="min-width:180px;">Method & Endpoint</th>
                  <th style="min-width:180px;">Model / Provider</th>
                  <th style="width:95px; text-align:center;">Status</th>
                  <th style="width:90px; text-align:right;">Latency</th>
                  <th style="width:110px; text-align:right;">Tokens (In/Out)</th>
                </tr>
              </thead>
              <tbody id="logs-table-body">
                <tr><td colspan="6" style="text-align:center; padding:1.5rem;">No recent requests logged yet. Invoke /v1/chat/completions or use the Playground.</td></tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <!-- ==================== 6. API ACCESS & KEYS ==================== -->
      <section id="view-access" class="section-view">
        <div class="view-title-block">
          <h1>API Access & Drop-in Integration</h1>
          <p>Nityaksha provides drop-in OpenAI compatible endpoints. Connect standard CLI tools, Python SDKs, or LangChain directly to this port.</p>
        </div>

        <div class="card">
          <div class="card-title"><span>Standard Drop-in OpenAI Endpoints</span></div>
          <div class="table-container" style="margin-bottom:1.25rem;">
            <table>
              <thead>
                <tr>
                  <th>Endpoint</th>
                  <th>HTTP Method</th>
                  <th>Description</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><code>http://localhost:8080/v1/chat/completions</code></td>
                  <td><span class="badge badge-green">POST</span></td>
                  <td>OpenAI-compatible chat completions with RTK compression and failover</td>
                </tr>
                <tr>
                  <td><code>http://localhost:8080/v1/models</code></td>
                  <td><span class="badge badge-blue">GET</span></td>
                  <td>List all available models across all 39 providers</td>
                </tr>
                <tr>
                  <td><code>http://localhost:8080/v1/health</code></td>
                  <td><span class="badge badge-gray">GET</span></td>
                  <td>Health check and heartbeat</td>
                </tr>
                <tr>
                  <td><code>http://localhost:8080/api/router/stats</code></td>
                  <td><span class="badge badge-orange">GET</span></td>
                  <td>Telemetry metrics, token savings, and circuit-breaker matrix</td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="card-title" style="margin-top:1.5rem;"><span>Integration Code Snippets</span></div>
          
          <div style="margin-bottom:1rem;">
            <label class="field-label">cURL (Direct Terminal)</label>
            <pre style="background:var(--bg-canvas); padding:1rem; border-radius:8px; border:1px solid var(--border-subtle); overflow-x:auto; font-family:var(--font-mono); font-size:0.8rem; color:var(--text-body);">curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "auto-free",
    "messages": [{"role": "user", "content": "Explain zero-copy deserialization"}]
  }'</pre>
          </div>

          <div>
            <label class="field-label">Python (OpenAI Official SDK)</label>
            <pre style="background:var(--bg-canvas); padding:1rem; border-radius:8px; border:1px solid var(--border-subtle); overflow-x:auto; font-family:var(--font-mono); font-size:0.8rem; color:var(--text-body);">from openai import OpenAI

client = OpenAI(base_url="http://localhost:8080/v1", api_key="ntx-local")

response = client.chat.completions.create(
    model="auto-free",
    messages=[{"role": "user", "content": "Explain market structure in quant trading"}]
)
print(response.choices[0].message.content)</pre>
          </div>
        </div>
      </section>

      <!-- ==================== 7. BOTS & DAG WORKFLOWS ==================== -->
      <section id="view-bots" class="section-view">
        <div class="view-title-block">
          <h1>Autonomous Bot Catalog & Multi-Agent DAGs</h1>
          <p>Preconfigured specialized agent personas and Directed Acyclic Graph (DAG) pipelines.</p>
        </div>

        <div class="card">
          <div class="card-title"><span>Specialized Autonomous Bots (7)</span></div>
          <div style="display:grid; grid-template-columns:repeat(auto-fit, minmax(280px, 1fr)); gap:1rem;">
            <div style="background:var(--bg-canvas); padding:1.25rem; border-radius:10px; border:1px solid var(--border-subtle);">
              <div style="font-size:1.05rem; font-weight:700; color:var(--text-heading); margin-bottom:0.35rem;">👑 Concierge Prime</div>
              <div style="font-size:0.8rem; color:var(--text-muted); margin-bottom:0.75rem;">Master router & intent synthesis agent with full access to memory.</div>
              <span class="badge badge-green">ACTIVE</span>
            </div>
            <div style="background:var(--bg-canvas); padding:1.25rem; border-radius:10px; border:1px solid var(--border-subtle);">
              <div style="font-size:1.05rem; font-weight:700; color:var(--text-heading); margin-bottom:0.35rem;">⚙️ Code Architect</div>
              <div style="font-size:0.8rem; color:var(--text-muted); margin-bottom:0.75rem;">AST pruner, cargo compiler, and unit test synthesizer.</div>
              <span class="badge badge-blue">READY</span>
            </div>
            <div style="background:var(--bg-canvas); padding:1.25rem; border-radius:10px; border:1px solid var(--border-subtle);">
              <div style="font-size:1.05rem; font-weight:700; color:var(--text-heading); margin-bottom:0.35rem;">📊 Quant Risk Specialist</div>
              <div style="font-size:0.8rem; color:var(--text-muted); margin-bottom:0.75rem;">Pine Script v6 verification, killzones, and cumulative volume profile.</div>
              <span class="badge badge-orange">READY</span>
            </div>
            <div style="background:var(--bg-canvas); padding:1.25rem; border-radius:10px; border:1px solid var(--border-subtle);">
              <div style="font-size:1.05rem; font-weight:700; color:var(--text-heading); margin-bottom:0.35rem;">🛡️ Security RedTeam</div>
              <div style="font-size:0.8rem; color:var(--text-muted); margin-bottom:0.75rem;">POSIX cgroups v2 quotas and container namespace jail.</div>
              <span class="badge badge-amber">ARMED</span>
            </div>
          </div>
        </div>
      </section>

    </div>
  </main>

  <!-- ==================== CONFIGURE MODAL ==================== -->
  <div class="modal-backdrop" id="config-modal">
    <div class="modal-box">
      <div class="modal-header">
        <h3 id="modal-prov-title">Configure Provider</h3>
        <button class="modal-close" onclick="closeConfigModal()">✕</button>
      </div>
      <input type="hidden" id="modal-prov-id">

      <div class="field-group" style="margin-bottom:0.85rem;">
        <label class="field-label">API Key / Token</label>
        <input type="password" id="modal-api-key" placeholder="Enter API key or leave blank to keep unchanged" style="width:100%;">
      </div>

      <div class="field-group" style="margin-bottom:0.85rem;">
        <label class="field-label">Custom Base URL (Optional)</label>
        <input type="text" id="modal-base-url" placeholder="e.g. https://api.openai.com/v1" style="width:100%;">
      </div>

      <div class="field-group" style="margin-bottom:0.85rem;">
        <label class="field-label">Default Model</label>
        <select id="modal-model-select" style="width:100%;"></select>
      </div>

      <div class="field-group">
        <label style="display:flex; align-items:center; gap:0.5rem; font-size:0.85rem; cursor:pointer;">
          <input type="checkbox" id="modal-set-active" checked>
          <span>Set as Active Provider immediately</span>
        </label>
      </div>

      <div class="modal-footer">
        <button class="btn-outline" onclick="closeConfigModal()">Cancel</button>
        <button class="btn-brand" onclick="saveProviderConfig()">Save Configuration</button>
      </div>
    </div>
  </div>

  <script>
    let providersCache = [];
    let recentLogs = [];

    // --- Navigation Switching ---
    function switchNav(viewId) {
      document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.section-view').forEach(el => el.classList.remove('active'));

      const target = document.getElementById('view-' + viewId);
      if (target) {
        target.classList.add('active');
      }

      // Mark nav link
      const links = document.querySelectorAll('.nav-item');
      links.forEach(l => {
        if (l.getAttribute('onclick') && l.getAttribute('onclick').includes(viewId)) {
          l.classList.add('active');
        }
      });

      const titles = {
        'overview': 'Overview & Analytics',
        'providers': 'AI Provider Hub',
        'router': 'Smart Router & Circuit Breakers',
        'playground': 'Playground / API Tester',
        'logs': 'Request Logs & Audit',
        'access': 'API Keys & Integration',
        'bots': 'Autonomous Bots & DAGs'
      };
      document.getElementById('breadcrumb-title').textContent = titles[viewId] || 'Control Plane';
    }

    // --- Unified Theme Engine (Light & Dark with LocalStorage Persistence) ---
    function applyTheme(theme) {
      document.documentElement.setAttribute('data-theme', theme);
      try {
        localStorage.setItem('ntx_theme', theme);
      } catch (e) {}

      const isLight = theme === 'light';
      const nextIcon = isLight ? '🌙' : '☀️';
      const nextModeText = isLight ? 'Dark Mode' : 'Light Mode';
      const nextShortText = isLight ? 'Dark' : 'Light';

      const topIcon = document.getElementById('top-theme-icon');
      const topLabel = document.getElementById('top-theme-label');
      if (topIcon) topIcon.textContent = nextIcon;
      if (topLabel) topLabel.textContent = nextModeText;

      const sideIcon = document.getElementById('sidebar-theme-icon');
      const sideLabel = document.getElementById('sidebar-theme-label');
      if (sideIcon) sideIcon.textContent = nextIcon;
      if (sideLabel) sideLabel.textContent = nextShortText;
    }

    function toggleTheme() {
      const current = document.documentElement.getAttribute('data-theme') || 'light';
      const next = current === 'dark' ? 'light' : 'dark';
      applyTheme(next);
    }

    // --- Fetch Providers Matrix ---
    function loadProviders() {
      fetch('/api/providers')
        .then(r => r.json())
        .then(data => {
          providersCache = data;
          renderProvidersTable(data);
          updateQuickSelect(data);
          updateRouterTable(data);
        })
        .catch(err => console.error("Error loading providers:", err));
    }

    function renderProvidersTable(providers) {
      const tbody = document.getElementById('providers-table-body');
      if (!tbody) return;

      let html = '';
      providers.forEach((p, idx) => {
        const isAuto = p.id === 'auto';
        const rowClass = isAuto ? 'row-auto-highlight' : '';

        const badgeHtml = p.is_active
          ? '<span class="badge badge-green">● ACTIVE</span>'
          : (p.is_configured ? '<span class="badge badge-blue">✓ READY</span>' : '<span class="badge badge-gray">○ NEED KEY</span>');

        const catBadge = isAuto
          ? '<span class="badge badge-orange">SMART ROUTER</span>'
          : `<span class="badge badge-gray">${escapeHtml(p.category)}</span>`;

        const authHtml = isAuto
          ? '<span style="color:var(--color-emerald); font-weight:600; font-size:0.75rem;">● Free Tier (Built-in)</span>'
          : (p.is_configured
              ? '<span style="color:var(--color-emerald); font-weight:600; font-size:0.75rem;">● Key Configured</span>'
              : `<div style="line-height:1.2;">
                   <span style="color:var(--text-dim); font-size:0.75rem;">Unconfigured</span>
                   <div style="font-family:var(--font-mono); font-size:0.67rem; color:var(--text-dim); opacity:0.8;">${escapeHtml(p.auth_summary.replace('Not configured ', ''))}</div>
                 </div>`);

        html += `
          <tr class="${rowClass}">
            <td style="font-family:var(--font-mono); font-weight:700; color:var(--text-dim); text-align:center;">${idx}</td>
            <td>
              <div style="font-weight:700; color:var(--text-heading); display:flex; align-items:center; gap:0.35rem; white-space:nowrap;">
                ${isAuto ? '<span style="color:var(--color-brand); font-size:0.95rem;">⚡</span>' : ''}
                <span>${escapeHtml(p.name)}</span>
              </div>
              <span style="font-size:0.72rem; color:var(--text-dim); font-family:var(--font-mono);">${escapeHtml(p.id)}</span>
            </td>
            <td style="text-align:center;">${catBadge}</td>
            <td>
              <span style="font-family:var(--font-mono); font-size:0.78rem; color:var(--text-heading); display:inline-block; max-width:195px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${escapeHtml(p.default_model)}">
                ${escapeHtml(p.default_model)}
              </span>
            </td>
            <td style="text-align:center;">${badgeHtml}</td>
            <td>${authHtml}</td>
            <td style="text-align:right; white-space:nowrap;">
              <div style="display:inline-flex; align-items:center; justify-content:flex-end; gap:0.35rem;">
                ${!p.is_active 
                  ? `<button class="btn-brand btn-sm" onclick="setActiveProvider('${p.id}')">Select</button>` 
                  : '<span class="badge badge-green" style="padding:0.25rem 0.5rem; font-size:0.68rem;">● ACTIVE</span>'
                }
                <button class="btn-outline btn-sm" onclick="testProviderPing('${p.id}', this)" title="Test latency">Ping</button>
                <button class="btn-outline btn-sm" onclick="openConfigModal('${p.id}')" title="Configure credentials">Config</button>
              </div>
            </td>
          </tr>
        `;
      });
      tbody.innerHTML = html;
    }

    function updateQuickSelect(providers) {
      const sel = document.getElementById('quick-provider-select');
      const playSel = document.getElementById('play-model');
      if (!sel) return;

      sel.innerHTML = '';
      let playModelsHtml = '';

      providers.forEach((p, idx) => {
        const opt = document.createElement('option');
        opt.value = p.id;
        opt.textContent = `[${idx}] ${p.name} (${p.default_model})`;
        if (p.is_active) {
          opt.selected = true;
          document.getElementById('footer-active-prov').textContent = `${p.name}`;
          document.getElementById('kpi-active-pool').textContent = p.name;
        }
        sel.appendChild(opt);

        // Add models to playground selector
        p.available_models.forEach(m => {
          playModelsHtml += `<option value="${escapeHtml(m)}">${escapeHtml(p.name)} · ${escapeHtml(m)}</option>`;
        });
      });

      if (playSel && playModelsHtml) {
        playSel.innerHTML = playModelsHtml;
      }
    }

    function quickSwitchProvider(provId) {
      setActiveProvider(provId);
    }

    function setActiveProvider(provId) {
      fetch('/api/providers/select', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: provId })
      })
      .then(r => r.json())
      .then(data => {
        loadProviders();
        refreshRouterState();
      })
      .catch(err => alert("Failed to switch provider: " + err));
    }

    function testProviderPing(provId, btnEl) {
      const origText = btnEl.textContent;
      btnEl.textContent = '...';
      fetch('/api/providers/test', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: provId })
      })
      .then(r => r.json())
      .then(res => {
        if (res.status === 'ok') {
          btnEl.textContent = '✓ OK';
          btnEl.className = 'btn-outline btn-sm badge-green';
        } else {
          btnEl.textContent = '✖ Err';
          btnEl.className = 'btn-outline btn-sm badge-red';
        }
        setTimeout(() => { btnEl.textContent = origText; btnEl.className = 'btn-outline btn-sm'; }, 3000);
      })
      .catch(() => {
        btnEl.textContent = '✖ Err';
        setTimeout(() => { btnEl.textContent = origText; }, 3000);
      });
    }

    // --- Config Modal ---
    function openConfigModal(provId) {
      const p = providersCache.find(x => x.id === provId);
      if (!p) return;

      document.getElementById('modal-prov-id').value = p.id;
      document.getElementById('modal-prov-title').textContent = `Configure ${p.name} (${p.id})`;
      document.getElementById('modal-api-key').value = '';
      document.getElementById('modal-base-url').value = p.custom_base_url || '';

      const modelSel = document.getElementById('modal-model-select');
      modelSel.innerHTML = '';
      p.available_models.forEach(m => {
        const opt = document.createElement('option');
        opt.value = m;
        opt.textContent = m;
        if (m === p.default_model) opt.selected = true;
        modelSel.appendChild(opt);
      });

      document.getElementById('config-modal').style.display = 'flex';
    }

    function closeConfigModal() {
      document.getElementById('config-modal').style.display = 'none';
    }

    function saveProviderConfig() {
      const id = document.getElementById('modal-prov-id').value;
      const apiKey = document.getElementById('modal-api-key').value;
      const baseUrl = document.getElementById('modal-base-url').value;
      const model = document.getElementById('modal-model-select').value;
      const setActive = document.getElementById('modal-set-active').checked;

      const payload = { id, model, set_active: setActive };
      if (apiKey) payload.api_key = apiKey;
      if (baseUrl) payload.base_url = baseUrl;

      fetch('/api/providers/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      })
      .then(r => r.json())
      .then(() => {
        closeConfigModal();
        loadProviders();
      })
      .catch(err => alert("Error saving config: " + err));
    }

    // --- Search & Filter Providers ---
    function filterProviders(cat, tabEl) {
      document.querySelectorAll('.filter-tab').forEach(t => t.classList.remove('active'));
      tabEl.classList.add('active');

      if (cat === 'all') {
        renderProvidersTable(providersCache);
      } else if (cat === 'smart') {
        renderProvidersTable(providersCache.filter(p => p.id === 'auto'));
      } else if (cat === 'ready') {
        renderProvidersTable(providersCache.filter(p => p.is_configured || p.is_active));
      } else if (cat === 'commercial') {
        renderProvidersTable(providersCache.filter(p => p.category === 'Commercial'));
      } else if (cat === 'local') {
        renderProvidersTable(providersCache.filter(p => p.auth_type === 'none' && p.id !== 'auto'));
      } else if (cat === 'cloud') {
        renderProvidersTable(providersCache.filter(p => p.category.includes('Cloud')));
      }
    }

    function searchProviders(q) {
      const term = q.toLowerCase();
      const filtered = providersCache.filter(p =>
        p.id.toLowerCase().includes(term) ||
        p.name.toLowerCase().includes(term) ||
        p.default_model.toLowerCase().includes(term)
      );
      renderProvidersTable(filtered);
    }

    // --- Router Stats & Breakers ---
    function refreshRouterState() {
      fetch('/api/router/stats')
        .then(r => r.json())
        .then(data => {
          const saved = data.total_tokens_saved || 0;
          document.getElementById('top-tokens-saved').textContent = formatCommas(saved);
          document.getElementById('kpi-tokens-saved').textContent = formatCommas(saved);
          document.getElementById('router-rtk-saved').textContent = formatCommas(saved) + ' tok';

          updateOverviewBreakers(data.providers || []);
          updateRouterFullTable(data.providers || []);
        })
        .catch(err => console.error("Router stats error:", err));
    }

    function updateOverviewBreakers(records) {
      const tbody = document.getElementById('overview-breakers-table');
      if (!tbody) return;

      if (!records || records.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6" style="text-align:center; padding:1.25rem;">All upstream providers healthy. Auto-failover armed.</td></tr>';
        return;
      }

      let html = '';
      records.slice(0, 8).forEach(r => {
        const stateBadge = r.is_healthy
          ? '<span class="badge badge-green">● HEALTHY</span>'
          : '<span class="badge badge-red">✖ COOLDOWN</span>';

        html += `
          <tr>
            <td><strong>${escapeHtml(r.name)}</strong></td>
            <td style="text-align:center;">${stateBadge}</td>
            <td style="text-align:right; font-family:var(--font-mono);">${r.successful_requests}</td>
            <td style="text-align:right; font-family:var(--font-mono);">${r.failed_requests}</td>
            <td style="text-align:right; font-family:var(--font-mono);">${r.avg_latency_ms > 0 ? r.avg_latency_ms + ' ms' : '---'}</td>
            <td>${r.cooldown_remaining_secs > 0 ? `<span class="badge badge-amber">${r.cooldown_remaining_secs}s left</span>` : '<span style="color:var(--text-dim);">Ready</span>'}</td>
          </tr>
        `;
      });
      tbody.innerHTML = html;
    }

    function updateRouterFullTable(records) {
      const tbody = document.getElementById('router-full-table');
      if (!tbody) return;

      if (!records || records.length === 0) {
        tbody.innerHTML = '<tr><td colspan="7" style="text-align:center; padding:1.5rem;">No breaker events recorded. All providers available.</td></tr>';
        return;
      }

      let html = '';
      records.forEach(r => {
        const stateBadge = r.is_healthy
          ? '<span class="badge badge-green">● HEALTHY</span>'
          : '<span class="badge badge-red">✖ COOLDOWN</span>';

        html += `
          <tr>
            <td><strong>${escapeHtml(r.name)}</strong> <span style="font-size:0.75rem; color:var(--text-dim); font-family:var(--font-mono);">(${escapeHtml(r.id)})</span></td>
            <td style="text-align:center;">${stateBadge}</td>
            <td style="text-align:right; font-family:var(--font-mono);">${r.total_requests}</td>
            <td style="text-align:right; font-family:var(--font-mono);">${r.successful_requests}</td>
            <td style="text-align:right; font-family:var(--font-mono);">${r.failed_requests}</td>
            <td style="text-align:right; font-family:var(--font-mono);">${r.avg_latency_ms > 0 ? r.avg_latency_ms + ' ms' : '---'}</td>
            <td style="text-align:center;">${r.cooldown_remaining_secs > 0 ? `<span class="badge badge-amber">${r.cooldown_remaining_secs}s cooldown</span>` : '<span style="color:var(--color-emerald); font-weight:600;">Normal</span>'}</td>
          </tr>
        `;
      });
      tbody.innerHTML = html;
    }

    function resetAllBreakers() {
      fetch('/api/router/reset', { method: 'POST' })
        .then(r => r.json())
        .then(() => {
          refreshRouterState();
          alert("All provider circuit breakers have been reset to Healthy.");
        })
        .catch(err => alert("Reset error: " + err));
    }

    // --- Simulators ---
    function runSimCompress() {
      const txt = document.getElementById('sim-compress-input').value;
      if (!txt) return;
      fetch('/api/router/compress', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt: txt })
      })
      .then(r => r.json())
      .then(res => {
        document.getElementById('sim-compress-res').textContent =
          `Saved: ${res.tokens_saved} tokens (${res.savings_percent.toFixed(1)}%) | ${res.original_tokens} → ${res.compressed_tokens}`;
        refreshRouterState();
      });
    }

    function runSimClassify() {
      const txt = document.getElementById('sim-classify-input').value;
      if (!txt) return;
      fetch('/api/router/classify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt: txt })
      })
      .then(r => r.json())
      .then(res => {
        document.getElementById('sim-classify-res').textContent =
          `Tier: ${res.tier_name} | Recommended: ${res.recommended_model}`;
      });
    }

    // --- Playground Execution ---
    function sendPlaygroundRequest() {
      const model = document.getElementById('play-model').value;
      const sys = document.getElementById('play-sys').value;
      const prompt = document.getElementById('play-prompt').value;
      const temp = parseFloat(document.getElementById('play-temp').value) || 0.7;

      if (!prompt.trim()) {
        alert("Please enter a user prompt.");
        return;
      }

      const btn = document.getElementById('btn-play-send');
      const respEl = document.getElementById('play-response');
      const pill = document.getElementById('play-telemetry-pill');

      btn.disabled = true;
      btn.innerHTML = '<span>⏳ Generating...</span>';
      respEl.textContent = 'Dispatching prompt through Nityaksha Gateway router...';

      const start = Date.now();
      const messages = [];
      if (sys) messages.push({ role: "system", content: sys });
      messages.push({ role: "user", content: prompt });

      fetch('/v1/chat/completions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model: model,
          messages: messages,
          temperature: temp
        })
      })
      .then(r => r.json().then(data => ({ status: r.status, data })))
      .then(({ status, data }) => {
        const latency = Date.now() - start;
        btn.disabled = false;
        btn.innerHTML = '<span>⚡ Execute Prompt</span>';

        if (status === 200 && data.choices && data.choices[0]) {
          const content = data.choices[0].message.content;
          respEl.textContent = content;

          const u = data.usage || {};
          pill.style.display = 'inline-flex';
          pill.className = 'badge badge-green';
          pill.textContent = `Latency: ${latency}ms | Tokens: ${u.prompt_tokens || 0} in / ${u.completion_tokens || 0} out`;

          // Add to recent logs table
          appendLogEntry(new Date().toLocaleTimeString(), 'POST /v1/chat/completions', data.model || model, '200 OK', `${latency}ms`, `${u.prompt_tokens || 0} / ${u.completion_tokens || 0}`);
        } else {
          const err = data.error ? data.error.message : JSON.stringify(data);
          respEl.textContent = `Error: ${err}`;
          pill.style.display = 'inline-flex';
          pill.className = 'badge badge-red';
          pill.textContent = `Status: ${status} Error (${latency}ms)`;

          appendLogEntry(new Date().toLocaleTimeString(), 'POST /v1/chat/completions', model, `${status} Error`, `${latency}ms`, '---');
        }
        refreshRouterState();
      })
      .catch(err => {
        btn.disabled = false;
        btn.innerHTML = '<span>⚡ Execute Prompt</span>';
        respEl.textContent = "Network / Gateway error: " + err;
      });
    }

    function setPlayPrompt(text) {
      document.getElementById('play-prompt').value = text;
    }

    function copyPlayResponse() {
      const txt = document.getElementById('play-response').textContent;
      navigator.clipboard.writeText(txt).then(() => alert("Response copied to clipboard!"));
    }

    function appendLogEntry(time, ep, model, status, lat, tokens) {
      const tbody = document.getElementById('logs-table-body');
      if (!tbody) return;

      const is200 = status.includes('200');
      const badgeCls = is200 ? 'badge-green' : 'badge-red';

      const row = `
        <tr>
          <td style="font-family:var(--font-mono); font-size:0.75rem;">${time}</td>
          <td><code>${ep}</code></td>
          <td><span style="font-family:var(--font-mono);">${escapeHtml(model)}</span></td>
          <td><span class="badge ${badgeCls}">${status}</span></td>
          <td style="font-family:var(--font-mono);">${lat}</td>
          <td style="font-family:var(--font-mono);">${tokens}</td>
        </tr>
      `;

      if (tbody.innerHTML.includes('No recent requests')) {
        tbody.innerHTML = row;
      } else {
        tbody.innerHTML = row + tbody.innerHTML;
      }
    }

    function clearLogsTable() {
      document.getElementById('logs-table-body').innerHTML = '<tr><td colspan="6" style="text-align:center; padding:1.5rem;">Logs cleared.</td></tr>';
    }

    // --- Helpers ---
    function formatCommas(n) {
      return (n || 0).toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
    }

    function escapeHtml(s) {
      return String(s || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }

    // --- Initialize ---
    window.addEventListener('DOMContentLoaded', () => {
      // Initialize theme state (Default Light Mode / LocalStorage)
      const currentTheme = document.documentElement.getAttribute('data-theme') || 'light';
      applyTheme(currentTheme);

      const origin = window.location.origin || 'http://46.62.188.249:8080';
      const host = window.location.host || '46.62.188.249:8080';

      const badge = document.getElementById('gateway-status-badge');
      if (badge) badge.textContent = `Gateway Online · ${host}`;

      document.querySelectorAll('code, pre').forEach(el => {
        if (el.textContent.includes('http://localhost:8080')) {
          el.textContent = el.textContent.replaceAll('http://localhost:8080', origin);
        }
      });

      loadProviders();
      refreshRouterState();
      setInterval(refreshRouterState, 10000);
    });
  </script>
</body>
</html>
"##;
