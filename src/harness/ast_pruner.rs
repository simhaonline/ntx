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
    /// Compute exact closing line for brace-delimited blocks (Rust, Go, TS/JS, C++)
    fn find_brace_block_end(lines: &[&str], start_idx: usize) -> usize {
        let mut depth = 0i32;
        let mut seen_open = false;
        for (offset, &line) in lines[start_idx..].iter().enumerate() {
            for ch in line.chars() {
                if ch == '{' {
                    depth += 1;
                    seen_open = true;
                } else if ch == '}' {
                    depth -= 1;
                }
            }
            if seen_open && depth <= 0 {
                return start_idx + offset + 1;
            }
            if !seen_open && line.trim_end().ends_with(';') {
                return start_idx + offset + 1;
            }
        }
        (start_idx + 35).min(lines.len())
    }

    /// Compute exact closing line for indentation-delimited blocks (Python)
    fn find_indent_block_end(lines: &[&str], start_idx: usize) -> usize {
        let base_indent = lines[start_idx].chars().take_while(|c| c.is_whitespace()).count();
        let mut last_valid = start_idx + 1;
        for (offset, &line) in lines[start_idx + 1..].iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            if indent <= base_indent {
                break;
            }
            last_valid = start_idx + 1 + offset + 1;
        }
        last_valid.min(lines.len())
    }

    /// Scan a source file and extract exact code symbols with true block boundaries (Rust, Python, Go, TS/JS)
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
            if line_trimmed.starts_with("//") || line_trimmed.starts_with('#') {
                continue;
            }

            // 1. Rust / TS / Go functions
            if line_trimmed.starts_with("pub fn ")
                || line_trimmed.starts_with("pub async fn ")
                || line_trimmed.starts_with("async fn ")
                || line_trimmed.starts_with("fn ")
                || line_trimmed.starts_with("func ")
                || line_trimmed.starts_with("export function ")
                || line_trimmed.starts_with("export async function ")
                || line_trimmed.starts_with("function ")
            {
                let after_kw = if let Some(idx) = line_trimmed.find("fn ") {
                    &line_trimmed[idx + 3..]
                } else if let Some(idx) = line_trimmed.find("func ") {
                    &line_trimmed[idx + 5..]
                } else if let Some(idx) = line_trimmed.find("function ") {
                    &line_trimmed[idx + 9..]
                } else {
                    ""
                };
                let name = after_kw
                    .trim()
                    .split(|c: char| c == '(' || c == '<' || c.is_whitespace())
                    .find(|s| !s.is_empty())
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    let end_line = Self::find_brace_block_end(&lines, i);
                    symbols.push(CodeSymbol {
                        name,
                        symbol_type: "fn".into(),
                        file_path: file_path.clone(),
                        start_line: i + 1,
                        end_line,
                        signature: line_trimmed.to_string(),
                    });
                }
            }
            // 2. Rust / Go / TS structs, enums, traits, impls, classes, interfaces
            else if line_trimmed.starts_with("pub struct ")
                || line_trimmed.starts_with("struct ")
                || line_trimmed.starts_with("pub enum ")
                || line_trimmed.starts_with("enum ")
                || line_trimmed.starts_with("pub trait ")
                || line_trimmed.starts_with("trait ")
                || line_trimmed.starts_with("impl ")
                || line_trimmed.starts_with("impl<")
                || line_trimmed.starts_with("export class ")
                || line_trimmed.starts_with("export interface ")
                || line_trimmed.starts_with("class ")
                || line_trimmed.starts_with("interface ")
            {
                let (sym_kind, name_part) = if line_trimmed.contains("struct ") {
                    ("struct", line_trimmed.split("struct ").nth(1).unwrap_or(""))
                } else if line_trimmed.contains("enum ") {
                    ("enum", line_trimmed.split("enum ").nth(1).unwrap_or(""))
                } else if line_trimmed.contains("trait ") {
                    ("trait", line_trimmed.split("trait ").nth(1).unwrap_or(""))
                } else if line_trimmed.starts_with("impl") {
                    ("impl", line_trimmed.trim_start_matches("impl"))
                } else if line_trimmed.contains("interface ") {
                    ("interface", line_trimmed.split("interface ").nth(1).unwrap_or(""))
                } else {
                    ("class", line_trimmed.split("class ").nth(1).unwrap_or(""))
                };
                let name = name_part
                    .trim()
                    .split(|c: char| c == '{' || c == '(' || c == '<' || c == ':' || c.is_whitespace())
                    .find(|s| !s.is_empty())
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() {
                    let end_line = if file_path.ends_with(".py") {
                        Self::find_indent_block_end(&lines, i)
                    } else {
                        Self::find_brace_block_end(&lines, i)
                    };
                    symbols.push(CodeSymbol {
                        name,
                        symbol_type: sym_kind.into(),
                        file_path: file_path.clone(),
                        start_line: i + 1,
                        end_line,
                        signature: line_trimmed.to_string(),
                    });
                }
            }
            // 3. Python def / async def
            else if line_trimmed.starts_with("def ") || line_trimmed.starts_with("async def ") {
                let after_def = line_trimmed.split("def ").nth(1).unwrap_or("");
                let name = after_def
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !name.is_empty() {
                    let end_line = Self::find_indent_block_end(&lines, i);
                    symbols.push(CodeSymbol {
                        name,
                        symbol_type: "def".into(),
                        file_path: file_path.clone(),
                        start_line: i + 1,
                        end_line,
                        signature: line_trimmed.to_string(),
                    });
                }
            }
        }

        symbols
    }

    /// Recursively collect source files in a workspace directory
    fn collect_source_files(dir: &Path, out: &mut Vec<std::path::PathBuf>, depth: usize) {
        if depth > 4 {
            return;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" || name == "node_modules" || name == "dist" {
                    continue;
                }
                if p.is_dir() {
                    Self::collect_source_files(&p, out, depth + 1);
                } else if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                        if matches!(ext, "rs" | "py" | "go" | "ts" | "tsx" | "js" | "sh") {
                            out.push(p);
                        }
                    }
                }
            }
        }
    }

    /// Prune context with Call-Graph Dependency Resolution: extracts matching symbols AND referenced callee symbols
    pub fn prune_context_for_intent(repo_dir: &Path, intent: &str) -> Vec<PrunedContextSnippet> {
        let mut snippets = Vec::new();
        let intent_lower = intent.to_lowercase();
        let keywords: Vec<&str> = intent_lower
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|k| k.len() >= 2)
            .collect();

        let mut source_files = Vec::new();
        Self::collect_source_files(repo_dir, &mut source_files, 0);

        let mut all_symbols = Vec::new();
        for file in &source_files {
            all_symbols.extend(Self::parse_symbols(file));
        }

        let mut matched_bodies = Vec::new();
        let mut seen_keys = std::collections::HashSet::new();

        for s in &all_symbols {
            let matches_kw = keywords.is_empty()
                || keywords
                    .iter()
                    .any(|&k| s.name.to_lowercase().contains(k) || s.signature.to_lowercase().contains(k));
            if matches_kw {
                if let Ok(content) = fs::read_to_string(&s.file_path) {
                    let lines: Vec<&str> = content.lines().collect();
                    let start = s.start_line.saturating_sub(1);
                    let end = s.end_line.min(lines.len());
                    if start < end {
                        let slice = lines[start..end].join("\n");
                        let key = format!("{}:{}", s.file_path, s.start_line);
                        if seen_keys.insert(key) {
                            matched_bodies.push(slice.clone());
                            snippets.push(PrunedContextSnippet {
                                file_path: s.file_path.clone(),
                                start_line: s.start_line,
                                end_line: s.end_line,
                                content: slice,
                                matched_symbol: s.name.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Call-Graph 1-Hop Dependency Expansion: include symbols called inside matched bodies
        for s in &all_symbols {
            if s.name.len() < 3 {
                continue;
            }
            let key = format!("{}:{}", s.file_path, s.start_line);
            if seen_keys.contains(&key) {
                continue;
            }
            let is_referenced = matched_bodies.iter().any(|body| body.contains(&s.name));
            if is_referenced {
                if let Ok(content) = fs::read_to_string(&s.file_path) {
                    let lines: Vec<&str> = content.lines().collect();
                    let start = s.start_line.saturating_sub(1);
                    let end = s.end_line.min(lines.len());
                    if start < end && seen_keys.insert(key) {
                        let slice = lines[start..end].join("\n");
                        snippets.push(PrunedContextSnippet {
                            file_path: s.file_path.clone(),
                            start_line: s.start_line,
                            end_line: s.end_line,
                            content: slice,
                            matched_symbol: format!("{} (callee-dep)", s.name),
                        });
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
