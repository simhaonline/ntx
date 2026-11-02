//! Nityaksha Interactive REPL, Onboarding Wizard, Theme Engine, and Directory Trust Manager
//! Provides guided prompts, color theme customization, workspace trust verification, and AI completions.

use crate::db::MemoryEngine;
use crate::provider::{GoogleOAuthEngine, ProviderRegistry};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct CliPalette {
    pub id: &'static str,
    pub name: &'static str,
    pub primary: &'static str,
    pub accent: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub muted: &'static str,
    pub dim: &'static str,
    pub bold: &'static str,
    pub reset: &'static str,
    pub prompt_glyph: &'static str,
}

pub const THEMES: &[CliPalette] = &[
    CliPalette {
        id: "rust",
        name: "🦀 Rust Orange & Deep Navy (Official Nityaksha Brand)",
        primary: "\x1b[38;2;192;90;40m",
        accent: "\x1b[38;2;14;165;233m",
        success: "\x1b[38;2;16;185;129m",
        warning: "\x1b[38;2;245;158;11m",
        muted: "\x1b[38;2;100;116;139m",
        dim: "\x1b[90m",
        bold: "\x1b[1m",
        reset: "\x1b[0m",
        prompt_glyph: "🦀 ntx ❯",
    },
    CliPalette {
        id: "cyan",
        name: "💎 Luminous Cyan & Steel (High Visibility / Modern)",
        primary: "\x1b[96m",
        accent: "\x1b[94m",
        success: "\x1b[92m",
        warning: "\x1b[93m",
        muted: "\x1b[37m",
        dim: "\x1b[90m",
        bold: "\x1b[1m",
        reset: "\x1b[0m",
        prompt_glyph: "💎 ntx ❯",
    },
    CliPalette {
        id: "emerald",
        name: "🟢 Matrix Emerald (Monochrome Terminal Classic)",
        primary: "\x1b[92m",
        accent: "\x1b[32m",
        success: "\x1b[92m",
        warning: "\x1b[33m",
        muted: "\x1b[32m",
        dim: "\x1b[90m",
        bold: "\x1b[1m",
        reset: "\x1b[0m",
        prompt_glyph: "🟢 ntx ❯",
    },
    CliPalette {
        id: "neon",
        name: "⚡ Cyberpunk Neon (Vibrant Purple & Gold)",
        primary: "\x1b[95m",
        accent: "\x1b[93m",
        success: "\x1b[92m",
        warning: "\x1b[91m",
        muted: "\x1b[94m",
        dim: "\x1b[90m",
        bold: "\x1b[1m",
        reset: "\x1b[0m",
        prompt_glyph: "⚡ ntx ❯",
    },
    CliPalette {
        id: "plain",
        name: "⚪ Clean Minimal (Plain ANSI / No Formatting)",
        primary: "",
        accent: "",
        success: "",
        warning: "",
        muted: "",
        dim: "",
        bold: "",
        reset: "",
        prompt_glyph: "ntx >",
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveConfig {
    pub theme_id: String,
    pub trusted_dirs: Vec<String>,
    pub onboarded: bool,
    pub active_bot: Option<String>,
}

impl Default for InteractiveConfig {
    fn default() -> Self {
        Self {
            theme_id: "rust".to_string(),
            trusted_dirs: Vec::new(),
            onboarded: false,
            active_bot: None,
        }
    }
}

pub struct InteractiveShell {
    pub engine: Arc<MemoryEngine>,
    pub config: InteractiveConfig,
    pub current_dir: String,
    pub git_branch: Option<String>,
}

impl InteractiveShell {
    pub fn new(engine: Arc<MemoryEngine>) -> Self {
        let config = Self::load_config();
        let current_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());
        let git_branch = Self::detect_git_branch();

        Self {
            engine,
            config,
            current_dir,
            git_branch,
        }
    }

    pub fn config_file() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home).join(".config/nityaksha/interactive.json");
            if let Some(parent) = p.parent() {
                let _ = fs::create_dir_all(parent);
            }
            return p;
        }
        PathBuf::from("interactive.json")
    }

    pub fn load_config() -> InteractiveConfig {
        let path = Self::config_file();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<InteractiveConfig>(&data) {
                    return cfg;
                }
            }
        }
        InteractiveConfig::default()
    }

    pub fn save_config(&self) {
        let path = Self::config_file();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            let _ = fs::write(&path, json);
        }
    }

    pub fn get_theme(&self) -> &'static CliPalette {
        THEMES
            .iter()
            .find(|t| t.id == self.config.theme_id)
            .unwrap_or(&THEMES[0])
    }

    pub fn detect_git_branch() -> Option<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()?;
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() {
                return Some(branch);
            }
        }
        None
    }

    pub fn is_current_dir_trusted(&self) -> bool {
        self.config.trusted_dirs.iter().any(|d| d == &self.current_dir)
    }

    /// Onboarding Wizard: Folder Trust & Color Theme Selection
    pub fn run_onboarding_wizard_if_needed(&mut self) {
        let theme = self.get_theme();

        // 1. Check Directory Trust
        if !self.is_current_dir_trusted() {
            println!("\n{}┌────────────────────────────────────────────────────────────────────────┐{}", theme.accent, theme.reset);
            println!("{}│ 🛡️  WORKSPACE TRUST VERIFICATION                                       │{}", theme.accent, theme.reset);
            println!("{}├────────────────────────────────────────────────────────────────────────┤{}", theme.accent, theme.reset);
            println!("│ Directory: {}{:<58}{} │", theme.bold, self.current_dir, theme.reset);
            println!("│                                                                        │");
            println!("│ Do you trust the authors and scripts in this directory?                │");
            println!("│ Trusting this workspace permits Nityaksha to inspect repository AST,   │");
            println!("│ execute isolated tools, and apply code diffs within security gates.    │");
            println!("│                                                                        │");
            println!("│   [Y] Yes, trust this folder (Recommended)                             │");
            println!("│   [N] No, run in strictly read-only mode                               │");
            println!("{}└────────────────────────────────────────────────────────────────────────┘{}", theme.accent, theme.reset);

            print!("{}Trust this directory? [Y/n]:{} ", theme.bold, theme.reset);
            let _ = io::stdout().flush();
            let mut answer = String::new();
            let _ = io::stdin().read_line(&mut answer);
            let trimmed = answer.trim().to_lowercase();
            if trimmed.is_empty() || trimmed == "y" || trimmed == "yes" {
                self.config.trusted_dirs.push(self.current_dir.clone());
                println!("{}✓ Directory marked as trusted.{}", theme.success, theme.reset);
            } else {
                println!("{}! Directory marked as unconfined read-only. Destructive actions blocked.{}", theme.warning, theme.reset);
            }
            self.save_config();
        }

        // 2. First-Time Setup Theme Selection
        if !self.config.onboarded {
            self.interactive_select_theme();
            self.config.onboarded = true;
            self.save_config();
        }
    }

    /// Interactive Theme Selector
    pub fn interactive_select_theme(&mut self) {
        println!("\n┌────────────────────────────────────────────────────────────────────────┐");
        println!("│ 🎨 SELECT CLI COLOR PALETTE & PRESENTATION THEME                       │");
        println!("├────────────────────────────────────────────────────────────────────────┤");
        for (i, t) in THEMES.iter().enumerate() {
            println!("│   [{}] {}{:<60}{} │", i + 1, t.primary, t.name, t.reset);
        }
        println!("└────────────────────────────────────────────────────────────────────────┘");

        print!("Select palette [1-{} | Enter to keep current]: ", THEMES.len());
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let trimmed = input.trim();
        if let Ok(idx) = trimmed.parse::<usize>() {
            if idx >= 1 && idx <= THEMES.len() {
                self.config.theme_id = THEMES[idx - 1].id.to_string();
                self.save_config();
                let theme = self.get_theme();
                println!("{}✓ Theme set to: {}{}", theme.success, theme.name, theme.reset);
                return;
            }
        }
        println!("Keeping current theme: {}", self.get_theme().name);
    }

    /// Launch Main Interactive Prompt Shell
    pub fn run_interactive_loop(&mut self, session_id: Option<String>) {
        self.run_onboarding_wizard_if_needed();

        let sid = session_id.unwrap_or_else(|| format!("sess-{}", chrono::Utc::now().timestamp_millis()));
        let mut turn_counter = 0;

        self.print_welcome_header(&sid);

        loop {
            let theme = self.get_theme();
            let trusted_label = if self.is_current_dir_trusted() { "TRUSTED ✓" } else { "READ-ONLY" };
            let git_info = self.git_branch.as_deref().unwrap_or("none");
            let active_p = ProviderRegistry::active();

            // Status Line Prompt
            print!(
                "\n{dim}[dir: {dir} | git: {git} | {trusted}] [{model}]{reset}\n{primary}{glyph}{reset} ",
                dim = theme.dim,
                dir = clean_path(&self.current_dir, 32),
                git = git_info,
                trusted = trusted_label,
                model = active_p.default_model,
                primary = theme.primary,
                glyph = theme.prompt_glyph,
                reset = theme.reset
            );
            let _ = io::stdout().flush();

            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }

            let input = line.trim();
            if input.is_empty() {
                continue;
            }

            // Check for exit
            if input == "/exit" || input == "/quit" || input == "exit" || input == "quit" {
                println!("{}Exiting Nityaksha Concierge Platform. Goodbye!{}", theme.muted, theme.reset);
                break;
            }

            // Handle Slash Commands
            if input.starts_with('/') {
                self.handle_slash_command(input, &sid);
                continue;
            }

            // Direct Natural Language Prompt Execution
            turn_counter += 1;
            self.execute_user_prompt(input, &sid, turn_counter);
        }
    }

    fn print_welcome_header(&self, sid: &str) {
        let theme = self.get_theme();
        let active_p = ProviderRegistry::active();
        let trusted_str = if self.is_current_dir_trusted() {
            format!("{}TRUSTED ✓{}", theme.success, theme.reset)
        } else {
            format!("{}READ-ONLY (Untrusted){}", theme.warning, theme.reset)
        };

        println!("\n{bold}================================================================================{reset}", bold = theme.bold, reset = theme.reset);
        println!("  {primary}NITYAKSHA — An AI Concierge Platform (v1.0.0 Pure Rust Native){reset}", primary = theme.primary, reset = theme.reset);
        println!("{bold}================================================================================{reset}", bold = theme.bold, reset = theme.reset);
        println!("  Session ID  : {accent}{}{reset}", sid, accent = theme.accent, reset = theme.reset);
        println!("  Workspace   : {}{}{} ({})", theme.bold, self.current_dir, theme.reset, trusted_str);
        if let Some(ref branch) = self.git_branch {
            println!("  Git Branch  : {accent}{}{reset}", branch, accent = theme.accent, reset = theme.reset);
        }
        println!("  AI Provider : {primary}{}{reset} ({})", active_p.name, active_p.default_model, primary = theme.primary, reset = theme.reset);
        println!("  Auth State  : {}", active_p.auth_summary);
        println!("  Theme       : {}", theme.name);
        println!("--------------------------------------------------------------------------------");
        println!("  {}Type your prompt to chat, or use slash commands:{}", theme.dim, theme.reset);
        println!("  {a}/help{r} • {a}/provider{r} • {a}/auth{r} • {a}/theme{r} • {a}/model{r} • {a}/bot{r} • {a}/doctor{r} • {a}/exit{r}", a = theme.accent, r = theme.reset);
        println!("{bold}================================================================================{reset}", bold = theme.bold, reset = theme.reset);
    }

    fn handle_slash_command(&mut self, input: &str, sid: &str) {
        let theme = self.get_theme();
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0];

        match cmd {
            "/help" | "/?" => {
                println!("\n{}Interactive Guided Commands & Hotkeys:{}", theme.bold, theme.reset);
                println!("  {accent}/help{reset}                - Show this reference guide", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/provider{reset}            - Switch or configure AI provider (21 backends)", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/auth{reset}                - Check or login with Google OAuth 2.0", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/theme{reset}               - Switch CLI color palette and theme", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/trust{reset}               - Manage workspace folder trust", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/bot [id]{reset}            - Switch to autonomous bot persona", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/workflow [id]{reset}       - Execute or inspect DAG workflows", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/model [name]{reset}        - Switch active model", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/status{reset}              - Framework health and runtime metrics", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/doctor{reset}              - Run security and cgroup sandbox audit", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/memory <query>{reset}      - Search episodic memory vectors & BM25", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/clear{reset}               - Clear screen", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/exit{reset}                - Exit interactive shell", accent = theme.accent, reset = theme.reset);
            }
            "/theme" => {
                self.interactive_select_theme();
            }
            "/trust" => {
                if self.is_current_dir_trusted() {
                    println!("{}Workspace '{}' is TRUSTED.{}", theme.success, self.current_dir, theme.reset);
                    print!("Do you want to revoke trust? [y/N]: ");
                    let _ = io::stdout().flush();
                    let mut ans = String::new();
                    let _ = io::stdin().read_line(&mut ans);
                    if ans.trim().to_lowercase() == "y" {
                        self.config.trusted_dirs.retain(|d| d != &self.current_dir);
                        self.save_config();
                        println!("{}Trust revoked for this folder.{}", theme.warning, theme.reset);
                    }
                } else {
                    println!("{}Workspace '{}' is UNTRUSTED.{}", theme.warning, self.current_dir, theme.reset);
                    print!("Mark as trusted? [Y/n]: ");
                    let _ = io::stdout().flush();
                    let mut ans = String::new();
                    let _ = io::stdin().read_line(&mut ans);
                    if ans.trim().is_empty() || ans.trim().to_lowercase() == "y" {
                        self.config.trusted_dirs.push(self.current_dir.clone());
                        self.save_config();
                        println!("{}✓ Directory marked as trusted.{}", theme.success, theme.reset);
                    }
                }
            }
            "/provider" | "/providers" => {
                if parts.len() > 1 {
                    let id = parts[1];
                    match ProviderRegistry::set_active(id) {
                        Ok(()) => {
                            let active = ProviderRegistry::active();
                            println!("{}✓ Switched active provider to: {} ({}){}", theme.success, active.name, active.default_model, theme.reset);
                        }
                        Err(e) => eprintln!("{}Error: {}{}", theme.warning, e, theme.reset),
                    }
                } else {
                    let providers = ProviderRegistry::list_all();
                    println!("\n{}AI Provider Gateway (Select active provider):{}", theme.bold, theme.reset);
                    for (i, p) in providers.iter().enumerate() {
                        let active_mark = if p.is_active { "*" } else { " " };
                        println!("  [{:2}] {}{:<12} - {:<26} ({})", i + 1, active_mark, p.id, p.name, p.default_model);
                    }
                    print!("\nEnter provider ID or number to activate [Enter to cancel]: ");
                    let _ = io::stdout().flush();
                    let mut choice = String::new();
                    let _ = io::stdin().read_line(&mut choice);
                    let trimmed = choice.trim();
                    if let Ok(idx) = trimmed.parse::<usize>() {
                        if idx >= 1 && idx <= providers.len() {
                            let _ = ProviderRegistry::set_active(&providers[idx - 1].id);
                            let active = ProviderRegistry::active();
                            println!("{}✓ Switched active provider to: {} ({}){}", theme.success, active.name, active.default_model, theme.reset);
                            return;
                        }
                    } else if !trimmed.is_empty() {
                        let _ = ProviderRegistry::set_active(trimmed);
                        let active = ProviderRegistry::active();
                        println!("{}✓ Switched active provider to: {} ({}){}", theme.success, active.name, active.default_model, theme.reset);
                    }
                }
            }
            "/auth" => {
                if parts.len() > 1 && parts[1] == "login" {
                    println!("[INFO] Starting Google OAuth login for Gemini...");
                    let _ = GoogleOAuthEngine::interactive_login(None, None);
                } else {
                    println!("\n{}Google OAuth 2.0 Status:{}", theme.bold, theme.reset);
                    if let Some(creds) = GoogleOAuthEngine::load_credentials() {
                        let email = creds.email.as_deref().unwrap_or("Connected Account");
                        let mins = (creds.expires_at - chrono::Utc::now().timestamp()) / 60;
                        println!("  {}Status       : AUTHENTICATED ✓{}", theme.success, theme.reset);
                        println!("  Account      : {}", email);
                        println!("  Expires In   : ~{} minutes (Auto-Refresh Enabled)", mins);
                    } else {
                        println!("  {}Status       : NOT AUTHENTICATED{}", theme.warning, theme.reset);
                        println!("  Type '/auth login' or 'ntx auth login google' to connect your Google account.");
                    }
                }
            }
            "/bot" => {
                if parts.len() > 1 {
                    let bot_id = parts[1];
                    self.config.active_bot = Some(bot_id.to_string());
                    self.save_config();
                    println!("{}✓ Switched persona to: {}{}", theme.success, bot_id, theme.reset);
                } else {
                    let bots = self.engine.relational.list_bots();
                    println!("\n{}Available Preconfigured Bots:{}", theme.bold, theme.reset);
                    for b in &bots {
                        println!("  • {:<26} - {} ({})", b.id, b.name, b.category);
                    }
                    println!("\nType '/bot <id>' to switch persona.");
                }
            }
            "/workflow" => {
                let wfs = self.engine.relational.list_workflows();
                println!("\n{}Registered Workflow DAGs:{}", theme.bold, theme.reset);
                for wf in &wfs {
                    println!("  • {:<28} - {} ({} steps)", wf.id, wf.name, wf.nodes.len());
                }
                println!("\nRun 'ntx workflow run <id>' to launch.");
            }
            "/doctor" => {
                println!("\n{}Running Nityaksha Security & Governance Audit...{}", theme.bold, theme.reset);
                let _ = Command::new("ntx").arg("doctor").status();
            }
            "/status" => {
                let _ = Command::new("ntx").arg("status").status();
            }
            "/clear" => {
                print!("\x1b[2J\x1b[H");
                let _ = io::stdout().flush();
                self.print_welcome_header(sid);
            }
            _ => {
                println!("{}Unknown slash command '{}'. Type /help for available commands.{}", theme.warning, cmd, theme.reset);
            }
        }
    }

    fn execute_user_prompt(&self, prompt: &str, sid: &str, turn: usize) {
        let theme = self.get_theme();
        let active_p = ProviderRegistry::active();
        let start = Instant::now();

        println!("{}Dispatching turn #{} to {} ({})...{}", theme.dim, turn, active_p.name, active_p.default_model, theme.reset);

        let system_prompt = if let Some(ref bot_id) = self.config.active_bot {
            if let Some(bot) = self.engine.relational.get_bot(bot_id) {
                bot.system_prompt_template
            } else {
                "Objective Technical Execution Mode. Pure Rust Native.".to_string()
            }
        } else {
            "Objective Technical Execution Mode. Pure Rust Native.".to_string()
        };

        match ProviderRegistry::dispatch_prompt(prompt, Some(&system_prompt), 0.7) {
            Ok(output) => {
                let elapsed = start.elapsed().as_millis();
                println!("\n{}{}{}", theme.bold, output, theme.reset);
                println!("\n{dim}─── [{provider} | model={model} | {ms}ms | session={sid}] ───{reset}",
                    dim = theme.dim,
                    provider = active_p.id,
                    model = active_p.default_model,
                    ms = elapsed,
                    sid = clean_path(sid, 16),
                    reset = theme.reset
                );
            }
            Err(e) => {
                eprintln!("\n{}Execution Error: {}{}", theme.warning, e, theme.reset);
                println!("{}Tip: Run '/auth' for Google OAuth or '/provider' to switch models.{}", theme.dim, theme.reset);
            }
        }
    }
}

fn clean_path(path: &str, max_len: usize) -> String {
    if path.chars().count() > max_len {
        let tail: String = path.chars().rev().take(max_len.saturating_sub(3)).collect();
        format!("...{}", tail.chars().rev().collect::<String>())
    } else {
        path.to_string()
    }
}
