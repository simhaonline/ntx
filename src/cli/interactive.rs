//! Nityaksha Interactive REPL, Onboarding Wizard, Theme Engine, and Directory Trust Manager
//! Provides guided prompts, color theme customization, workspace trust verification, and AI completions.

use crate::db::MemoryEngine;
use crate::provider::ProviderRegistry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

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
    pub session_input_tokens: usize,
    pub session_output_tokens: usize,
    pub session_total_tokens: usize,
    pub conversation_history: Vec<(String, String)>,
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
            session_input_tokens: 0,
            session_output_tokens: 0,
            session_total_tokens: 0,
            conversation_history: Vec::new(),
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
            let cols = terminal_cols().clamp(72, 160);
            let border = "═".repeat(cols);
            let thin_border = "─".repeat(cols);

            println!("\n{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
            println!("  {accent}🛡️  WORKSPACE TRUST VERIFICATION{reset}", accent = theme.accent, reset = theme.reset);
            println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
            println!("  Directory   : {bold}{}{reset}", self.current_dir, bold = theme.bold, reset = theme.reset);
            println!("  Description : Do you trust the authors and scripts in this directory?");
            println!("                Trusting permits Nityaksha to inspect repository AST,");
            println!("                execute isolated tools, and apply code diffs within security gates.");
            println!("{dim}{thin_border}{reset}", dim = theme.dim, thin_border = thin_border, reset = theme.reset);
            println!("    [Y] Yes, trust this folder (Recommended)");
            println!("    [N] No, run in strictly read-only mode");
            println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);

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
        let theme = self.get_theme();
        let cols = terminal_cols().clamp(72, 160);
        let border = "═".repeat(cols);

        println!("\n{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
        println!("  {accent}🎨 SELECT CLI COLOR PALETTE & PRESENTATION THEME{reset}", accent = theme.accent, reset = theme.reset);
        println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
        for (i, t) in THEMES.iter().enumerate() {
            println!("    [{}] {}{}{}", i + 1, t.primary, t.name, t.reset);
        }
        println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);

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
            let ctx_limit = active_p.context_window_for_model(&active_p.default_model);

            let history_tokens: usize = self.conversation_history
                .iter()
                .map(|(u, a)| (u.len() + a.len()) / 4)
                .sum();
            let ctx_pct = if ctx_limit > 0 {
                (history_tokens as f64 / ctx_limit as f64 * 100.0).min(100.0)
            } else {
                0.0
            };

            // Status Line Prompt with live model, context percentage, and token counter
            print!(
                "\n{dim}[dir: {dir} | git: {git} | {trusted}] [{model} | ctx: {pct:.2}% of {ctx_win} | tokens: {tok}]{reset}\n{primary}{glyph}{reset} ",
                dim = theme.dim,
                dir = clean_path(&self.current_dir, 32),
                git = git_info,
                trusted = trusted_label,
                model = active_p.default_model,
                pct = ctx_pct,
                ctx_win = format_tokens_short(ctx_limit),
                tok = format_num_commas(self.session_total_tokens),
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

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // 1. If line contains the bullet character '•' (user pasted banner command bar):
            if trimmed.contains('•') {
                println!("{}Guided Command Bar detected. Displaying reference guide:{}", theme.dim, theme.reset);
                self.handle_command("help", &sid);
                continue;
            }

            // 2. Normalize and check if input is a known CLI command (with or without '/')
            let (is_command, normalized_cmd) = Self::classify_command(trimmed);
            if is_command {
                let first_token = normalized_cmd.split_whitespace().next().unwrap_or("");
                if first_token == "exit" || first_token == "quit" || first_token == ":q" {
                    println!("{}Exiting Nityaksha Concierge Platform. Goodbye!{}", theme.muted, theme.reset);
                    break;
                }
                self.handle_command(&normalized_cmd, &sid);
                continue;
            }

            // 3. Natural Language Prompt Execution
            turn_counter += 1;
            self.execute_user_prompt(trimmed, &sid, turn_counter);
        }
    }

    pub fn print_welcome_header(&self, sid: &str) {
        let theme = self.get_theme();
        let active_p = ProviderRegistry::active();
        let cols = terminal_cols().clamp(72, 160);
        let border = "═".repeat(cols);
        let thin_border = "─".repeat(cols);
        let trusted_str = if self.is_current_dir_trusted() {
            format!("{}TRUSTED ✓{}", theme.success, theme.reset)
        } else {
            format!("{}READ-ONLY (Untrusted){}", theme.warning, theme.reset)
        };
        let ctx_limit = active_p.context_window_for_model(&active_p.default_model);

        println!("\n{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
        println!("  {primary}NITYAKSHA — An AI Concierge Platform (v1.0.0 Pure Rust Native){reset}", primary = theme.primary, reset = theme.reset);
        println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
        println!("  Session ID  : {accent}{}{reset}", sid, accent = theme.accent, reset = theme.reset);
        println!("  Workspace   : {}{}{} ({})", theme.bold, self.current_dir, theme.reset, trusted_str);
        if let Some(ref branch) = self.git_branch {
            println!("  Git Branch  : {accent}{}{reset}", branch, accent = theme.accent, reset = theme.reset);
        }
        println!("  AI Provider : {primary}{}{reset} (model: {bold}{}{reset} | context window: {accent}{} tokens / {}{reset})",
            active_p.name, active_p.default_model, format_num_commas(ctx_limit), format_tokens_short(ctx_limit),
            primary = theme.primary, bold = theme.bold, accent = theme.accent, reset = theme.reset);
        println!("  Auth State  : {}", active_p.auth_summary);
        if let Some(ref bot) = self.config.active_bot {
            println!("  Active Bot  : {accent}{}{reset}", bot, accent = theme.accent, reset = theme.reset);
        }
        println!("  Theme       : {}", theme.name);
        println!("{dim}{thin_border}{reset}", dim = theme.dim, thin_border = thin_border, reset = theme.reset);
        println!("  {}Type your prompt to chat, or execute guided commands:{}", theme.dim, theme.reset);
        println!("  {a}/help{r}  •  {a}/provider{r}  •  {a}/auth{r}  •  {a}/model{r}  •  {a}/tokens{r}  •  {a}/theme{r}  •  {a}/bot{r}  •  {a}/workflow{r}  •  {a}/doctor{r}  •  {a}/exit{r}", a = theme.accent, r = theme.reset);
        println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
    }

    fn classify_command(input: &str) -> (bool, String) {
        let trimmed = input.trim();
        if trimmed.starts_with('/') {
            return (true, trimmed[1..].trim().to_string());
        }
        let first_word = trimmed.split_whitespace().next().unwrap_or("").to_lowercase();
        match first_word.as_str() {
            "help" | "?" | "-h" | "--help" |
            "provider" | "providers" |
            "auth" | "login" | "logout" |
            "model" | "models" |
            "tokens" | "token" | "context" |
            "theme" | "themes" |
            "trust" |
            "bot" | "bots" | "persona" |
            "workflow" | "workflows" |
            "doctor" | "status" |
            "clear" | "cls" |
            "exit" | "quit" | ":q" |
            "memory" => {
                (true, trimmed.to_string())
            }
            _ => (false, trimmed.to_string()),
        }
    }

    fn handle_command(&mut self, input: &str, sid: &str) {
        let theme = self.get_theme();
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "help" | "?" | "-h" | "--help" => {
                let cols = terminal_cols().clamp(72, 160);
                let border = "═".repeat(cols);
                let thin_border = "─".repeat(cols);
                println!("\n{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
                println!("  {accent}💡 NITYAKSHA INTERACTIVE GUIDED COMMANDS REFERENCE{reset}", accent = theme.accent, reset = theme.reset);
                println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
                println!("  {accent}/help{reset}                   - Show this comprehensive reference guide", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/provider [id]{reset}          - Switch AI provider (21 backends: Gemini, Claude, OpenAI...)", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/auth [login|logout]{reset}    - Authenticate via Google OAuth or API keys", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/model [name|idx]{reset}       - Inspect or switch active model & context window", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/tokens{reset} or {accent}/context{reset}    - Display token telemetry and context window utilization", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/theme{reset}                  - Select CLI color palette (Rust, Cyan, Emerald, Neon)", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/trust{reset}                  - View or toggle workspace directory trust authorization", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/bot [id]{reset}               - Switch autonomous bot persona (coding, architect...)", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/workflow [id]{reset}          - List or trigger registered DAG workflows", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/status{reset}                 - Live system status, database stats, and engine state", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/doctor{reset}                 - Security, cgroup namespace, and connectivity audit", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/clear{reset}                  - Clear screen and reprint welcome dashboard", accent = theme.accent, reset = theme.reset);
                println!("  {accent}/exit{reset} or {accent}/quit{reset}          - Safely close interactive concierge session", accent = theme.accent, reset = theme.reset);
                println!("{dim}{thin_border}{reset}", dim = theme.dim, thin_border = thin_border, reset = theme.reset);
                println!("  {}Tip: Commands work with or without the leading '/' (e.g. 'help' or '/help').{}", theme.dim, theme.reset);
                println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
            }
            "model" | "models" => {
                let active_p = ProviderRegistry::active();
                let cols = terminal_cols().clamp(72, 160);
                let thin_border = "─".repeat(cols);

                if parts.len() > 1 {
                    let target = parts[1];
                    let chosen = if let Ok(idx) = target.parse::<usize>() {
                        if idx >= 1 && idx <= active_p.available_models.len() {
                            Some(active_p.available_models[idx - 1].clone())
                        } else {
                            None
                        }
                    } else {
                        active_p.available_models.iter()
                            .find(|m| m.eq_ignore_ascii_case(target) || m.to_lowercase().contains(&target.to_lowercase()))
                            .cloned()
                            .or_else(|| Some(target.to_string()))
                    };

                    if let Some(m) = chosen {
                        let _ = ProviderRegistry::set_model(&active_p.id, &m);
                        let ctx = active_p.context_window_for_model(&m);
                        println!("{}✓ Switched model for {} to: {} (context limit: {} tokens / {}){}",
                            theme.success, active_p.name, m, format_num_commas(ctx), format_tokens_short(ctx), theme.reset);
                    } else {
                        println!("{}Invalid model index or name. Type /model to see list.{}", theme.warning, theme.reset);
                    }
                } else {
                    println!("\n{}Available AI Models for {} (Active Provider):{}", theme.bold, active_p.name, theme.reset);
                    println!("{dim}{thin_border}{reset}", dim = theme.dim, thin_border = thin_border, reset = theme.reset);
                    for (i, m) in active_p.available_models.iter().enumerate() {
                        let is_active = m == &active_p.default_model;
                        let mark = if is_active { format!("{}* (active){}", theme.success, theme.reset) } else { "          ".to_string() };
                        let ctx = active_p.context_window_for_model(m);
                        println!("  [{}] {} {:<32} - Max Context: {:>10} tokens ({})",
                            i + 1, mark, m, format_num_commas(ctx), format_tokens_short(ctx));
                    }
                    println!("{dim}{thin_border}{reset}", dim = theme.dim, thin_border = thin_border, reset = theme.reset);
                    print!("Enter model index [1-{}] or model name [Enter to cancel]: ", active_p.available_models.len());
                    let _ = io::stdout().flush();
                    let mut choice = String::new();
                    let _ = io::stdin().read_line(&mut choice);
                    let trimmed = choice.trim();
                    if let Ok(idx) = trimmed.parse::<usize>() {
                        if idx >= 1 && idx <= active_p.available_models.len() {
                            let chosen = &active_p.available_models[idx - 1];
                            let _ = ProviderRegistry::set_model(&active_p.id, chosen);
                            let ctx = active_p.context_window_for_model(chosen);
                            println!("{}✓ Switched model for {} to: {} (context limit: {} tokens / {}){}",
                                theme.success, active_p.name, chosen, format_num_commas(ctx), format_tokens_short(ctx), theme.reset);
                        }
                    } else if !trimmed.is_empty() {
                        let _ = ProviderRegistry::set_model(&active_p.id, trimmed);
                        let ctx = active_p.context_window_for_model(trimmed);
                        println!("{}✓ Switched model for {} to: {} (context limit: {} tokens / {}){}",
                            theme.success, active_p.name, trimmed, format_num_commas(ctx), format_tokens_short(ctx), theme.reset);
                    }
                }
            }
            "tokens" | "token" | "context" => {
                self.show_token_telemetry();
            }
            "provider" | "providers" => {
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
                        let ctx = p.context_window_for_model(&p.default_model);
                        println!("  [{:2}] {}{:<12} - {:<26} (model: {:<24} | ctx: {})",
                            i + 1, active_mark, p.id, p.name, p.default_model, format_tokens_short(ctx));
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
            "auth" | "login" | "logout" => {
                if cmd == "login" || (parts.len() > 1 && parts[1] == "login") {
                    let provider = if cmd == "login" { parts.get(1).copied() } else { parts.get(2).copied() };
                    let key = if cmd == "login" { parts.get(2).copied() } else { parts.get(3).copied() };
                    let _ = ProviderRegistry::login_provider(provider, key);
                } else if cmd == "logout" || (parts.len() > 1 && parts[1] == "logout") {
                    let provider = if cmd == "logout" { parts.get(1).copied() } else { parts.get(2).copied() };
                    let _ = ProviderRegistry::logout_provider(provider);
                } else {
                    ProviderRegistry::print_auth_status();
                    println!("\n  {}Commands:{} '/auth login [provider] [key]' to authenticate, '/auth logout [provider]' to clear.", theme.dim, theme.reset);
                }
            }
            "theme" | "themes" => {
                self.interactive_select_theme();
            }
            "trust" => {
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
            "bot" | "bots" | "persona" => {
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
            "workflow" | "workflows" => {
                let wfs = self.engine.relational.list_workflows();
                println!("\n{}Registered Workflow DAGs:{}", theme.bold, theme.reset);
                for wf in &wfs {
                    println!("  • {:<28} - {} ({} steps)", wf.id, wf.name, wf.nodes.len());
                }
                println!("\nRun 'ntx workflow run <id>' to launch.");
            }
            "doctor" => {
                println!("\n{}Running Nityaksha Security & Governance Audit...{}", theme.bold, theme.reset);
                let _ = Command::new("ntx").arg("doctor").status();
            }
            "status" => {
                let _ = Command::new("ntx").arg("status").status();
            }
            "clear" | "cls" => {
                print!("\x1b[2J\x1b[H");
                let _ = io::stdout().flush();
                self.print_welcome_header(sid);
            }
            "memory" => {
                if parts.len() > 1 {
                    let query = parts[1..].join(" ");
                    println!("{}Searching episodic and semantic memory for '{}'...{}", theme.dim, query, theme.reset);
                    let results = self.engine.hybrid_rag.search(&query, None, 5, 0.5);
                    println!("\n{}Found {} memory fragments:{}", theme.bold, results.len(), theme.reset);
                    for (i, r) in results.iter().enumerate() {
                        println!("  [{}] score: {:.4} | id: {} | text: {}",
                            i + 1, r.hybrid_score, r.id, clean_path(&r.text, 60));
                    }
                } else {
                    println!("Usage: /memory <query> (e.g. /memory auth logic)");
                }
            }
            _ => {
                println!("{}Unknown command '{}'. Type /help or help for available commands.{}", theme.warning, parts[0], theme.reset);
            }
        }
    }

    fn show_token_telemetry(&self) {
        let theme = self.get_theme();
        let active_p = ProviderRegistry::active();
        let cols = terminal_cols().clamp(72, 160);
        let border = "═".repeat(cols);
        let thin_border = "─".repeat(cols);
        let ctx_limit = active_p.context_window_for_model(&active_p.default_model);
        let history_tokens: usize = self.conversation_history
            .iter()
            .map(|(u, a)| (u.len() + a.len()) / 4)
            .sum();
        let ctx_pct = if ctx_limit > 0 {
            (history_tokens as f64 / ctx_limit as f64 * 100.0).min(100.0)
        } else {
            0.0
        };
        let remaining = ctx_limit.saturating_sub(history_tokens);

        println!("\n{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
        println!("  {accent}📊 TOKEN & CONTEXT UTILIZATION TELEMETRY{reset}", accent = theme.accent, reset = theme.reset);
        println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
        println!("  Active Provider      : {bold}{}{reset}", active_p.name, bold = theme.bold, reset = theme.reset);
        println!("  Active Model         : {bold}{}{reset}", active_p.default_model, bold = theme.bold, reset = theme.reset);
        println!("  Model Context Limit  : {accent}{} tokens{reset} ({})", format_num_commas(ctx_limit), format_tokens_short(ctx_limit), accent = theme.accent, reset = theme.reset);
        println!("  Active Context Used  : {accent}{} tokens{reset} ({:.2}% of model limit)", format_num_commas(history_tokens), ctx_pct, accent = theme.accent, reset = theme.reset);
        println!("  Available Headroom   : {success}{} tokens remaining{reset}", format_num_commas(remaining), success = theme.success, reset = theme.reset);
        println!("{dim}{thin_border}{reset}", dim = theme.dim, thin_border = thin_border, reset = theme.reset);
        println!("  Session Prompt (In)  : {} tokens", format_num_commas(self.session_input_tokens));
        println!("  Session Resp (Out)   : {} tokens", format_num_commas(self.session_output_tokens));
        println!("  Session Cumulative   : {bold}{} tokens{reset}", format_num_commas(self.session_total_tokens), bold = theme.bold, reset = theme.reset);
        println!("  Preserved History    : {} conversational turns in rolling context", self.conversation_history.len());
        println!("{bold}{border}{reset}", bold = theme.bold, border = border, reset = theme.reset);
    }

    fn execute_user_prompt(&mut self, prompt: &str, sid: &str, turn: usize) {
        let theme = self.get_theme();
        let active_p = ProviderRegistry::active();

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

        match ProviderRegistry::dispatch_prompt_detailed(
            prompt,
            Some(&system_prompt),
            0.7,
            &self.conversation_history,
        ) {
            Ok(result) => {
                // Update session token telemetry
                self.session_input_tokens += result.input_tokens;
                self.session_output_tokens += result.output_tokens;
                self.session_total_tokens += result.total_tokens;

                // Push to rolling conversation history (keep last 20 turns)
                self.conversation_history.push((prompt.to_string(), result.text.clone()));
                if self.conversation_history.len() > 20 {
                    self.conversation_history.remove(0);
                }

                // Calculate context utilization
                let ctx_pct = if result.context_limit > 0 {
                    (result.input_tokens as f64 / result.context_limit as f64 * 100.0).min(100.0)
                } else {
                    0.0
                };

                let cols = terminal_cols().clamp(72, 160);
                let thin_border = "─".repeat(cols);

                // Print AI Completion Output
                println!("\n{}{}{}", theme.bold, result.text, theme.reset);

                // Print Token and Telemetry Footer
                println!("\n{dim}{thin_border}{reset}", dim = theme.dim, thin_border = thin_border, reset = theme.reset);
                println!(
                    "  {dim}[{provider} | model={model} | latency={ms}ms | session={sid}]{reset}",
                    dim = theme.dim,
                    provider = result.provider,
                    model = result.model,
                    ms = result.duration_ms,
                    sid = clean_path(sid, 16),
                    reset = theme.reset
                );
                println!(
                    "  📊 {bold}Turn Tokens:{reset} in: {accent}{}{reset} | out: {accent}{}{reset} | turn total: {bold}{}{reset}",
                    format_num_commas(result.input_tokens),
                    format_num_commas(result.output_tokens),
                    format_num_commas(result.total_tokens),
                    bold = theme.bold,
                    accent = theme.accent,
                    reset = theme.reset
                );
                println!(
                    "  🧠 {bold}Context Window:{reset} {} / {} tokens ({accent}{:.2}%{reset} utilized | model limit: {})",
                    format_num_commas(result.input_tokens),
                    format_num_commas(result.context_limit),
                    ctx_pct,
                    format_tokens_short(result.context_limit),
                    bold = theme.bold,
                    accent = theme.accent,
                    reset = theme.reset
                );
                println!(
                    "  📈 {bold}Session Total:{reset} in: {} | out: {} | cumulative: {bold}{} tokens{reset}",
                    format_num_commas(self.session_input_tokens),
                    format_num_commas(self.session_output_tokens),
                    format_num_commas(self.session_total_tokens),
                    bold = theme.bold,
                    reset = theme.reset
                );
                println!("{dim}{thin_border}{reset}", dim = theme.dim, thin_border = thin_border, reset = theme.reset);
            }
            Err(e) => {
                eprintln!("\n{}❌ Execution Error: {}{}", theme.warning, e, theme.reset);
                println!("\n{}Guided Resolution Options:{}", theme.bold, theme.reset);
                println!("  • To authenticate Gemini : {accent}/auth login gemini <api_key>{reset}  (or {accent}/auth login google{reset})", accent = theme.accent, reset = theme.reset);
                println!("  • To get free API key    : https://aistudio.google.com/app/apikey");
                println!("  • To switch AI provider  : {accent}/provider{reset} (21 supported models)", accent = theme.accent, reset = theme.reset);
                println!("  • To use zero-auth local : {accent}/provider ollama{reset} or {accent}/provider lmstudio{reset}", accent = theme.accent, reset = theme.reset);
                println!("  • For system diagnostics : {accent}/doctor{reset}", accent = theme.accent, reset = theme.reset);
            }
        }
    }
}

pub fn terminal_cols() -> usize {
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        unsafe {
            let mut winsize = MaybeUninit::<libc::winsize>::uninit();
            if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, winsize.as_mut_ptr()) == 0 {
                let ws = winsize.assume_init();
                if ws.ws_col >= 40 {
                    return ws.ws_col as usize;
                }
            }
        }
    }
    if let Ok(cols_str) = std::env::var("COLUMNS") {
        if let Ok(c) = cols_str.trim().parse::<usize>() {
            if c >= 40 {
                return c;
            }
        }
    }
    100
}

pub fn format_tokens_short(tokens: usize) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{}k", tokens / 1000)
    } else {
        format!("{}", tokens)
    }
}

pub fn format_num_commas(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    for (i, &ch) in chars.iter().enumerate() {
        result.push(ch);
        let rem = len - 1 - i;
        if rem > 0 && rem % 3 == 0 {
            result.push(',');
        }
    }
    result
}

fn clean_path(path: &str, max_len: usize) -> String {
    if path.chars().count() > max_len {
        let tail: String = path.chars().rev().take(max_len.saturating_sub(3)).collect();
        format!("...{}", tail.chars().rev().collect::<String>())
    } else {
        path.to_string()
    }
}
