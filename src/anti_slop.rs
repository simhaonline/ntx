//! Anti-Slop High-Throughput Stream Filter
//! Eliminates conversational preamble, moralizing disclaimers, hedging, and meta-announcements
//! while strictly preserving code fences, JSON blocks, and technical tables.

pub struct AntiSlopFilter;

impl AntiSlopFilter {
    const SLOP_PREFIXES: &'static [&'static str] = &[
        "certainly",
        "sure thing",
        "of course",
        "i would be happy to",
        "i'd be happy to",
        "as an ai",
        "as an ai language model",
        "here is the",
        "below is the",
        "i hope this helps",
        "let me know if you need",
        "it is important to remember",
        "please keep in mind",
    ];

    pub fn filter(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut in_code_block = false;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                out.push_str(line);
                out.push('\n');
                continue;
            }

            if in_code_block {
                out.push_str(line);
                out.push('\n');
                continue;
            }

            let lower = trimmed.to_lowercase();
            let is_slop = Self::SLOP_PREFIXES.iter().any(|&prefix| lower.starts_with(prefix));

            if !is_slop && !trimmed.is_empty() {
                out.push_str(line);
                out.push('\n');
            }
        }

        out.trim().to_string()
    }
}
