use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use dirs_next::config_dir;
use serde::{Deserialize, Serialize};

const APP_DIR: &str = "bilibili_favlist_helper";
const CONFIG_NAME: &str = "config.json";
const DEFAULTS_NAME: &str = "defaults.json";
pub const DEFAULT_BBDOWN_URL: &str = "http://localhost:23333";
pub const DEFAULT_POLL_INTERVAL_MS: u64 = 500;

fn default_bbdown_url() -> String {
    DEFAULT_BBDOWN_URL.to_string()
}

fn default_true() -> bool {
    true
}

fn default_poll_interval() -> u64 {
    DEFAULT_POLL_INTERVAL_MS
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavConfig {
    pub fav_url: String,
    #[serde(rename = "download_dir")]
    pub api_download_dir: String,
    #[serde(default)]
    pub scan_download_dir: Option<String>,
    pub csv_path: String,
    pub encoding: String,
    pub page_size: u32,
    pub cookie: Option<String>,
    pub timeout_secs: u64,
    pub last_synced_at: Option<String>,
    pub name: Option<String>,
    #[serde(default = "default_bbdown_url")]
    pub bbdown_serve_url: String,
    #[serde(default = "default_true")]
    pub bbdown_auto_launch: bool,
    #[serde(default)]
    pub bbdown_launch_args: Vec<String>,
    #[serde(default = "default_poll_interval")]
    pub bbdown_poll_interval_ms: u64,
    #[serde(default)]
    pub file_pattern: Option<String>,
    #[serde(default)]
    pub multi_file_pattern: Option<String>,
}

impl FavConfig {
    pub fn scan_download_dir_path(&self) -> PathBuf {
        PathBuf::from(self.scan_download_dir())
    }

    pub fn csv_path(&self) -> PathBuf {
        PathBuf::from(&self.csv_path)
    }

    pub fn apply_defaults(&mut self) {
        if self.bbdown_serve_url.is_empty() {
            self.bbdown_serve_url = default_bbdown_url();
        }
        if self.bbdown_poll_interval_ms == 0 {
            self.bbdown_poll_interval_ms = DEFAULT_POLL_INTERVAL_MS;
        }
        if self.scan_download_dir.as_deref().unwrap_or("").is_empty() {
            let fallback = Path::new(&self.csv_path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| self.api_download_dir.clone());
            self.scan_download_dir = Some(fallback);
        }
    }

    pub fn effective_serve_url(&self) -> &str {
        self.bbdown_serve_url.as_str()
    }

    pub fn resolve_file_pattern(&self) -> Option<String> {
        self.file_pattern
            .as_ref()
            .map(|pattern| join_download_path(&self.api_download_dir, pattern))
    }

    pub fn resolve_multi_file_pattern(&self) -> Option<String> {
        self.multi_file_pattern
            .as_ref()
            .map(|pattern| join_download_path(&self.api_download_dir, pattern))
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.bbdown_poll_interval_ms.max(50))
    }

    pub fn scan_download_dir(&self) -> &str {
        self.scan_download_dir
            .as_deref()
            .unwrap_or(&self.api_download_dir)
    }
}

#[derive(Debug)]
pub struct ConfigStore {
    path: PathBuf,
    configs: Vec<FavConfig>,
}

impl ConfigStore {
    pub fn load(custom_path: Option<PathBuf>) -> Result<Self> {
        let path = custom_path.unwrap_or_else(default_config_path);
        let mut configs: Vec<FavConfig> = if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
            serde_json::from_str(&content).with_context(|| "解析配置文件失败")?
        } else {
            Vec::new()
        };
        configs.iter_mut().for_each(FavConfig::apply_defaults);
        Ok(Self { path, configs })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录失败: {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(&self.configs)?;
        fs::write(&self.path, json).with_context(|| "写入配置文件失败")?;
        Ok(())
    }

    pub fn configs(&self) -> &[FavConfig] {
        &self.configs
    }

    pub fn add(&mut self, mut config: FavConfig) -> Result<()> {
        config.apply_defaults();
        self.configs.push(config);
        self.save()
    }

    pub fn update(&mut self, index: usize, mut config: FavConfig) -> Result<()> {
        if index >= self.configs.len() {
            return Err(anyhow!("配置索引超出范围"));
        }
        config.apply_defaults();
        self.configs[index] = config;
        self.save()
    }

    pub fn config_dir(&self) -> PathBuf {
        self.path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

fn default_config_path() -> PathBuf {
    let mut base = config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.push(APP_DIR);
    base.push(CONFIG_NAME);
    base
}

fn join_download_path(base: &str, pattern: &str) -> String {
    if base.contains(':') {
        let mut sanitized = base.trim_end_matches(['\\', '/']).to_string();
        if sanitized.is_empty() {
            pattern.to_string()
        } else {
            sanitized.push('\\');
            sanitized.push_str(pattern);
            sanitized
        }
    } else {
        let mut path = Path::new(base).to_path_buf();
        path.push(pattern);
        path.to_string_lossy().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalDefaults {
    pub api_download_dir: Option<String>,
    pub scan_download_dir: Option<String>,
    pub bbdown_serve_url: Option<String>,
    pub file_pattern: Option<String>,
    pub multi_file_pattern: Option<String>,
}

impl GlobalDefaults {}

pub struct GlobalDefaultsStore {
    path: PathBuf,
    data: GlobalDefaults,
}

impl GlobalDefaultsStore {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let path = config_dir.join(DEFAULTS_NAME);
        let data = if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("读取全局默认失败: {}", path.display()))?;
            serde_json::from_str(&content).with_context(|| "解析全局默认失败")?
        } else {
            GlobalDefaults::default()
        };
        Ok(Self { path, data })
    }

    pub fn data(&self) -> &GlobalDefaults {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut GlobalDefaults {
        &mut self.data
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建全局默认目录失败: {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, json).with_context(|| "写入全局默认失败")?;
        Ok(())
    }
}
