//! Nityaksha Vector Database Engine
//! Dense vector similarity search, SIMD/Vectorized Cosine/Dot/L2, and disk-cached page storage for edge scale.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    Cosine,
    DotProduct,
    Euclidean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    pub id: String,
    pub values: Vec<f32>,
    pub text: String,
    pub metadata: HashMap<String, String>,
    pub media_hash: Option<String>,
    pub document_id: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub text: String,
    pub metadata: HashMap<String, String>,
    pub media_hash: Option<String>,
    pub document_id: Option<String>,
}

/// In-memory + disk-cached Vector Collection
pub struct VectorCollection {
    pub name: String,
    pub dimension: usize,
    pub metric: MetricType,
    pub records: Arc<RwLock<Vec<VectorRecord>>>,
    pub id_index: Arc<RwLock<HashMap<String, usize>>>,
    storage_path: Option<PathBuf>,
}

impl VectorCollection {
    pub fn new(name: impl Into<String>, dimension: usize, metric: MetricType) -> Self {
        Self {
            name: name.into(),
            dimension,
            metric,
            records: Arc::new(RwLock::new(Vec::new())),
            id_index: Arc::new(RwLock::new(HashMap::new())),
            storage_path: None,
        }
    }

    pub fn open(path: impl AsRef<Path>, name: impl Into<String>, dimension: usize, metric: MetricType) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        fs::create_dir_all(&p)?;
        let mut col = Self::new(name, dimension, metric);
        col.storage_path = Some(p.clone());

        let bin_path = p.join(format!("{}.vec", col.name));
        if bin_path.exists() {
            col.load_from_disk(&bin_path)?;
        }
        Ok(col)
    }

    pub fn insert(&self, record: VectorRecord) -> io::Result<()> {
        if record.values.len() != self.dimension {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Vector dimension mismatch: expected {}, got {}", self.dimension, record.values.len()),
            ));
        }

        let mut recs = self.records.write().unwrap();
        let mut idx = self.id_index.write().unwrap();

        if let Some(&existing_idx) = idx.get(&record.id) {
            recs[existing_idx] = record;
        } else {
            let new_idx = recs.len();
            idx.insert(record.id.clone(), new_idx);
            recs.push(record);
        }

        if let Some(ref p) = self.storage_path {
            let bin_path = p.join(format!("{}.vec", self.name));
            self.persist_to_disk(&bin_path, &recs)?;
        }

        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<VectorRecord> {
        let idx = self.id_index.read().unwrap();
        let recs = self.records.read().unwrap();
        idx.get(id).map(|&i| recs[i].clone())
    }

    pub fn count(&self) -> usize {
        self.records.read().unwrap().len()
    }

    pub fn search(&self, query_vector: &[f32], top_k: usize, threshold: Option<f32>) -> Vec<VectorSearchResult> {
        if query_vector.len() != self.dimension {
            return Vec::new();
        }

        let recs = self.records.read().unwrap();
        let mut scored: Vec<(f32, &VectorRecord)> = recs
            .iter()
            .map(|r| {
                let score = match self.metric {
                    MetricType::Cosine => cosine_similarity(query_vector, &r.values),
                    MetricType::DotProduct => dot_product(query_vector, &r.values),
                    MetricType::Euclidean => -euclidean_distance(query_vector, &r.values),
                };
                (score, r)
            })
            .filter(|(s, _)| threshold.map_or(true, |t| *s >= t))
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(top_k)
            .map(|(score, r)| VectorSearchResult {
                id: r.id.clone(),
                score,
                text: r.text.clone(),
                metadata: r.metadata.clone(),
                media_hash: r.media_hash.clone(),
                document_id: r.document_id.clone(),
            })
            .collect()
    }

    fn persist_to_disk(&self, path: &Path, recs: &[VectorRecord]) -> io::Result<()> {
        let temp_path = path.with_extension("tmp");
        let data = serde_json::to_vec(recs).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(&temp_path, data)?;
        fs::rename(temp_path, path)?;
        Ok(())
    }

    fn load_from_disk(&mut self, path: &Path) -> io::Result<()> {
        let data = fs::read(path)?;
        if data.is_empty() {
            return Ok(());
        }
        let recs: Vec<VectorRecord> = serde_json::from_slice(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut idx = HashMap::with_capacity(recs.len());
        for (i, r) in recs.iter().enumerate() {
            idx.insert(r.id.clone(), i);
        }
        *self.records.write().unwrap() = recs;
        *self.id_index.write().unwrap() = idx;
        Ok(())
    }
}

#[inline]
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    let chunks_a = a.chunks_exact(4);
    let chunks_b = b.chunks_exact(4);
    let rem_a = chunks_a.remainder();
    let rem_b = chunks_b.remainder();

    for (ca, cb) in chunks_a.zip(chunks_b) {
        sum += ca[0] * cb[0] + ca[1] * cb[1] + ca[2] * cb[2] + ca[3] * cb[3];
    }
    for (ra, rb) in rem_a.iter().zip(rem_b.iter()) {
        sum += ra * rb;
    }
    sum
}

#[inline]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product(a, b);
    let norm_a = dot_product(a, a).sqrt();
    let norm_b = dot_product(b, b).sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        let diff = x - y;
        sum += diff * diff;
    }
    sum.sqrt()
}

