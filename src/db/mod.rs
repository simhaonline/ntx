//! Nityaksha Pure Rust Complete Agent Memory Stack & Vector DB Engine
//! Relational + Hybrid Vectors, Columnar + Multimodal, In-Memory or Disk Directory, Automatic Version Tracking.

pub mod vector;
pub mod hybrid;
pub mod multimodal;
pub mod columnar;
pub mod relational;
pub mod versioning;

use crate::db::multimodal::to_hex;
use sha2::Digest;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use columnar::{AggregatedStats, ColumnarTable, TelemetryRow};
pub use hybrid::{Bm25Index, HybridRagEngine, HybridSearchResult};
pub use multimodal::{MediaAsset, MediaType, MultimodalStore};
pub use relational::{
    AuditEventRow, CapabilityRow, PalaceDrawerRow, RelationalStore, RelationalTables, SourceRow,
    TodoRow, TrajectoryRow,
};
pub use vector::{MetricType, VectorCollection, VectorRecord, VectorSearchResult};
pub use versioning::{VersionCommit, VersionManager, VersionTimeline};

/// Master Agent Memory Engine
pub struct MemoryEngine {
    pub storage_dir: Option<PathBuf>,
    pub relational: Arc<RelationalStore>,
    pub multimodal: Arc<MultimodalStore>,
    pub vector: Arc<VectorCollection>,
    pub hybrid_rag: Arc<HybridRagEngine>,
    pub columnar: Arc<parking_lot_or_std::RwLock<ColumnarTable>>,
    pub versioning: Arc<VersionManager>,
}

mod parking_lot_or_std {
    pub use std::sync::RwLock;
}

impl MemoryEngine {
    /// Open database backed by a disk directory (e.g. `/var/lib/nityaksha/db`)
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let dir = path.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;

        let relational = Arc::new(RelationalStore::open(dir.join("relational"))?);
        let multimodal = Arc::new(MultimodalStore::open(dir.join("media"))?);
        let vector = Arc::new(VectorCollection::open(dir.join("vectors"), "default", 384, MetricType::Cosine)?);
        let hybrid_rag = Arc::new(HybridRagEngine::new(vector.clone()));
        let columnar = Arc::new(parking_lot_or_std::RwLock::new(ColumnarTable::new()));
        let versioning = Arc::new(VersionManager::new());

        let engine = Self {
            storage_dir: Some(dir),
            relational,
            multimodal,
            vector,
            hybrid_rag,
            columnar,
            versioning,
        };

        // Ingest existing catalog files from /opt/nityaksha/catalog into relational tables
        engine.sync_filesystem_catalog(Path::new("/opt/nityaksha/catalog"))?;

        Ok(engine)
    }

    /// Open in-memory fast conversational prototype mode
    pub fn in_memory() -> Self {
        let relational = Arc::new(RelationalStore::new());
        let multimodal = Arc::new(MultimodalStore::in_memory());
        let vector = Arc::new(VectorCollection::new("in_memory", 384, MetricType::Cosine));
        let hybrid_rag = Arc::new(HybridRagEngine::new(vector.clone()));
        let columnar = Arc::new(parking_lot_or_std::RwLock::new(ColumnarTable::new()));
        let versioning = Arc::new(VersionManager::new());

        Self {
            storage_dir: None,
            relational,
            multimodal,
            vector,
            hybrid_rag,
            columnar,
            versioning,
        }
    }

    /// Sync capabilities from catalog directory into relational memory
    pub fn sync_filesystem_catalog(&self, catalog_path: &Path) -> io::Result<()> {
        if !catalog_path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(catalog_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let kind = path.file_name().unwrap().to_string_lossy().trim_end_matches('s').to_string();
                for file in fs::read_dir(&path)? {
                    let file = file?;
                    let fpath = file.path();
                    if fpath.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                        if let Ok(content) = fs::read_to_string(&fpath) {
                            let mut name = fpath.file_stem().unwrap().to_string_lossy().to_string();
                            let mut desc = String::new();
                            let mut origin_repo = "simhaonline/ntx".to_string();
                            let mut orig_name = name.clone();
                            let mut ver = "1.0.0".to_string();

                            for line in content.lines() {
                                let line = line.trim();
                                if line.starts_with("name:") {
                                    name = line["name:".len()..].trim().trim_matches('"').trim_matches('\'').to_string();
                                } else if line.starts_with("original_name:") {
                                    orig_name = line["original_name:".len()..].trim().trim_matches('"').trim_matches('\'').to_string();
                                } else if line.starts_with("origin_repository:") || line.starts_with("repository:") {
                                    origin_repo = line.split(':').nth(1).unwrap_or("").trim().trim_matches('"').trim_matches('\'').to_string();
                                } else if line.starts_with("description:") {
                                    desc = line["description:".len()..].trim().trim_matches('"').trim_matches('\'').to_string();
                                } else if line.starts_with("version:") {
                                    ver = line["version:".len()..].trim().trim_matches('"').trim_matches('\'').to_string();
                                }
                            }

                            if !name.starts_with("ntx-") {
                                orig_name = name.clone();
                                name = format!("ntx-{}", name);
                            }

                            let id = format!("{}:{}", kind, name);
                            let content_sha256 = to_hex(&sha2::Sha256::digest(content.as_bytes()));

                            let full_text = format!("{} {} {}", name, desc, origin_repo);
                            let values = crate::db::vector::embed_text(&full_text, 384);
                            let mut meta = std::collections::HashMap::new();
                            meta.insert("kind".to_string(), kind.clone());
                            meta.insert("origin_repo".to_string(), origin_repo.clone());
                            meta.insert("name".to_string(), name.clone());

                            let _ = self.vector.insert(VectorRecord {
                                id: id.clone(),
                                values,
                                text: full_text.clone(),
                                metadata: meta,
                                media_hash: None,
                                document_id: Some(id.clone()),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            });
                            self.hybrid_rag.index_record(&id, &full_text);

                            let _ = self.relational.upsert_capability(CapabilityRow {
                                id,
                                namespace: "default".into(),
                                kind: kind.clone(),
                                name,
                                version: ver,
                                description: desc,
                                manifest_path: fpath.to_string_lossy().to_string(),
                                trust_tier: "approved".into(),
                                content_sha256,
                                origin_repo,
                                original_name: orig_name,
                                created_at: chrono::Utc::now().to_rfc3339(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
