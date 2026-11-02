//! Nityaksha Multimodal & Binary Media Engine
//! Co-locating raw media files (images, charts, screenshots, spreadsheets, PDFs, audio logs) alongside text & vectors.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Pdf,
    Chart,
    Spreadsheet,
    Audio,
    Document,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    pub hash: String, // SHA-256
    pub media_type: MediaType,
    pub mime_type: String,
    pub filename: String,
    pub byte_size: usize,
    pub metadata: HashMap<String, String>,
    pub created_at: String,
}

pub struct MultimodalStore {
    root_dir: PathBuf,
    assets: Arc<RwLock<HashMap<String, MediaAsset>>>,
}

impl MultimodalStore {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let root = path.as_ref().to_path_buf();
        let blobs_dir = root.join("blobs");
        fs::create_dir_all(&blobs_dir)?;

        let mut store = Self {
            root_dir: root,
            assets: Arc::new(RwLock::new(HashMap::new())),
        };

        store.load_index()?;
        Ok(store)
    }

    pub fn in_memory() -> Self {
        Self {
            root_dir: PathBuf::from("/tmp/ntx-media-mem"),
            assets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a raw binary asset into Content-Addressable Storage (CAS)
    pub fn store_asset(
        &self,
        filename: &str,
        data: &[u8],
        custom_mime: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> io::Result<MediaAsset> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = to_hex(&hasher.finalize());

        let mime = custom_mime
            .map(|s| s.to_string())
            .unwrap_or_else(|| detect_mime_type(filename, data));

        let media_type = classify_media(&mime);

        let asset = MediaAsset {
            hash: hash.clone(),
            media_type,
            mime_type: mime,
            filename: filename.to_string(),
            byte_size: data.len(),
            metadata,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let blobs_dir = self.root_dir.join("blobs");
        if blobs_dir.exists() {
            let blob_path = blobs_dir.join(&hash);
            if !blob_path.exists() {
                fs::write(blob_path, data)?;
            }
        }

        self.assets.write().unwrap().insert(hash.clone(), asset.clone());
        self.save_index()?;

        Ok(asset)
    }

    pub fn get_asset(&self, hash: &str) -> Option<MediaAsset> {
        self.assets.read().unwrap().get(hash).cloned()
    }

    pub fn read_bytes(&self, hash: &str) -> io::Result<Vec<u8>> {
        let blob_path = self.root_dir.join("blobs").join(hash);
        if blob_path.exists() {
            fs::read(blob_path)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, format!("Media blob {} not found", hash)))
        }
    }

    pub fn list_assets(&self) -> Vec<MediaAsset> {
        self.assets.read().unwrap().values().cloned().collect()
    }

    fn save_index(&self) -> io::Result<()> {
        let index_path = self.root_dir.join("index.json");
        if self.root_dir.exists() {
            let assets = self.assets.read().unwrap();
            let data = serde_json::to_vec_pretty(&*assets)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let temp = index_path.with_extension("tmp");
            fs::write(&temp, data)?;
            fs::rename(temp, index_path)?;
        }
        Ok(())
    }

    fn load_index(&mut self) -> io::Result<()> {
        let index_path = self.root_dir.join("index.json");
        if index_path.exists() {
            let data = fs::read(&index_path)?;
            if !data.is_empty() {
                let assets: HashMap<String, MediaAsset> =
                    serde_json::from_slice(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                *self.assets.write().unwrap() = assets;
            }
        }
        Ok(())
    }
}

pub fn detect_mime_type(filename: &str, data: &[u8]) -> String {
    // Magic bytes sniffing
    if data.starts_with(b"%PDF") {
        return "application/pdf".to_string();
    }
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return "image/png".to_string();
    }
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return "image/jpeg".to_string();
    }
    if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
        return "image/webp".to_string();
    }
    if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WAVE" {
        return "audio/wav".to_string();
    }
    if data.starts_with(b"<svg") || (data.len() > 50 && data[..50].windows(4).any(|w| w == b"<svg")) {
        return "image/svg+xml".to_string();
    }

    // Extension fallback
    let ext = filename.split('.').last().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => "application/pdf".to_string(),
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "webp" => "image/webp".to_string(),
        "svg" => "image/svg+xml".to_string(),
        "csv" => "text/csv".to_string(),
        "tsv" => "text/tab-separated-values".to_string(),
        "json" => "application/json".to_string(),
        "mp3" => "audio/mpeg".to_string(),
        "wav" => "audio/wav".to_string(),
        "txt" | "md" => "text/plain".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

pub fn classify_media(mime: &str) -> MediaType {
    if mime == "application/pdf" {
        MediaType::Pdf
    } else if mime.starts_with("image/") {
        if mime == "image/svg+xml" {
            MediaType::Chart
        } else {
            MediaType::Image
        }
    } else if mime == "text/csv" || mime == "text/tab-separated-values" {
        MediaType::Spreadsheet
    } else if mime.starts_with("audio/") {
        MediaType::Audio
    } else if mime.starts_with("text/") || mime == "application/json" {
        MediaType::Document
    } else {
        MediaType::Binary
    }
}
