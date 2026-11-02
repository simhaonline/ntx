//! Bradley-Terry Statistical ELO Ranking Engine
//! Inspired by lmarena/arena-rank and simhaonline/llm-judge-app
//! Implements deterministic pairwise ELO updates and model rankings from scratch.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModelRating {
    pub name: String,
    pub elo: f64,
    pub matches: u32,
    pub wins: u32,
    pub losses: u32,
    pub ties: u32,
}

pub struct ArenaRanker {
    pub k_factor: f64,
    pub models: HashMap<String, ModelRating>,
}

impl ArenaRanker {
    pub fn new(k_factor: f64) -> Self {
        Self {
            k_factor,
            models: HashMap::new(),
        }
    }

    pub fn get_or_register(&mut self, name: &str) -> &mut ModelRating {
        self.models.entry(name.to_string()).or_insert_with(|| ModelRating {
            name: name.to_string(),
            elo: 1000.0,
            matches: 0,
            wins: 0,
            losses: 0,
            ties: 0,
        })
    }

    /// Record a match outcome between model_a and model_b:
    /// score_a: 1.0 (win), 0.5 (tie), 0.0 (loss)
    pub fn record_battle(&mut self, model_a: &str, model_b: &str, score_a: f64) {
        let score_b = 1.0 - score_a;

        self.get_or_register(model_a);
        self.get_or_register(model_b);
        let elo_a = self.models.get(model_a).unwrap().elo;
        let elo_b = self.models.get(model_b).unwrap().elo;

        // Bradley-Terry expected score logistic function
        let expected_a = 1.0 / (1.0 + 10.0_f64.powf((elo_b - elo_a) / 400.0));
        let expected_b = 1.0 - expected_a;

        let delta_a = self.k_factor * (score_a - expected_a);
        let delta_b = self.k_factor * (score_b - expected_b);

        {
            let a = self.models.get_mut(model_a).unwrap();
            a.elo += delta_a;
            a.matches += 1;
            if score_a == 1.0 { a.wins += 1; }
            else if score_a == 0.0 { a.losses += 1; }
            else { a.ties += 1; }
        }

        {
            let b = self.models.get_mut(model_b).unwrap();
            b.elo += delta_b;
            b.matches += 1;
            if score_b == 1.0 { b.wins += 1; }
            else if score_b == 0.0 { b.losses += 1; }
            else { b.ties += 1; }
        }
    }

    pub fn leaderboard(&self) -> Vec<&ModelRating> {
        let mut list: Vec<&ModelRating> = self.models.values().collect();
        list.sort_by(|a, b| b.elo.partial_cmp(&a.elo).unwrap_or(std::cmp::Ordering::Equal));
        list
    }
}
