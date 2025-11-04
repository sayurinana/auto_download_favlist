use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::runtime::Builder;

use crate::client::{BiliFavClient, ClientOptions};
use crate::csv_utils::{load_existing_bv_ids, write_entries};
use crate::errors::{ExportError, FavlistError};
use crate::models::{FolderInfo, VideoEntry, VideoItem};
use crate::timestamp::{current_timestamp, parse_media_id};

#[derive(Debug, Clone)]
pub struct ExportProgress {
    pub current: u64,
    pub total: Option<u64>,
}

pub type ProgressCallback = Arc<dyn Fn(ExportProgress) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct ExportOptions {
    pub fav_url: String,
    pub csv_path: PathBuf,
    pub encoding: String,
    pub page_size: u32,
    pub cookie: Option<String>,
    pub timeout_secs: u64,
    pub timestamp: Option<String>,
    pub extra_headers: HashMap<String, String>,
    pub base_url: Option<String>,
    pub progress_callback: Option<ProgressCallback>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            fav_url: String::new(),
            csv_path: PathBuf::new(),
            encoding: "utf-8".to_string(),
            page_size: 40,
            cookie: None,
            timeout_secs: 10,
            timestamp: None,
            extra_headers: HashMap::new(),
            base_url: None,
            progress_callback: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportResult {
    pub csv_path: PathBuf,
    pub folder_info: FolderInfo,
    pub new_entries: Vec<VideoEntry>,
    pub timestamp: String,
    pub processed_count: u64,
    pub total_count: Option<u64>,
}

pub async fn export_favlist(mut options: ExportOptions) -> Result<ExportResult, ExportError> {
    let media_id = parse_media_id(&options.fav_url)?;
    let csv_path = options.csv_path.clone();
    let encoding = options.encoding.clone();
    let timestamp = options.timestamp.take().unwrap_or_else(current_timestamp);
    let progress_callback = options.progress_callback.clone();

    let client_options = ClientOptions {
        timeout: Duration::from_secs(options.timeout_secs),
        cookie: options.cookie.clone(),
        extra_headers: options.extra_headers.clone(),
        base_url: options.base_url.clone(),
    };
    let client = BiliFavClient::new(client_options)?;

    let folder_info = client
        .get_folder_info(media_id)
        .await
        .map_err(ExportError::from)?;
    let total_count = if folder_info.media_count > 0 {
        Some(folder_info.media_count as u64)
    } else {
        None
    };
    let page_payloads = client
        .list_videos(media_id, options.page_size)
        .await
        .map_err(ExportError::from)?;

    let mut existing_ids = load_existing_bv_ids(&csv_path, &encoding).map_err(ExportError::from)?;

    let mut new_entries = Vec::new();
    let mut processed_count: u64 = 0;
    if let Some(callback) = progress_callback.as_ref() {
        callback(ExportProgress {
            current: processed_count,
            total: total_count,
        });
    }

    for page in page_payloads {
        for item in page.medias {
            processed_count = processed_count.saturating_add(1);
            if let Some(callback) = progress_callback.as_ref() {
                callback(ExportProgress {
                    current: processed_count,
                    total: total_count,
                });
            }

            if let Some(entry) = build_video_entry(&item, &folder_info.title, &timestamp) {
                if existing_ids.insert(entry.bv_id.clone()) {
                    new_entries.push(entry);
                }
            }
        }
    }

    if new_entries.is_empty() {
        ensure_csv_exists(&csv_path, &encoding)?;
    } else {
        write_entries(&csv_path, &encoding, &new_entries).map_err(ExportError::from)?;
    }

    Ok(ExportResult {
        csv_path,
        folder_info,
        new_entries,
        timestamp,
        processed_count,
        total_count,
    })
}

pub fn export_favlist_blocking(options: ExportOptions) -> Result<ExportResult, ExportError> {
    let rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| FavlistError::Other(format!("Tokio运行时初始化失败: {err}")))?;
    rt.block_on(export_favlist(options))
}

fn build_video_entry(item: &VideoItem, fav_title: &str, timestamp: &str) -> Option<VideoEntry> {
    let bvid = item.resolve_bvid()?;
    Some(VideoEntry {
        bv_id: bvid,
        title: item.title.trim().to_string(),
        fav_title: fav_title.to_string(),
        timestamp: timestamp.to_string(),
        aid: item.id,
    })
}

fn ensure_csv_exists(path: &Path, encoding: &str) -> Result<(), ExportError> {
    write_entries(path, encoding, &[])
        .map(|_| ())
        .map_err(ExportError::from)
}