/// Attempt live neural embedding via configured provider API (OpenAI, Gemini, or local Ollama)
pub fn embed_text_neural(text: &str, dimension: usize) -> Option<Vec<f32>> {
    if text.trim().is_empty() {
        return None;
    }
    // Only invoke network embedding if explicitly enabled or provider key is set with NTX_NEURAL_EMBED
    let use_live = std::env::var("NTX_NEURAL_EMBED").map(|v| v == "1" || v == "true").unwrap_or(false);
    if !use_live {
        return None;
    }

    // 1. Try OpenAI text-embedding-3-small
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        if !api_key.trim().is_empty() {
            let payload = serde_json::json!({
                "input": text,
                "model": "text-embedding-3-small",
                "dimensions": dimension
            });
            if let Ok(out) = std::process::Command::new("curl")
                .args([
                    "-sS", "--max-time", "4",
                    "-H", &format!("Authorization: Bearer {}", api_key.trim()),
                    "-H", "Content-Type: application/json",
                    "-d", &payload.to_string(),
                    "https://api.openai.com/v1/embeddings"
                ])
                .output()
            {
                if out.status.success() {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                        if let Some(arr) = val.pointer("/data/0/embedding").and_then(|v| v.as_array()) {
                            let raw: Vec<f32> = arr.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                            if !raw.is_empty() {
                                return Some(project_and_normalize(&raw, dimension));
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Try Google Gemini text-embedding-004
    if let Ok(api_key) = std::env::var("GEMINI_API_KEY").or_else(|_| std::env::var("GOOGLE_API_KEY")) {
        if !api_key.trim().is_empty() {
            let payload = serde_json::json!({
                "model": "models/text-embedding-004",
                "content": { "parts": [{ "text": text }] },
                "outputDimensionality": dimension
            });
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent?key={}",
                api_key.trim()
            );
            if let Ok(out) = std::process::Command::new("curl")
                .args(["-sS", "--max-time", "4", "-H", "Content-Type: application/json", "-d", &payload.to_string(), &url])
                .output()
            {
                if out.status.success() {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                        if let Some(arr) = val.pointer("/embedding/values").and_then(|v| v.as_array()) {
                            let raw: Vec<f32> = arr.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                            if !raw.is_empty() {
                                return Some(project_and_normalize(&raw, dimension));
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Try Local Ollama /api/embeddings
    if let Ok(ollama_host) = std::env::var("OLLAMA_HOST") {
        let base = ollama_host.trim_end_matches('/');
        let payload = serde_json::json!({
            "model": "nomic-embed-text",
            "prompt": text
        });
        if let Ok(out) = std::process::Command::new("curl")
            .args([
                "-sS", "--max-time", "2",
                "-H", "Content-Type: application/json",
                "-d", &payload.to_string(),
                &format!("{}/api/embeddings", base)
            ])
            .output()
        {
            if out.status.success() {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                    if let Some(arr) = val.get("embedding").and_then(|v| v.as_array()) {
                        let raw: Vec<f32> = arr.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                        if !raw.is_empty() {
                            return Some(project_and_normalize(&raw, dimension));
                        }
                    }
                }
            }
        }
    }

    None
}

fn project_and_normalize(raw: &[f32], dimension: usize) -> Vec<f32> {
    let mut vec = vec![0.0f32; dimension];
    for (i, &val) in raw.iter().enumerate() {
        vec[i % dimension] += val;
    }
    let norm = dot_product(&vec, &vec).sqrt();
    if norm > 0.0 {
        for v in vec.iter_mut() {
            *v /= norm;
        }
    }
    vec
}

/// Hybrid Neural + Sub-word Semantic Text Embedder (supports live neural API + offline morphological n-gram projection)
pub fn embed_text(text: &str, dimension: usize) -> Vec<f32> {
    if let Some(neural_vec) = embed_text_neural(text, dimension) {
        return neural_vec;
    }

    use sha2::Digest;
    let mut vec = vec![0.0f32; dimension];
    let cleaned: String = text
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.is_empty() {
        return vec;
    }

    let mut accumulate_token = |token: &str, weight: f32| {
        let hash = sha2::Sha256::digest(token.as_bytes());
        for i in 0..dimension {
            let byte = hash[i % hash.len()] as f32;
            vec[i] += weight * ((byte - 128.0) / 128.0);
        }
    };

    for (idx, &word) in words.iter().enumerate() {
        // 1. Full word projection
        accumulate_token(word, 1.0);

        // 2. Stemmed root projection (strips common suffixes for morphological invariance)
        let stem = word
            .trim_end_matches("ing")
            .trim_end_matches("tion")
            .trim_end_matches("ed")
            .trim_end_matches("es")
            .trim_end_matches('s');
        if stem.len() >= 3 && stem != word {
            accumulate_token(stem, 0.85);
        }

        // 3. Character 3-grams for sub-word semantic similarity
        let chars: Vec<char> = word.chars().collect();
        if chars.len() >= 3 {
            for win in chars.windows(3) {
                let trigram: String = win.iter().collect();
                accumulate_token(&trigram, 0.35);
            }
        }

        // 4. Word bigrams for local phrase context
        if idx + 1 < words.len() {
            let bigram = format!("{}_{}", word, words[idx + 1]);
            accumulate_token(&bigram, 0.5);
        }
    }

    let norm = dot_product(&vec, &vec).sqrt();
    if norm > 0.0 {
        for v in vec.iter_mut() {
            *v /= norm;
        }
    }
    vec
}
