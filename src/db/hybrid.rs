//! Nityaksha Hybrid Search & Fast RAG Retrieval Engine
//! BM25 Sparse Lexical Index + Dense Vector Semantic Reranker.

use crate::db::vector::{VectorCollection, VectorSearchResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalCitation {
    pub document_id: String,
    pub source_path: String,
    pub line_range: Option<String>,
    pub commit_sha: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub id: String,
    pub text: String,
    pub hybrid_score: f32,
    pub dense_score: f32,
    pub sparse_score: f32,
    pub metadata: HashMap<String, String>,
    pub media_hash: Option<String>,
    pub document_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExternalVectorBackend {
    ChromaDb { endpoint: String, collection: String },
    Qdrant { endpoint: String, collection: String },
    PgVector { connection_uri: String, table: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalVectorAdapter {
    pub backend: ExternalVectorBackend,
    pub tenant_id: String,
}

impl ExternalVectorAdapter {
    pub fn new(backend: ExternalVectorBackend, tenant_id: &str) -> Self {
        Self {
            backend,
            tenant_id: tenant_id.to_string(),
        }
    }

    /// Builds a tenant-isolated query payload or SQL statement for the configured external vector store (`WP-13`)
    pub fn build_query_payload(&self, query_vector: &[f32], top_k: usize) -> serde_json::Value {
        match &self.backend {
            ExternalVectorBackend::ChromaDb { endpoint, collection } => serde_json::json!({
                "adapter": "chromadb",
                "url": format!("{}/api/v1/collections/{}/query", endpoint.trim_end_matches('/'), collection),
                "body": {
                    "query_embeddings": [query_vector],
                    "n_results": top_k,
                    "where": { "tenant_id": self.tenant_id }
                }
            }),
            ExternalVectorBackend::Qdrant { endpoint, collection } => serde_json::json!({
                "adapter": "qdrant",
                "url": format!("{}/collections/{}/points/search", endpoint.trim_end_matches('/'), collection),
                "body": {
                    "vector": query_vector,
                    "limit": top_k,
                    "filter": {
                        "must": [{ "key": "tenant_id", "match": { "value": self.tenant_id } }]
                    },
                    "with_payload": true
                }
            }),
            ExternalVectorBackend::PgVector { connection_uri, table } => serde_json::json!({
                "adapter": "pgvector",
                "connection_uri_redacted": connection_uri.split('@').last().unwrap_or("localhost:5432"),
                "sql": format!(
                    "SELECT id, text, metadata, 1 - (embedding <=> $1::vector) AS score FROM {} WHERE metadata->>'tenant_id' = $2 ORDER BY embedding <=> $1::vector LIMIT {}",
                    table, top_k
                ),
                "params": [format!("{:?}", query_vector), self.tenant_id]
            }),
        }
    }
}

/// BM25 inverted lexical index for keyword grounding
pub struct Bm25Index {
    k1: f32,
    b: f32,
    doc_count: usize,
    doc_lengths: HashMap<String, usize>,
    avg_doc_len: f32,
    inverted_index: HashMap<String, HashMap<String, usize>>, // term -> (doc_id -> term_freq)
}

impl Bm25Index {
    pub fn new() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            doc_count: 0,
            doc_lengths: HashMap::new(),
            avg_doc_len: 0.0,
            inverted_index: HashMap::new(),
        }
    }

    pub fn index_document(&mut self, doc_id: &str, text: &str) {
        let tokens = tokenize(text);
        let len = tokens.len();
        if len == 0 {
            return;
        }

        let mut term_freqs: HashMap<String, usize> = HashMap::new();
        for t in tokens {
            *term_freqs.entry(t).or_insert(0) += 1;
        }

        for (term, freq) in term_freqs {
            self.inverted_index
                .entry(term)
                .or_insert_with(HashMap::new)
                .insert(doc_id.to_string(), freq);
        }

        self.doc_lengths.insert(doc_id.to_string(), len);
        self.doc_count += 1;
        let total_len: usize = self.doc_lengths.values().sum();
        self.avg_doc_len = total_len as f32 / self.doc_count as f32;
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<(String, f32)> {
        let query_terms = tokenize(query);
        if query_terms.is_empty() || self.doc_count == 0 {
            return Vec::new();
        }

        let mut scores: HashMap<String, f32> = HashMap::new();

        for term in &query_terms {
            if let Some(postings) = self.inverted_index.get(term) {
                let n = postings.len() as f32;
                // Standard Lucene/BM25 IDF
                let idf = ((self.doc_count as f32 - n + 0.5) / (n + 0.5) + 1.0).ln();

                for (doc_id, &tf) in postings {
                    let doc_len = *self.doc_lengths.get(doc_id).unwrap_or(&0) as f32;
                    let denom = tf as f32 + self.k1 * (1.0 - self.b + self.b * (doc_len / self.avg_doc_len.max(1.0)));
                    let term_score = idf * (tf as f32 * (self.k1 + 1.0)) / denom;
                    *scores.entry(doc_id.clone()).or_insert(0.0) += term_score;
                }
            }
        }

        let mut results: Vec<(String, f32)> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }
}

/// Hybrid RAG search engine combining Vector Collection + BM25 Lexical Index
pub struct HybridRagEngine {
    pub vector_col: Arc<VectorCollection>,
    pub bm25: Arc<RwLock<Bm25Index>>,
}

impl HybridRagEngine {
    pub fn new(vector_col: Arc<VectorCollection>) -> Self {
        let mut bm25 = Bm25Index::new();
        // Index existing vectors if any
        let recs = vector_col.records.read().unwrap();
        for r in recs.iter() {
            bm25.index_document(&r.id, &r.text);
        }
        drop(recs);

        Self {
            vector_col,
            bm25: Arc::new(RwLock::new(bm25)),
        }
    }

    pub fn index_record(&self, id: &str, text: &str) {
        self.bm25.write().unwrap().index_document(id, text);
    }

    /// Query with hybrid weight `alpha` (1.0 = pure vector, 0.0 = pure BM25 lexical)
    pub fn search(
        &self,
        query_text: &str,
        query_vector: Option<&[f32]>,
        top_k: usize,
        alpha: f32,
    ) -> Vec<HybridSearchResult> {
        let alpha = alpha.clamp(0.0, 1.0);

        // 1. Sparse BM25 Search
        let bm25_scores = self.bm25.read().unwrap().search(query_text, top_k * 2);
        let max_bm25 = bm25_scores.first().map(|s| s.1).unwrap_or(1.0).max(1.0);
        let bm25_map: HashMap<String, f32> = bm25_scores
            .into_iter()
            .map(|(id, score)| (id, score / max_bm25))
            .collect();

        // 2. Dense Vector Search
        let generated_vec = if query_vector.is_none() && !query_text.is_empty() {
            Some(crate::db::vector::embed_text(query_text, self.vector_col.dimension))
        } else {
            None
        };
        let effective_vec = query_vector.or_else(|| generated_vec.as_deref());

        let dense_results = if let Some(q_vec) = effective_vec {
            self.vector_col.search(q_vec, top_k * 2, None)
        } else {
            Vec::new()
        };

        let mut candidate_ids: HashSet<String> = HashSet::new();
        for (id, _) in &bm25_map {
            candidate_ids.insert(id.clone());
        }
        for d in &dense_results {
            candidate_ids.insert(d.id.clone());
        }

        let dense_map: HashMap<String, &VectorSearchResult> =
            dense_results.iter().map(|d| (d.id.clone(), d)).collect();

        // 3. Combined Fusion
        let mut results = Vec::new();
        for id in candidate_ids {
            let sparse_score = *bm25_map.get(&id).unwrap_or(&0.0);
            let (dense_score, text, metadata, media_hash, doc_id) = if let Some(d) = dense_map.get(&id) {
                (d.score, d.text.clone(), d.metadata.clone(), d.media_hash.clone(), d.document_id.clone())
            } else if let Some(rec) = self.vector_col.get(&id) {
                (0.0, rec.text, rec.metadata, rec.media_hash, rec.document_id)
            } else {
                continue;
            };

            let hybrid_score = alpha * dense_score.max(0.0) + (1.0 - alpha) * sparse_score;

            results.push(HybridSearchResult {
                id,
                text,
                hybrid_score,
                dense_score,
                sparse_score,
                metadata,
                media_hash,
                document_id: doc_id,
            });
        }

        results.sort_by(|a, b| b.hybrid_score.partial_cmp(&a.hybrid_score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    /// Pre-retrieval tenant-isolated hybrid search (`S-003` & `WP-13`).
    /// Filters candidate records strictly by `tenant_id` BEFORE scoring and ranking so cross-tenant data is never scored or returned.
    pub fn search_for_tenant(
        &self,
        query_text: &str,
        caller_tenant_id: &str,
        top_k: usize,
        alpha: f32,
    ) -> Result<Vec<HybridSearchResult>, String> {
        if caller_tenant_id.trim().is_empty() {
            return Err("TENANT_REQUIRED: caller_tenant_id cannot be empty".into());
        }

        // Pre-filter allowed document IDs belonging exclusively to `caller_tenant_id`
        let allowed_ids: HashSet<String> = {
            let recs = self.vector_col.records.read().unwrap();
            recs.iter()
                .filter(|r| {
                    r.metadata
                        .get("tenant_id")
                        .map(|t| t == caller_tenant_id)
                        .unwrap_or(false)
                })
                .map(|r| r.id.clone())
                .collect()
        };

        if allowed_ids.is_empty() {
            return Ok(Vec::new());
        }

        let raw = self.search(query_text, None, top_k * 4, alpha);
        let filtered: Vec<HybridSearchResult> = raw
            .into_iter()
            .filter(|r| allowed_ids.contains(&r.id))
            .take(top_k)
            .collect();
        Ok(filtered)
    }

    /// Reciprocal Rank Fusion (RRF) combining BM25 rank + Dense Cosine rank (`WP-13`)
    /// `score = 1 / (rrf_k + rank_dense) + 1 / (rrf_k + rank_sparse)`
    pub fn search_rrf(&self, query_text: &str, top_k: usize, rrf_k: f32) -> Vec<HybridSearchResult> {
        let k = rrf_k.max(1.0);
        let bm25_ranked = self.bm25.read().unwrap().search(query_text, top_k * 3);
        let q_vec = crate::db::vector::embed_text(query_text, self.vector_col.dimension);
        let dense_ranked = self.vector_col.search(&q_vec, top_k * 3, None);

        let mut rrf_scores: HashMap<String, f32> = HashMap::new();
        let mut sparse_scores: HashMap<String, f32> = HashMap::new();
        let mut dense_scores: HashMap<String, f32> = HashMap::new();

        for (rank_idx, (doc_id, score)) in bm25_ranked.iter().enumerate() {
            let rank = (rank_idx + 1) as f32;
            *rrf_scores.entry(doc_id.clone()).or_insert(0.0) += 1.0 / (k + rank);
            sparse_scores.insert(doc_id.clone(), *score);
        }

        for (rank_idx, d) in dense_ranked.iter().enumerate() {
            let rank = (rank_idx + 1) as f32;
            *rrf_scores.entry(d.id.clone()).or_insert(0.0) += 1.0 / (k + rank);
            dense_scores.insert(d.id.clone(), d.score);
        }

        let mut results = Vec::new();
        for (id, rrf_score) in rrf_scores {
            if let Some(rec) = self.vector_col.get(&id) {
                results.push(HybridSearchResult {
                    id: id.clone(),
                    text: rec.text,
                    hybrid_score: rrf_score,
                    dense_score: *dense_scores.get(&id).unwrap_or(&0.0),
                    sparse_score: *sparse_scores.get(&id).unwrap_or(&0.0),
                    metadata: rec.metadata,
                    media_hash: rec.media_hash,
                    document_id: rec.document_id,
                });
            }
        }

        results.sort_by(|a, b| b.hybrid_score.partial_cmp(&a.hybrid_score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    /// Extract verifiable citations from hybrid retrieval results (`WP-13`)
    pub fn build_citations(results: &[HybridSearchResult]) -> Vec<RetrievalCitation> {
        results
            .iter()
            .map(|r| RetrievalCitation {
                document_id: r.document_id.clone().unwrap_or_else(|| r.id.clone()),
                source_path: r
                    .metadata
                    .get("source_path")
                    .cloned()
                    .unwrap_or_else(|| format!("cas://{}", r.id)),
                line_range: r.metadata.get("line_range").cloned(),
                commit_sha: r.metadata.get("commit_sha").cloned(),
                score: r.hybrid_score,
            })
            .collect()
    }
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|t| t.len() >= 2)
        .map(|s| s.to_string())
        .collect()
}
