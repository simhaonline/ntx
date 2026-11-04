//! Nityaksha Native Scraper & Capability Synthesizer
//! Scrapes technical documentation and dynamically synthesizes original, clean, proprietary Nityaksha skills and runbooks.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedKnowledge {
    pub source_url: String,
    pub title: String,
    pub extracted_text: String,
    pub tool_signatures: Vec<String>,
    pub execution_rules: Vec<String>,
}

pub struct ScraperEngine {
    output_catalog_dir: PathBuf,
    output_agents_dir: PathBuf,
}

impl ScraperEngine {
    pub fn new(catalog_dir: impl AsRef<Path>, agents_dir: impl AsRef<Path>) -> Self {
        Self {
            output_catalog_dir: catalog_dir.as_ref().to_path_buf(),
            output_agents_dir: agents_dir.as_ref().to_path_buf(),
        }
    }

    /// Extract text and headings from markdown or HTML content
    pub fn parse_documentation(&self, raw_content: &str, source_url: &str) -> ScrapedKnowledge {
        let mut title = String::new();
        let mut text_lines = Vec::new();
        let mut tools = Vec::new();
        let mut rules = Vec::new();

        for line in raw_content.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.starts_with("# ") && title.is_empty() {
                title = line_trimmed[2..].trim().to_string();
            } else if line_trimmed.starts_with("```") || line_trimmed.starts_with("$ ") {
                tools.push(line_trimmed.to_string());
            } else if line_trimmed.starts_with("- ") || line_trimmed.starts_with("* ") {
                rules.push(line_trimmed[2..].trim().to_string());
            } else if !line_trimmed.is_empty() {
                text_lines.push(line_trimmed);
            }
        }

        if title.is_empty() {
            title = source_url.split('/').last().unwrap_or("Synthesized Knowledge").to_string();
        }

        ScrapedKnowledge {
            source_url: source_url.to_string(),
            title,
            extracted_text: text_lines.join(" "),
            tool_signatures: tools,
            execution_rules: rules,
        }
    }

    /// Synthesize an original, proprietary first-party Nityaksha skill from scraped technical knowledge
    pub fn synthesize_proprietary_skill(
        &self,
        unique_name: &str,
        title: &str,
        knowledge: &ScrapedKnowledge,
    ) -> io::Result<PathBuf> {
        let skill_name = if unique_name.starts_with("ntx-") {
            unique_name.to_string()
        } else {
            format!("ntx-{}", unique_name)
        };

        let skill_dir = self.output_agents_dir.join("skills").join(&skill_name);
        fs::create_dir_all(&skill_dir)?;

        let cat_skills_dir = self.output_catalog_dir.join("skills");
        fs::create_dir_all(&cat_skills_dir)?;

        let desc = if knowledge.extracted_text.len() > 160 {
            format!("{}...", &knowledge.extracted_text[..160])
        } else {
            knowledge.extracted_text.clone()
        };

        // 1. Write clean proprietary catalog manifest
        let manifest_content = format!(
            r#"apiVersion: nityaksha.simha/v1
kind: Skill
metadata:
  namespace: default
  name: {}
  author: "Simha (simhaonline)"
  proprietary: true
  version: "1.0.0"
  description: {:?}
spec:
  source:
    provider: internal
    repository: nityaksha/core
    path: skills/{}/SKILL.md
  mode: instructions-only
  approval: approved
"#,
            skill_name, desc, skill_name
        );
        fs::write(cat_skills_dir.join(format!("{}.yaml", skill_name)), manifest_content)?;

        // 2. Write self-contained SKILL.md runbook in files and folder
        let mut skill_md = format!(
            r#"---
name: {}
title: {}
author: Simha (simhaonline)
version: 1.0.0
proprietary: true
provider: internal
repository: nityaksha/core
description: {}
---

# {}

## 1. Problem Decomposition & Invariants
- **Core Objective**: Autonomous execution of {}
- **Execution Rules**:
"#,
            skill_name, title, desc, title, title
        );

        for (i, rule) in knowledge.execution_rules.iter().take(5).enumerate() {
            skill_md.push_str(&format!("  {}. {}\n", i + 1, rule));
        }

        skill_md.push_str("\n## 2. Autonomous Operational Protocol\n```bash\n");
        for tool in knowledge.tool_signatures.iter().take(4) {
            skill_md.push_str(&format!("{}\n", tool));
        }
        skill_md.push_str("```\n");

        let skill_md_path = skill_dir.join("SKILL.md");
        fs::write(&skill_md_path, skill_md)?;

        Ok(skill_md_path)
    }
}
