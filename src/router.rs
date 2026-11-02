//! Semantic Skill & Capability Router with Multi-Tier Waterfall Fallbacks
//! Inspired by openmule/mulerouter-skills and CodebuffAI/freebuff

#[derive(Debug, Clone)]
pub struct SkillRoute {
    pub intent: &'static str,
    pub primary_agent: &'static str,
    pub fallback_agent: &'static str,
    pub model_policy: &'static str,
}

pub struct SkillRouter {
    routes: Vec<SkillRoute>,
}

impl SkillRouter {
    pub fn new() -> Self {
        Self {
            routes: vec![
                SkillRoute {
                    intent: "quant",
                    primary_agent: "tradingview-quant",
                    fallback_agent: "crypto-indicator-engineer",
                    model_policy: "gemini-3.8-flash",
                },
                SkillRoute {
                    intent: "audit",
                    primary_agent: "security-audit",
                    fallback_agent: "blue-team",
                    model_policy: "claude-3-7-sonnet",
                },
                SkillRoute {
                    intent: "rust",
                    primary_agent: "coding-engineer",
                    fallback_agent: "backend-architect",
                    model_policy: "deepseek-coder",
                },
                SkillRoute {
                    intent: "sre",
                    primary_agent: "opensre-triage",
                    fallback_agent: "doctor",
                    model_policy: "claude-3-7-sonnet",
                },
            ],
        }
    }

    pub fn resolve(&self, query: &str) -> &SkillRoute {
        let q_lower = query.to_lowercase();
        for route in &self.routes {
            if q_lower.contains(route.intent) {
                return route;
            }
        }
        &self.routes[2] // Default to coding-engineer
    }
}
