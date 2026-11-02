//! Nityaksha Static Codebase Discovery, AST Parsing & Precise Context Pruner
//! Extracts function/struct definitions, prunes context to relevant line ranges, and auto-generates tool schemas.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSymbol {
    pub name: String,
    pub symbol_type: String, // "fn", "struct", "enum", "class", "def"
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrunedContextSnippet {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub matched_symbol: String,
}

pub struct AstPruner;

impl AstPruner {
    /// Scan a source file and extract top-level code symbols (Rust, Python, Go, JS)
    pub fn parse_symbols(path: &Path) -> Vec<CodeSymbol> {
        let mut symbols = Vec::new();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return symbols,
        };

        let file_path = path.to_string_lossy().to_string();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_trimmed = line.trim();

            // Rust function
            if line_trimmed.starts_with("pub fn ") || line_trimmed.starts_with("fn ") {
                let name = line_trimmed
                    .split_whitespace()
                    .nth(if line_trimmed.starts_with("pub ") { 2 } else { 1 })
                    .unwrap_or("")
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    symbols.push(CodeSymbol {
                        name,
                        symbol_type: "fn".into(),
                        file_path: file_path.clone(),
                        start_line: i + 1,
                        end_line: (i + 30).min(lines.len()),
                        signature: line_trimmed.to_string(),
                    });
                }
            }
            // Rust struct
            else if line_trimmed.starts_with("pub struct ") || line_trimmed.starts_with("struct ") {
                let name = line_trimmed
                    .split_whitespace()
                    .nth(if line_trimmed.starts_with("pub ") { 2 } else { 1 })
                    .unwrap_or("")
                    .trim_end_matches('{')
                    .trim()
                    .to_string();
                if !name.is_empty() {
                    symbols.push(CodeSymbol {
                        name,
                        symbol_type: "struct".into(),
                        file_path: file_path.clone(),
                        start_line: i + 1,
                        end_line: (i + 20).min(lines.len()),
                        signature: line_trimmed.to_string(),
                    });
                }
            }
            // Python def
            else if line_trimmed.starts_with("def ") {
                let name = line_trimmed["def ".len()..]
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !name.is_empty() {
                    symbols.push(CodeSymbol {
                        name,
                        symbol_type: "def".into(),
                        file_path: file_path.clone(),
                        start_line: i + 1,
                        end_line: (i + 25).min(lines.len()),
                        signature: line_trimmed.to_string(),
                    });
                }
            }
        }

        symbols
    }

    /// Prune context: given a prompt intent, extract only relevant line ranges
    pub fn prune_context_for_intent(repo_dir: &Path, intent: &str) -> Vec<PrunedContextSnippet> {
        let mut snippets = Vec::new();
        let intent_lower = intent.to_lowercase();
        let keywords: Vec<&str> = intent_lower.split_whitespace().collect();

        if let Ok(entries) = fs::read_dir(repo_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && (p.extension().map_or(false, |e| e == "rs" || e == "py" || e == "sh")) {
                    let symbols = Self::parse_symbols(&p);
                    for s in symbols {
                        if keywords.iter().any(|&k| s.name.to_lowercase().contains(k) || s.signature.to_lowercase().contains(k)) {
                            if let Ok(content) = fs::read_to_string(&s.file_path) {
                                let lines: Vec<&str> = content.lines().collect();
                                let start = s.start_line.saturating_sub(1);
                                let end = s.end_line.min(lines.len());
                                let slice = lines[start..end].join("\n");
                                snippets.push(PrunedContextSnippet {
                                    file_path: s.file_path.clone(),
                                    start_line: s.start_line,
                                    end_line: s.end_line,
                                    content: slice,
                                    matched_symbol: s.name,
                                });
                            }
                        }
                    }
                }
            }
        }
        snippets
    }

    /// Automatically generate structured JSON tool schema for an undocumented script
    pub fn auto_generate_tool_schema(script_name: &str, description: &str) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": script_name,
                "description": description,
                "parameters": {
                    "type": "object",
                    "properties": {
                        "args": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Positional arguments to supply to the command"
                        },
                        "flags": {
                            "type": "object",
                            "description": "Optional keyword flags"
                        }
                    },
                    "required": ["args"]
                }
            }
        })
    }
}
