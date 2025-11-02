use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FolderInfo {
    pub media_id: i64,
    pub fid: i64,
    pub mid: i64,
    pub title: String,
    pub media_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoEntry {
    pub bv_id: String,
    pub title: String,
    pub fav_title: String,
    pub timestamp: String,
    pub aid: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: Option<String>,
    pub data: Option<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FolderInfoPayload {
    pub id: Option<i64>,
    pub fid: Option<i64>,
    pub mid: Option<i64>,
    pub title: Option<String>,
    #[serde(default)]
    pub media_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceListPayload {
    #[serde(default)]
    pub medias: Vec<VideoItem>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoItem {
    #[serde(default)]
    pub bv_id: String,
    #[serde(default)]
    pub bvid: String,
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub title: String,
}

impl FolderInfoPayload {
    pub fn into_folder_info(self, fallback_media_id: i64) -> FolderInfo {
        FolderInfo {
            media_id: self.id.unwrap_or(fallback_media_id),
            fid: self.fid.unwrap_or_default(),
            mid: self.mid.unwrap_or_default(),
            title: self.title.unwrap_or_default(),
            media_count: self.media_count,
        }
    }
}

impl VideoItem {
    pub fn resolve_bvid(&self) -> Option<String> {
        if !self.bv_id.trim().is_empty() {
            Some(self.bv_id.trim().to_string())
        } else if !self.bvid.trim().is_empty() {
            Some(self.bvid.trim().to_string())
        } else {
            self.id.map(|aid| format!("av{aid}"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub bvid: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySnapshot {
    pub generated_at: DateTime<FixedOffset>,
    pub items: Vec<InventoryItem>,
}
