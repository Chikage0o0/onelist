pub mod item;
pub mod thumb;

use std::sync::Arc;

use mini_moka::sync::Cache;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Caches {
    /// Cache for download URLs
    pub download_url_cache: Cache<String, String>,
    /// Cache for folder contents
    pub list_cache: Cache<String, Arc<Vec<FileInfo>>>,
    /// Cache for thumbnails
    pub thumb_cache: Cache<String, Arc<Thumbnails>>,
    /// Cache for file info
    pub file_cache: Cache<String, Arc<FileInfo>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thumbnails {
    pub small: String,
    pub medium: String,
    pub large: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileInfo {
    id: String,
    name: String,
    size: i64,
    last_modified_date_time: i64,
    full_path: String,
    #[serde(rename = "type")]
    file_type: FileTypes,
}

#[derive(Debug, Serialize, Clone)]
pub enum FileTypes {
    File,
    Folder,
    Video,
    Audio,
}

fn date_time_to_timestamp(date_time: Option<String>) -> i64 {
    date_time
        .and_then(|date_time| {
            chrono::DateTime::parse_from_rfc3339(&date_time)
                .ok()
                .map(|date_time| date_time.timestamp())
        })
        .unwrap_or_default()
}
